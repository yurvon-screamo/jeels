use crate::domain::{JeersError, User};
use ulid::Ulid;

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, user_id: Ulid) -> Result<Option<User>, JeersError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, JeersError>;
    async fn save(&self, user: &User) -> Result<(), JeersError>;
    async fn delete(&self, user_id: Ulid) -> Result<(), JeersError>;
}
