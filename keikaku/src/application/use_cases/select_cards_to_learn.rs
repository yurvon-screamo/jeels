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

        let learn_settings = user.settings().learn();

        let study_session_items = if learn_settings.low_stability_mode() {
            user.start_low_stability_cards_session(learn_settings.limit())
        } else {
            user.start_study_session(learn_settings.force_new_cards(), learn_settings.limit())
        };

        Ok(study_session_items)
    }
}
