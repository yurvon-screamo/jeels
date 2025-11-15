use crate::application::user_repository::UserRepository;
use crate::domain::{JeersError, User};
use crate::settings::Settings;
use surrealdb::engine::local::RocksDb;
use surrealdb::Surreal;
use ulid::Ulid;

pub struct SurrealUserRepository {
    database: Surreal<surrealdb::engine::local::Db>,
}

impl SurrealUserRepository {
    pub async fn new(settings: &Settings) -> Result<Self, JeersError> {
        let db = Surreal::new::<RocksDb>(settings.database.path.clone())
            .await
            .map_err(|e| JeersError::RepositoryError {
                reason: format!("Failed to initialize database: {}", e),
            })?;

        db.use_ns(&settings.database.namespace)
            .use_db(&settings.database.database)
            .await
            .map_err(|e| JeersError::RepositoryError {
                reason: format!("Failed to select namespace/database: {}", e),
            })?;

        Ok(Self { database: db })
    }

    fn user_id_to_resource(&self, user_id: Ulid) -> (&str, String) {
        ("user", user_id.to_string())
    }
}

#[async_trait::async_trait]
impl UserRepository for SurrealUserRepository {
    async fn find_by_id(&self, user_id: Ulid) -> Result<Option<User>, JeersError> {
        let (table, id) = self.user_id_to_resource(user_id);

        let user: Option<User> = self
            .database
            .select((table, id.clone()))
            .await
            .map_err(|e| JeersError::RepositoryError {
                reason: format!("Failed to find user by id: {}", e),
            })?;

        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, JeersError> {
        let username_string = username.to_string();
        let mut result = self
            .database
            .query("SELECT * FROM user WHERE username = $username LIMIT 1")
            .bind(("username", username_string))
            .await
            .map_err(|e| JeersError::RepositoryError {
                reason: format!("Failed to find user by username: {}", e),
            })?;

        let users: Vec<User> = result.take(0).map_err(|e| JeersError::RepositoryError {
            reason: format!("Failed to deserialize user: {}", e),
        })?;

        Ok(users.into_iter().next())
    }

    async fn save(&self, user: &User) -> Result<(), JeersError> {
        let (table, id) = self.user_id_to_resource(user.id());

        let _: Option<User> = self
            .database
            .upsert((table, id))
            .content(user.clone())
            .await
            .map_err(|e| JeersError::RepositoryError {
                reason: format!("Failed to save user: {}", e),
            })?;

        Ok(())
    }

    async fn delete(&self, user_id: Ulid) -> Result<(), JeersError> {
        let (table, id) = self.user_id_to_resource(user_id);

        let _: Option<User> =
            self.database
                .delete((table, id))
                .await
                .map_err(|e| JeersError::RepositoryError {
                    reason: format!("Failed to delete user: {}", e),
                })?;

        Ok(())
    }
}
