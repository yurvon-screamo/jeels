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

    pub async fn execute(
        &self,
        user_id: Ulid,
        force_new_cards: bool,
        low_stability_cards: bool,
        limit: Option<usize>,
    ) -> Result<Vec<StudySessionItem>, JeersError> {
        if force_new_cards && low_stability_cards {
            return Err(JeersError::InvalidValues {
                reason: "Force new cards and low stability cards cannot be used together"
                    .to_string(),
            });
        }

        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let study_session_items = if low_stability_cards {
            user.start_low_stability_cards_session(limit)
        } else {
            user.start_study_session(force_new_cards, limit)
        };

        Ok(study_session_items)
    }
}
