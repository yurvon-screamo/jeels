use crate::application::user_repository::UserRepository;
use crate::domain::error::KeikakuError;
use chrono::Duration;
use ulid::Ulid;

#[derive(Clone)]
pub struct CompleteLessonUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CompleteLessonUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        lesson_duration: Duration,
    ) -> Result<(), KeikakuError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(KeikakuError::UserNotFound { user_id })?;

        user.add_lesson_duration(lesson_duration);

        self.repository.save(&user).await?;

        Ok(())
    }
}
