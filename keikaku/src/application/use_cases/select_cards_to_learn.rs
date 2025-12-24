use crate::application::user_repository::UserRepository;
use crate::domain::error::JeersError;
use crate::domain::study_session::StudySessionItem;
use ulid::Ulid;

#[derive(Clone)]
pub struct SelectCardsToLearnUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> SelectCardsToLearnUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Ulid) -> Result<Vec<StudySessionItem>, JeersError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let study_session_items = user.start_study_session();

        Ok(study_session_items)
    }
}
