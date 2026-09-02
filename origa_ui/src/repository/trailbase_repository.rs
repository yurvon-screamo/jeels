use super::session::{TrailBaseSession, get_session, set_session_async};
use super::trailbase_client::{AuthError, TrailBaseClient};
use super::trailbase_id::uuid_to_ulid;
use chrono::{DateTime, Utc};
use origa::domain::{DailyLoad, KnowledgeSet, NativeLanguage, OrigaError, User};
use origa::traits::UserRepository;
use std::collections::HashSet;
use std::future::Future;
use ulid::Ulid;

use super::knowledge_set_codec;
use super::wire_fingerprint::wire_row_fingerprint;

#[cfg(test)]
#[path = "trailbase_repository_tests.rs"]
mod tests;

#[derive(Clone)]
pub struct TrailBaseUserRepository {
    client: TrailBaseClient,
    table_name: String,
}

/// A fetched remote row in its cheap form: the wire fields stay
/// undecoded (`knowledge_set` remains a raw string), and the content
/// fingerprint is computed before any materialization, so the sync
/// short-circuit (ADR-045) can skip without paying the multi-megabyte
/// inflate+parse of a large knowledge set.
pub(crate) struct RemoteRow {
    pub record_id: i64,
    pub fingerprint: String,
    row: UserRow,
}

impl RemoteRow {
    /// Decodes the row into a `User`. The nil-identity guard lives here so
    /// every consumer of a materialized remote user (sync, login) inherits
    /// it: a row whose `trailbase_id` does not decode to a real ULID would
    /// poison the local identity through `User::merge`.
    pub(crate) fn into_user(self) -> Result<User, OrigaError> {
        let user = self.row.to_user();
        if user.id() == Ulid::nil() {
            return Err(OrigaError::RepositoryError {
                reason: "Remote user trailbase_id did not decode to a valid ULID; refusing to sync a nil identity".to_string(),
            });
        }
        Ok(user)
    }
}

/// Builds a [`RemoteRow`] from a raw wire row. A row without a numeric
/// record id is an explicit error — silently skipping it would make the
/// sync fall through to `create` and duplicate the row (ADR-045).
pub(crate) fn remote_row_from_value(row: serde_json::Value) -> Result<RemoteRow, OrigaError> {
    let Some(record_id) = row.get("id").and_then(serde_json::Value::as_i64) else {
        return Err(OrigaError::RepositoryError {
            reason: "Record ID missing from database row".to_string(),
        });
    };

    let fingerprint = wire_row_fingerprint(&row);
    let parsed: UserRow = serde_json::from_value(row).map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to parse remote user row: {e}"),
    })?;

    Ok(RemoteRow {
        record_id,
        fingerprint,
        row: parsed,
    })
}

/// The remote half of the sync orchestration (ADR-045). Split from
/// [`UserRepository`] so the orchestration can be tested against an
/// in-memory spy and so raw-row access (no decode) is first-class.
pub(crate) trait RemoteUserSource {
    fn find_current_raw(&self) -> impl Future<Output = Result<Option<RemoteRow>, OrigaError>>;

    fn save_with_record_id(
        &self,
        record_id: i64,
        user: &User,
    ) -> impl Future<Output = Result<(), OrigaError>>;

    fn create(&self, user: &User) -> impl Future<Output = Result<i64, OrigaError>>;
}

fn map_auth_error(e: AuthError) -> OrigaError {
    match e {
        AuthError::SessionExpired => OrigaError::SessionExpired,
        AuthError::NetworkError(msg) => OrigaError::RepositoryError {
            reason: format!("Network error: {}", msg),
        },
        AuthError::ApiError(msg) => OrigaError::RepositoryError {
            reason: format!("API error: {}", msg),
        },
    }
}

impl TrailBaseUserRepository {
    pub fn new() -> Self {
        Self {
            client: TrailBaseClient::new(),
            table_name: "domain_user".to_string(),
        }
    }

    fn require_session(&self) -> Result<TrailBaseSession, OrigaError> {
        let session = get_session().ok_or_else(|| OrigaError::RepositoryError {
            reason: "Not authenticated".to_string(),
        })?;

        if session.email.is_empty() {
            return Err(OrigaError::RepositoryError {
                reason: "Email not found in session. Please re-login.".to_string(),
            });
        }

        Ok(session)
    }

    /// Fetches the current user's remote row without decoding it: the
    /// fingerprint covers every server column except `updated_at`
    /// (see `wire_fingerprint`). Duplicate rows (same email, legacy
    /// create-races) resolve to the smallest record id so the choice is
    /// deterministic across syncs.
    pub(crate) async fn find_current_raw(&self) -> Result<Option<RemoteRow>, OrigaError> {
        let session = self.require_session()?;

        let api = self.client.records(&self.table_name);
        let rows: Vec<serde_json::Value> = api
            .list_filtered("email", &session.email)
            .await
            .map_err(map_auth_error)?;

        let mut selected: Option<(i64, serde_json::Value)> = None;
        for row in rows {
            // A row without a numeric id is a broken server response: error
            // out instead of skipping — a skip would make the caller fall
            // through to `create` and duplicate the row (ADR-045).
            let Some(id) = row.get("id").and_then(serde_json::Value::as_i64) else {
                return Err(OrigaError::RepositoryError {
                    reason: "Record ID missing from database row".to_string(),
                });
            };
            let replaces_selection = match selected.as_ref() {
                None => true,
                Some((min_id, _)) => id < *min_id,
            };
            if replaces_selection {
                selected = Some((id, row));
            }
        }

        match selected {
            Some((_, row)) => remote_row_from_value(row).map(Some),
            None => Ok(None),
        }
    }

    pub async fn find_current(&self) -> Result<Option<(User, i64)>, OrigaError> {
        match self.find_current_raw().await? {
            Some(raw) => {
                let record_id = raw.record_id;
                Ok(Some((raw.into_user()?, record_id)))
            },
            None => Ok(None),
        }
    }
}

impl Default for TrailBaseUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct UserRow {
    #[serde(default)]
    id: Option<i64>,
    trailbase_id: String,
    username: String,
    email: String,
    native_language: i32,
    jlpt_progress: Option<String>,
    current_japanese_level: Option<i32>,
    telegram_user_id: Option<i64>,
    knowledge_set: Option<String>,
    updated_at: DateTime<Utc>,
    imported_sets: Option<String>,
    #[serde(default)]
    daily_load: Option<i32>,
    #[serde(default)]
    known_vocab_hash: Option<i32>,
}

impl UserRow {
    fn to_user(&self) -> User {
        let ulid = uuid_to_ulid(&self.trailbase_id);

        let jlpt_progress = self
            .jlpt_progress
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        // knowledge_set is the only field that switches wire format
        // (plain JSON -> deflated). Its decode is recovering: a corrupt or
        // legacy value resolves to an empty KnowledgeSet so the existing
        // self-heal (merge no-op -> local overwrites remote) is preserved.
        let knowledge_set: KnowledgeSet = self
            .knowledge_set
            .as_deref()
            .map(knowledge_set_codec::decode)
            .unwrap_or_default();

        let imported_sets: HashSet<String> = self
            .imported_sets
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        User::from_row(
            ulid,
            self.email.clone(),
            self.username.clone(),
            jlpt_progress,
            NativeLanguage::from(self.native_language),
            self.telegram_user_id.map(|id| id as u64),
            knowledge_set,
            self.updated_at,
            imported_sets,
            match self.daily_load {
                Some(val) => DailyLoad::from(val),
                None => {
                    tracing::warn!(
                        "User daily_load is None, using default (Medium). DB migration may be needed."
                    );
                    DailyLoad::default()
                },
            },
            self.known_vocab_hash.unwrap_or(0) as u32,
        )
    }
}

pub(crate) fn user_to_json(
    user: &User,
    trailbase_id: &str,
) -> Result<serde_json::Value, OrigaError> {
    let jlpt_progress_json =
        serde_json::to_string(user.jlpt_progress()).map_err(|e| OrigaError::RepositoryError {
            reason: format!("jlpt_progress encode failed: {e}"),
        })?;
    let knowledge_set_wire = knowledge_set_codec::encode(user.knowledge_set())?;
    let imported_sets_json =
        serde_json::to_string(user.imported_sets()).map_err(|e| OrigaError::RepositoryError {
            reason: format!("imported_sets encode failed: {e}"),
        })?;

    Ok(serde_json::json!({
        "trailbase_id": trailbase_id,
        "username": user.username(),
        "email": user.email(),
        "native_language": i32::from(*user.native_language()),
        "current_japanese_level": i32::from(user.current_japanese_level()),
        "jlpt_progress": jlpt_progress_json,
        "telegram_user_id": user.telegram_user_id().copied().map(|id| id as i64),
        "knowledge_set": knowledge_set_wire,
        "updated_at": user.updated_at().to_rfc3339(),
        "imported_sets": imported_sets_json,
        "daily_load": i32::from(*user.daily_load()),
        "known_vocab_hash": user.known_vocab_hash() as i32,
    }))
}

impl UserRepository for TrailBaseUserRepository {
    async fn get_current_user(&self) -> Result<Option<User>, OrigaError> {
        self.find_current()
            .await
            .map(|opt| opt.map(|(user, _)| user))
    }

    async fn save(&self, user: &User) -> Result<(), OrigaError> {
        let session = self.require_session()?;

        // Fast path: the record id discovered by a previous sync or create
        // is cached in the session, so a save costs one PATCH — no lookup
        // fetch (ADR-045 1c).
        if let Some(record_id) = session.record_id {
            return self.save_with_record_id(record_id, user).await;
        }

        match self.find_current_raw().await? {
            // The session's record id was missing (legacy sessions predate
            // the cache): look the row up without decoding it and update.
            Some(raw) => self.save_with_record_id(raw.record_id, user).await,
            None => self.create(user).await.map(|_| ()),
        }
    }

    async fn delete(&self, _user_id: Ulid) -> Result<(), OrigaError> {
        let _session = self.require_session()?;

        let api = self.client.records(&self.table_name);

        if let Some(raw) = self.find_current_raw().await? {
            api.delete(&raw.record_id.to_string())
                .await
                .map_err(map_auth_error)?;
        }

        Ok(())
    }
}

impl RemoteUserSource for TrailBaseUserRepository {
    async fn find_current_raw(&self) -> Result<Option<RemoteRow>, OrigaError> {
        TrailBaseUserRepository::find_current_raw(self).await
    }

    async fn save_with_record_id(&self, record_id: i64, user: &User) -> Result<(), OrigaError> {
        let session = self.require_session()?;

        let api = self.client.records(&self.table_name);
        let body = user_to_json(user, &session.trailbase_id)?;

        match api.update(&record_id.to_string(), &body).await {
            Ok(()) => {
                // Self-heal legacy sessions: the id is now proven valid, so
                // later saves skip the lookup fetch entirely.
                if session.record_id != Some(record_id) {
                    let updated = TrailBaseSession {
                        record_id: Some(record_id),
                        ..session
                    };
                    set_session_async(&updated)
                        .await
                        .map_err(|e| OrigaError::RepositoryError {
                            reason: format!("Failed to update session: {e}"),
                        })?;
                }
                Ok(())
            },
            // A failed update re-resolves the row: a live record (whatever
            // its id) retries the update and PROPAGATES on a second
            // failure — creating a duplicate row on a live record is never
            // right; only a genuinely missing row falls through to create
            // (ADR-045).
            Err(update_error) => {
                tracing::warn!(
                    "Record update by id {record_id} failed ({update_error:?}); re-resolving"
                );
                match self.find_current_raw().await? {
                    Some(raw) => {
                        api.update(&raw.record_id.to_string(), &body)
                            .await
                            .map_err(map_auth_error)?;
                        if session.record_id != Some(raw.record_id) {
                            let updated = TrailBaseSession {
                                record_id: Some(raw.record_id),
                                ..session
                            };
                            set_session_async(&updated).await.map_err(|e| {
                                OrigaError::RepositoryError {
                                    reason: format!("Failed to update session: {e}"),
                                }
                            })?;
                        }
                        Ok(())
                    },
                    None => self.create(user).await.map(|_| ()),
                }
            },
        }
    }

    async fn create(&self, user: &User) -> Result<i64, OrigaError> {
        let session = self.require_session()?;

        let api = self.client.records(&self.table_name);
        let body = user_to_json(user, &session.trailbase_id)?;

        let created_id = api.create(&body).await.map_err(map_auth_error)?;
        let record_id: i64 = created_id
            .parse()
            .map_err(|_| OrigaError::RepositoryError {
                reason: "Invalid record ID returned from create".to_string(),
            })?;

        let updated_session = TrailBaseSession {
            record_id: Some(record_id),
            ..session
        };
        set_session_async(&updated_session)
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to update session: {e}"),
            })?;

        Ok(record_id)
    }
}
