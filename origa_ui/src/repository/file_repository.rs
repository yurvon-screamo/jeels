use idb::{Database, DatabaseEvent, Factory, ObjectStoreParams, TransactionMode};
use origa::{
    domain::{OrigaError, User},
    traits::UserRepository,
};
use ulid::Ulid;
use wasm_bindgen::JsValue;

pub(crate) const DB_NAME: &str = "origa";
pub(crate) const DB_VERSION: u32 = 1;
pub(crate) const STORE_NAME: &str = "users";

pub(crate) fn user_key(user_id: Ulid) -> String {
    format!("user:{}", user_id)
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

        let request = store.get_all(None, None).map_err(|e| {
            let reason = format!("Failed to create get_all request: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        let all_values: Vec<JsValue> = request.await.map_err(|e| {
            let reason = format!("Failed to get all users: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

        let mut users = vec![];
        for value in all_values {
            match serde_wasm_bindgen::from_value::<User>(value) {
                Ok(user) => users.push(user),
                Err(e) => {
                    tracing::warn!("Skipping corrupted user entry in IndexedDB: {:?}", e);
                },
            }
        }

        Ok(users)
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
        // Backfill the onboarding-completed marker for users persisted before
        // the marker existed. Idempotent — no-op once the marker is present.
        // The marker lives in memory after this read and is written back to
        // storage lazily on the next `save` (rate card, etc.); routing only
        // needs the in-memory view.
        let mut users: Vec<User> = users;
        for user in &mut users {
            user.migrate_onboarding_state();
        }
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
        let value = serde_wasm_bindgen::to_value(&user).map_err(|e| {
            let reason = format!("Failed to serialize user: {:?}", e);
            tracing::error!("{}", reason);
            OrigaError::RepositoryError { reason }
        })?;

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
