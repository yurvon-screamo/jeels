use crate::application::user_repository::UserRepository;
use crate::domain::{JeersError, User};
use crate::settings::Settings;
use polodb_core::bson::doc;
use polodb_core::{CollectionT, Database};
use std::sync::{Arc, Mutex};
use ulid::Ulid;

pub struct PoloDbUserRepository {
    db: Arc<Mutex<Database>>,
}

impl PoloDbUserRepository {
    pub async fn new(settings: &Settings) -> Result<Self, JeersError> {
        std::fs::create_dir_all(&settings.database.path).map_err(|e| {
            JeersError::RepositoryError {
                reason: format!("Failed to create database directory: {}", e),
            }
        })?;

        let db_path = settings.database.path.to_string_lossy().to_string();

        let db = tokio::task::spawn_blocking(move || {
            Database::open_path(&db_path).map_err(|e| JeersError::RepositoryError {
                reason: format!("Failed to open database: {}", e),
            })
        })
        .await
        .map_err(|e| JeersError::RepositoryError {
            reason: format!("Failed to spawn database task: {}", e),
        })??;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }
}

#[async_trait::async_trait]
impl UserRepository for PoloDbUserRepository {
    async fn find_by_id(&self, user_id: Ulid) -> Result<Option<User>, JeersError> {
        let db = self.db.clone();
        let user_id_str = user_id.to_string();

        tokio::task::spawn_blocking(move || {
            let db = db.lock().unwrap();
            let collection = db.collection::<User>("users");

            let filter = doc! {
                "id": &user_id_str,
            };

            collection
                .find_one(filter)
                .map_err(|e| JeersError::RepositoryError {
                    reason: format!("Failed to find user by id: {}", e),
                })
        })
        .await
        .map_err(|e| JeersError::RepositoryError {
            reason: format!("Failed to spawn find task: {}", e),
        })?
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, JeersError> {
        let db = self.db.clone();
        let username = username.to_string();

        tokio::task::spawn_blocking(move || {
            let db = db.lock().unwrap();
            let collection = db.collection::<User>("users");

            let filter = doc! {
                "username": &username,
            };

            collection
                .find_one(filter)
                .map_err(|e| JeersError::RepositoryError {
                    reason: format!("Failed to find user by username: {}", e),
                })
        })
        .await
        .map_err(|e| JeersError::RepositoryError {
            reason: format!("Failed to spawn find task: {}", e),
        })?
    }

    async fn save(&self, user: &User) -> Result<(), JeersError> {
        let db = self.db.clone();
        let user = user.clone();

        tokio::task::spawn_blocking(move || {
            let db = db.lock().unwrap();
            let collection = db.collection::<User>("users");

            let filter = doc! {
                "id": user.id().to_string(),
            };

            let exists =
                collection
                    .find_one(filter.clone())
                    .map_err(|e| JeersError::RepositoryError {
                        reason: format!("Failed to check if user exists: {}", e),
                    })?;

            if exists.is_some() {
                collection
                    .update_many(
                        filter,
                        doc! {
                            "$set": polodb_core::bson::to_document(&user).map_err(|e| {
                                JeersError::RepositoryError {
                                    reason: format!("Failed to serialize user: {}", e),
                                }
                            })?,
                        },
                    )
                    .map_err(|e| JeersError::RepositoryError {
                        reason: format!("Failed to update user: {}", e),
                    })?;
            } else {
                collection
                    .insert_one(user)
                    .map_err(|e| JeersError::RepositoryError {
                        reason: format!("Failed to insert user: {}", e),
                    })?;
            }

            Ok(())
        })
        .await
        .map_err(|e| JeersError::RepositoryError {
            reason: format!("Failed to spawn save task: {}", e),
        })?
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), JeersError> {
        let db = self.db.clone();
        let user_id_str = user_id.to_string();

        tokio::task::spawn_blocking(move || {
            let db = db.lock().unwrap();
            let collection = db.collection::<User>("users");

            let filter = doc! {
                "id": &user_id_str,
            };

            collection
                .delete_one(filter)
                .map_err(|e| JeersError::RepositoryError {
                    reason: format!("Failed to delete user: {}", e),
                })?;

            Ok(())
        })
        .await
        .map_err(|e| JeersError::RepositoryError {
            reason: format!("Failed to spawn delete task: {}", e),
        })?
    }
}
