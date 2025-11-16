use crate::application::SrsService;
use crate::application::user_repository::UserRepository;
use crate::domain::error::JeersError;
use crate::domain::review::Review;
use crate::domain::value_objects::Rating;
use chrono::Utc;
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

        let card = user
            .get_card(card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        let reviews: Vec<Review> = card.reviews().iter().cloned().collect();
        let previous_memory_state = card.memory_state();
        let last_review_date = card.last_review_date().unwrap_or(card.next_review_date());
        let elapsed_days = if last_review_date <= Utc::now() {
            let duration = Utc::now().signed_duration_since(last_review_date);
            duration.num_days().max(0) as u32
        } else {
            0
        };

        let (interval, new_stability, memory_state) = self.srs_service.calculate_next_review(
            rating,
            previous_memory_state,
            &reviews,
            elapsed_days,
        )?;

        let next_review_date = Utc::now() + chrono::Duration::days(interval.days() as i64);

        user.rate_card(card_id, rating, interval)?;
        user.schedule_next_review(card_id, next_review_date, new_stability, memory_state)?;
        self.repository.save(&user).await?;

        Ok(())
    }
}
