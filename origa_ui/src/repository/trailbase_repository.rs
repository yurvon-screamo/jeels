use super::session::{TrailBaseSession, get_session, set_session_async};
use super::trailbase_client::{AuthError, TrailBaseClient};
use super::trailbase_id::uuid_to_ulid;
use chrono::{DateTime, Utc};
use origa::domain::{DailyLoad, KnowledgeSet, NativeLanguage, OrigaError, User};
use origa::traits::UserRepository;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use ulid::Ulid;

use super::knowledge_set_codec;

#[cfg(test)]
#[path = "trailbase_repository_tests.rs"]
mod tests;

#[derive(Clone)]
pub struct TrailBaseUserRepository {
    client: TrailBaseClient,
    table_name: String,
    user_cache: Arc<RwLock<HashMap<String, User>>>,
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
            table_name: "user".to_string(),
            user_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn find_current(&self) -> Result<Option<(User, i64)>, OrigaError> {
        let session = get_session().ok_or_else(|| OrigaError::RepositoryError {
            reason: "Not authenticated".to_string(),
        })?;

        if session.email.is_empty() {
            return Err(OrigaError::RepositoryError {
                reason: "Email not found in session. Please re-login.".to_string(),
            });
        }

        let api = self.client.records(&self.table_name);

        let records: Vec<UserRow> = api
            .list_filtered("email", &session.email)
            .await
            .map_err(map_auth_error)?;

        if let Some(row) = records.into_iter().next() {
            let record_id = row.id.ok_or_else(|| OrigaError::RepositoryError {
                reason: "Record ID missing from database row".to_string(),
            })?;
            let user = row.to_user();

            // A remote row whose trailbase_id does not decode to a real ULID is
            // corrupt for sync purposes: merging it would poison the local id
            // with nil (the very bug this code path guards against). Surface it
            // as an error instead of letting merge propagate the nil identity.
            if user.id() == Ulid::nil() {
                return Err(OrigaError::RepositoryError {
                    reason: "Remote user trailbase_id did not decode to a valid ULID; refusing to sync a nil identity".to_string(),
                });
            }

            if let Ok(mut cache) = self.user_cache.write() {
                cache.insert(session.email.clone(), user.clone());
            }

            return Ok(Some((user, record_id)));
        }

        Ok(None)
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

fn user_to_json(user: &User, trailbase_id: &str) -> Result<serde_json::Value, OrigaError> {
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
        let session = get_session().ok_or_else(|| OrigaError::RepositoryError {
            reason: "Not authenticated".to_string(),
        })?;

        if session.email.is_empty() {
            return Err(OrigaError::RepositoryError {
                reason: "Email not found in session. Please re-login.".to_string(),
            });
        }

        let api = self.client.records(&self.table_name);
        let body = user_to_json(user, &session.trailbase_id)?;

        if let Some((_, record_id)) = self.find_current().await? {
            api.update(&record_id.to_string(), &body)
                .await
                .map_err(map_auth_error)?;
        } else {
            let created_id = api.create(&body).await.map_err(map_auth_error)?;
            let record_id: i64 = created_id
                .parse()
                .map_err(|_| OrigaError::RepositoryError {
                    reason: "Invalid record ID returned from create".to_string(),
                })?;

            let updated_session = TrailBaseSession {
                record_id: Some(record_id),
                ..session.clone()
            };
            set_session_async(&updated_session)
                .await
                .map_err(|e| OrigaError::RepositoryError {
                    reason: format!("Failed to update session: {}", e),
                })?;
        }

        if let Ok(mut cache) = self.user_cache.write() {
            cache.insert(user.email().to_string(), user.clone());
        }

        Ok(())
    }

    async fn delete(&self, _user_id: Ulid) -> Result<(), OrigaError> {
        let _session = get_session().ok_or_else(|| OrigaError::RepositoryError {
            reason: "Not authenticated".to_string(),
        })?;

        let api = self.client.records(&self.table_name);

        if let Some((_, record_id)) = self.find_current().await? {
            api.delete(&record_id.to_string())
                .await
                .map_err(map_auth_error)?;
        }

        if let Ok(mut cache) = self.user_cache.write() {
            cache.clear();
        }

        Ok(())
    }
}
