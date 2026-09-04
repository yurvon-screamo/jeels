use idb::{Database, DatabaseEvent, Factory, ObjectStoreParams, TransactionMode};
use origa::{
    domain::{OrigaError, User},
    traits::UserRepository,
};
use ulid::Ulid;
use wasm_bindgen::JsValue;

use super::hybrid_repository::LocalUserPresence;

pub(crate) const DB_NAME: &str = "origa";
pub(crate) const DB_VERSION: u32 = 1;
pub(crate) const STORE_NAME: &str = "users";

pub(crate) fn user_key(user_id: Ulid) -> String {
    format!("user:{}", user_id)
}

/// Encodes a user into the value stored in IndexedDB: a JSON **string**.
/// A flat string is deliberate (#492): the previous structured-clone JS
/// object made every `put` clone a multi-megabyte object graph for a
/// full-corpus user, and that clone path stalled the request completion on
/// real hardware — the restore window never closed. A string clones as a
/// plain byte copy. JSON specifically (not bincode/postcard): the user
/// hierarchy transitively contains a `#[serde(flatten)]` field
/// (`KnowledgeSet.stats`), which binary formats cannot encode
/// (unknown-length maps); JSON is also the wire format the record already
/// uses server-side (`user_to_json`).
pub(crate) fn user_to_stored_value(user: &User) -> Result<JsValue, OrigaError> {
    let json = serde_json::to_string(user).map_err(|e| {
        let reason = format!("User JSON encode failed: {e}");
        tracing::error!("{reason}");
        OrigaError::RepositoryError { reason }
    })?;
    Ok(JsValue::from_str(&json))
}

/// Decodes a stored value back into a user. Accepts both formats:
/// the current JSON string and the legacy structured-clone JS object
/// written before the switch — records upgrade transparently on the next
/// save. `Err` carries the decode reason (bad JSON vs schema mismatch) so
/// callers can log WHY a record was treated as corrupted — the restore
/// path is exactly where that detail mattered during the #492
/// investigation.
pub(crate) fn user_from_stored_value(value: &JsValue) -> Result<User, String> {
    if let Some(json) = value.as_string() {
        return serde_json::from_str(&json)
            .map_err(|e| format!("JSON string record failed to decode: {e}"));
    }
    serde_wasm_bindgen::from_value(value.clone())
        .map_err(|e| format!("legacy object record failed to decode: {e}"))
}

/// Key range covering every `user:*` record and nothing else: the store
/// also holds non-user records (the `sync_meta` key, see
/// `sync_meta_store`), so listings and existence probes must be bounded or
/// the meta record surfaces as a spurious "corrupted user entry".
/// `user;` is the next code point after the prefix's last character, so
/// the exclusive upper bound covers every `user:*` key.
pub(crate) fn user_key_range() -> Result<idb::KeyRange, OrigaError> {
    let lower_bound = JsValue::from_str("user:");
    let upper_bound = JsValue::from_str("user;");
    idb::KeyRange::bound(&lower_bound, &upper_bound, None, Some(true)).map_err(|e| {
        let reason = format!("Failed to build user key range: {e:?}");
        tracing::error!("{}", reason);
        OrigaError::RepositoryError { reason }
    })
}

pub(crate) async fn open_database() -> Result<Database, OrigaError> {
    let factory = Factory::new().map_err(|e| {
        let reason = format!("Failed to create IndexedDB factory: {:?}", e);
        tracing::error!("{}", reason);
        OrigaError::RepositoryError { reason }
    })?;

    let mut open_request = factory.open(DB_NAME, Some(DB_VERSION)).map_err(|e| {
        let reason = format!("Failed to open IndexedDB: {:?}", e);
        tracing::error!("{}", reason);
        OrigaError::RepositoryError { reason }
    })?;

    open_request.on_upgrade_needed(|event| {
        let database = match event.database() {
            Ok(db) => db,
            Err(e) => {
                tracing::error!("Failed to get database: {:?}", e);
                return;
            },
        };

        if database.store_names().iter().any(|n| n == STORE_NAME) {
            return;
        }

        let store_params = ObjectStoreParams::new();

        match database.create_object_store(STORE_NAME, store_params) {
            Ok(_) => tracing::info!("Object store 'users' created"),
            Err(e) => tracing::error!("Failed to create object store: {:?}", e),
        }
    });

    open_request.await.map_err(|e| {
        let reason = format!("Failed to initialize IndexedDB: {:?}", e);
        tracing::error!("{}", reason);
        OrigaError::RepositoryError { reason }
    })
}

#[derive(Clone)]
pub struct FileSystemUserRepository {}

impl FileSystemUserRepository {
    pub fn new() -> Self {
        Self {}
    }

    /// Cheap existence check for the sync skip-path (ADR-045): an IndexedDB
    /// key count over the `user:` prefix — the multi-megabyte user record
    /// is never fetched or parsed.
    pub(crate) async fn has_any_user(&self) -> Result<bool, OrigaError> {
        let db = open_database().await?;

        let transaction = db
            .transaction(&[STORE_NAME], TransactionMode::ReadOnly)
            .map_err(|e| {
                let reason = format!("Failed to create transaction: {e:?}");
                tracing::error!("{reason}");
                OrigaError::RepositoryError { reason }
            })?;

        let store = transaction.object_store(STORE_NAME).map_err(|e| {
            let reason = format!("Failed to get object store: {e:?}");
            tracing::error!("{reason}");
            OrigaError::RepositoryError { reason }
        })?;

        let count = store
            .count(Some(idb::Query::from(user_key_range()?)))
            .map_err(|e| {
                let reason = format!("Failed to create count request: {e:?}");
                tracing::error!("{reason}");
                OrigaError::RepositoryError { reason }
            })?
            .await
            .map_err(|e| {
                let reason = format!("Failed to count users: {e:?}");
                tracing::error!("{reason}");
                OrigaError::RepositoryError { reason }
            })?;

        Ok(count > 0)
    }

    async fn list_users(&self) -> Result<Vec<User>, OrigaError> {
        let db = open_database().await?;

        let transaction = db
            .transaction(&[STORE_NAME], TransactionMode::ReadOnly)
            .map_err(|e| {
                let reason = format!("Failed to create transaction: {:?}", e);
                tracing::error!("{}", reason);
                OrigaError::RepositoryError { reason }
            })?;

        let store = transaction.object_store(STORE_NAME).map_err(|e| {
            let reason = format!("Failed to get object store: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        let request = store
            .get_all(Some(idb::Query::from(user_key_range()?)), None)
            .map_err(|e| {
                let reason = format!("Failed to create get_all request: {e:?}");
                tracing::error!("{reason}");
                OrigaError::RepositoryError { reason }
            })?;

        let all_values: Vec<JsValue> = request.await.map_err(|e| {
            let reason = format!("Failed to get all users: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        let mut users = vec![];
        for value in all_values {
            match user_from_stored_value(&value) {
                Ok(user) => users.push(user),
                Err(reason) => {
                    tracing::warn!("Skipping corrupted user entry in IndexedDB: {reason}");
                },
            }
        }

        Ok(users)
    }
}

impl LocalUserPresence for FileSystemUserRepository {
    async fn has_any_user(&self) -> Result<bool, OrigaError> {
        FileSystemUserRepository::has_any_user(self).await
    }
}

impl UserRepository for FileSystemUserRepository {
    async fn get_current_user(&self) -> Result<Option<User>, OrigaError> {
        // Self-heal legacy nil-keyed rows before reading so that the first
        // read after upgrade returns the canonical record. Migration errors are
        // non-fatal: the read still proceeds against whatever rows exist.
        if let Err(e) = super::legacy_migration::migrate_nil_users_to_session_id().await {
            tracing::warn!("Legacy nil-user migration skipped: {:?}", e);
        }

        let users = self.list_users().await?;
        Ok(users.into_iter().next())
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        let db = open_database().await?;

        let transaction = db
            .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
            .map_err(|e| {
                let reason = format!("Failed to create transaction: {:?}", e);
                tracing::error!("{}", reason);
                OrigaError::RepositoryError { reason }
            })?;

        let store = transaction.object_store(STORE_NAME).map_err(|e| {
            let reason = format!("Failed to get object store: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        let key = user_key(user.id());
        let value = user_to_stored_value(user)?;

        let request = store
            .put(&value, Some(&JsValue::from_str(&key)))
            .map_err(|e| {
                let reason = format!("Failed to create put request: {:?}", e);
                tracing::error!("{}", reason);
                OrigaError::RepositoryError { reason }
            })?;

        request.await.map_err(|e| {
            let reason = format!("Failed to save user: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        Ok(())
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), OrigaError> {
        let db = open_database().await?;

        let transaction = db
            .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
            .map_err(|e| {
                let reason = format!("Failed to create transaction: {:?}", e);
                tracing::error!("{}", reason);
                OrigaError::RepositoryError { reason }
            })?;

        let store = transaction.object_store(STORE_NAME).map_err(|e| {
            let reason = format!("Failed to get object store: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        let key = JsValue::from_str(&user_key(user_id));

        let request = store.delete(key).map_err(|e| {
            let reason = format!("Failed to create delete request: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        request.await.map_err(|e| {
            let reason = format!("Failed to delete user: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        Ok(())
    }
}
