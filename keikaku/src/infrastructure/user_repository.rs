use crate::application::user_repository::UserRepository;
use crate::domain::{JeersError, User};
use std::path::PathBuf;
use tokio::fs;
use ulid::Ulid;

pub struct FileSystemUserRepository {
    users_dir: PathBuf,
}

impl FileSystemUserRepository {
    pub async fn new(database_path: &str) -> Result<Self, JeersError> {
        let users_dir = PathBuf::from(database_path);

        fs::create_dir_all(&users_dir)
            .await
            .map_err(|e| JeersError::RepositoryError {
                reason: format!(
                    "Failed to create users directory {}: {}",
                    users_dir.display(),
                    e
                ),
            })?;

        Ok(Self { users_dir })
    }

    fn user_file_path(&self, user_id: Ulid) -> PathBuf {
        self.users_dir.join(format!("{}.json", user_id))
    }
}

#[async_trait::async_trait]
impl UserRepository for FileSystemUserRepository {
    async fn find_by_id(&self, user_id: Ulid) -> Result<Option<User>, JeersError> {
        let file_path = self.user_file_path(user_id);

        if !file_path.exists() {
            return Ok(None);
        }

        let content =
            fs::read_to_string(&file_path)
                .await
                .map_err(|e| JeersError::RepositoryError {
                    reason: format!("Failed to read user file: {}", e),
                })?;

        let user: User =
            serde_json::from_str(&content).map_err(|e| JeersError::RepositoryError {
                reason: format!("Failed to deserialize user: {}", e),
            })?;

        Ok(Some(user))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, JeersError> {
        let mut entries =
            fs::read_dir(&self.users_dir)
                .await
                .map_err(|e| JeersError::RepositoryError {
                    reason: format!("Failed to read users directory: {}", e),
                })?;

        while let Some(entry) =
            entries
                .next_entry()
                .await
                .map_err(|e| JeersError::RepositoryError {
                    reason: format!("Failed to read directory entry: {}", e),
                })?
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }

            let content =
                fs::read_to_string(&path)
                    .await
                    .map_err(|e| JeersError::RepositoryError {
                        reason: format!("Failed to read user file: {}", e),
                    })?;

            let user: User =
                serde_json::from_str(&content).map_err(|e| JeersError::RepositoryError {
                    reason: format!("Failed to deserialize user: {}", e),
                })?;

            if user.username() == username {
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    async fn save(&self, user: &User) -> Result<(), JeersError> {
        let file_path = self.user_file_path(user.id());
        let json = serde_json::to_string_pretty(user).map_err(|e| JeersError::RepositoryError {
            reason: format!("Failed to serialize user: {}", e),
        })?;

        fs::write(&file_path, json)
            .await
            .map_err(|e| JeersError::RepositoryError {
                reason: format!("Failed to write user file: {}", e),
            })?;

        Ok(())
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), JeersError> {
        let file_path = self.user_file_path(user_id);

        if file_path.exists() {
            fs::remove_file(&file_path)
                .await
                .map_err(|e| JeersError::RepositoryError {
                    reason: format!("Failed to delete user file: {}", e),
                })?;
        }

        Ok(())
    }
}
