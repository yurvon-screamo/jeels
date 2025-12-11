use crate::application::user_repository::UserRepository;
use crate::domain::error::JeersError;
use ulid::Ulid;

#[derive(Clone)]
pub struct CompleteLessonUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CompleteLessonUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Ulid) -> Result<(), JeersError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        user.update_daily_history();

        self.repository.save(&user).await?;

        Ok(())
    }
}
