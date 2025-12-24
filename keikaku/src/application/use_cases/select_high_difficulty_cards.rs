use crate::application::user_repository::UserRepository;
use crate::domain::error::JeersError;
use crate::domain::study_session::StudySessionItem;
use ulid::Ulid;

#[derive(Clone)]
pub struct SelectHighDifficultyCardsUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> SelectHighDifficultyCardsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        limit: Option<usize>,
    ) -> Result<Vec<StudySessionItem>, JeersError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let study_session_items = user.start_high_difficulty_cards_session(limit);

        Ok(study_session_items)
    }
}
