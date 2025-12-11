use crate::application::SrsService;
use crate::application::srs_service::NextReview;
use crate::application::user_repository::UserRepository;
use crate::domain::error::JeersError;
use crate::domain::review::Review;
use crate::domain::value_objects::Rating;
use ulid::Ulid;

#[derive(Clone, Copy)]
pub struct RateCardUseCase<'a, R: UserRepository, S: SrsService> {
    repository: &'a R,
    srs_service: &'a S,
}

impl<'a, R: UserRepository, S: SrsService> RateCardUseCase<'a, R, S> {
    pub fn new(repository: &'a R, srs_service: &'a S) -> Self {
        Self {
            repository,
            srs_service,
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        card_id: Ulid,
        rating: Rating,
    ) -> Result<(), JeersError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let (reviews, previous_memory_state) = if let Some(card) = user.get_card(card_id) {
            (
                card.memory()
                    .reviews()
                    .iter()
                    .cloned()
                    .collect::<Vec<Review>>(),
                card.memory().memory_state(),
            )
        } else if let Some(card) = user.get_kanji_card(card_id) {
            (
                card.memory_history()
                    .reviews()
                    .iter()
                    .cloned()
                    .collect::<Vec<Review>>(),
                card.memory_history().memory_state(),
            )
        } else {
            return Err(JeersError::CardNotFound { card_id });
        };

        let NextReview {
            interval,
            memory_state,
        } = self
            .srs_service
            .calculate_next_review(rating, previous_memory_state, &reviews)
            .await?;

        user.rate_card(card_id, rating, interval, memory_state)?;

        self.repository.save(&user).await?;

        Ok(())
    }
}
