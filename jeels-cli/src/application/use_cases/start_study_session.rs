use crate::application::user_repository::UserRepository;
use crate::domain::Card;
use crate::domain::error::JeersError;
use ulid::Ulid;

#[derive(Clone)]
pub struct StartStudySessionUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> StartStudySessionUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Ulid) -> Result<Vec<Card>, JeersError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let cards = user.start_study_session();
        Ok(cards)
    }
}
