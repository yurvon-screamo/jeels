use crate::application::SrsService;
use crate::application::srs_service::NextReview;
use crate::domain::error::JeersError;
use crate::domain::review::{MemoryState, Review};
use crate::domain::value_objects::{Difficulty, Rating, Stability};
use chrono::{Duration, Utc};
use rs_fsrs::{Card as FsrsCard, FSRS, Parameters, Rating as FsrsRating, State as FsrsState};

pub struct FsrsSrsService {
    fsrs: FSRS,
}

impl FsrsSrsService {
    pub fn new() -> Result<Self, JeersError> {
        let mut parameters = Parameters::default();
        parameters.request_retention = 0.95;
        parameters.enable_fuzz = true;
        parameters.enable_short_term = false;

        Ok(Self {
            fsrs: FSRS::new(parameters),
        })
    }
}

impl SrsService for FsrsSrsService {
    async fn calculate_next_review(
        &self,
        rating: Rating,
        previous_memory_state: Option<&MemoryState>,
        reviews: &[Review],
    ) -> Result<NextReview, JeersError> {
        let now = Utc::now();
        let card = if let Some(memory_state) = previous_memory_state {
            let last_review_date = reviews
                .last()
                .map(|review| review.timestamp())
                .unwrap_or(now);

            let elapsed_days = now
                .signed_duration_since(last_review_date)
                .num_days()
                .max(0);

            let scheduled_days = memory_state
                .next_review_date()
                .signed_duration_since(last_review_date)
                .num_days()
                .max(0);

            let reps = reviews.len() as i32;
            let lapses = reviews
                .iter()
                .filter(|review| matches!(review.rating(), Rating::Again))
                .count() as i32;

            FsrsCard {
                due: *memory_state.next_review_date(),
                stability: memory_state.stability().value(),
                difficulty: memory_state.difficulty().value(),
                elapsed_days,
                scheduled_days,
                reps,
                lapses,
                state: FsrsState::Review,
                last_review: last_review_date,
            }
        } else {
            FsrsCard::new()
        };

        let fsrs_rating = match rating {
            Rating::Again => FsrsRating::Again,
            Rating::Hard => FsrsRating::Hard,
            Rating::Good => FsrsRating::Good,
            Rating::Easy => FsrsRating::Easy,
        };

        let scheduling_info = self.fsrs.next(card, now, fsrs_rating);
        let next_review_date = scheduling_info.card.due;

        let interval = next_review_date.signed_duration_since(now);
        let interval = if interval < Duration::zero() {
            Duration::zero()
        } else {
            interval
        };

        let stability = Stability::new(scheduling_info.card.stability)?;
        let difficulty = Difficulty::new(scheduling_info.card.difficulty)?;
        let memory_state = MemoryState::new(stability, difficulty, next_review_date);

        Ok(NextReview {
            interval,
            memory_state,
        })
    }
}
