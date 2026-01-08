use crate::domain::{KeikakuError, User};
use ulid::Ulid;

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, user_id: Ulid) -> Result<Option<User>, KeikakuError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, KeikakuError>;
    async fn save(&self, user: &User) -> Result<(), KeikakuError>;
    async fn delete(&self, user_id: Ulid) -> Result<(), KeikakuError>;
}
