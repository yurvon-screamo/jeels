use crate::application::SrsService;
use crate::application::srs_service::{NextReview, RateMode};
use crate::domain::error::KeikakuError;
use crate::domain::review::{MemoryHistory, MemoryState};
use crate::domain::value_objects::{Difficulty, Rating, Stability};
use chrono::{Duration, Utc};
use rs_fsrs::{Card as FsrsCard, FSRS, Parameters, Rating as FsrsRating, State as FsrsState};

pub struct FsrsSrsService {
    short_term_fsrs: FSRS,
    long_term_fsrs: FSRS,
}

impl FsrsSrsService {
    pub fn new() -> Result<Self, KeikakuError> {
        let mut short_term_parameters = Parameters::default();
        short_term_parameters.request_retention = 0.95;
        short_term_parameters.enable_fuzz = true;
        short_term_parameters.enable_short_term = false;

        let mut long_term_parameters = Parameters::default();
        long_term_parameters.request_retention = 0.90;
        long_term_parameters.enable_fuzz = true;
        long_term_parameters.enable_short_term = true;

        Ok(Self {
            long_term_fsrs: FSRS::new(long_term_parameters),
            short_term_fsrs: FSRS::new(short_term_parameters),
        })
    }
}

impl SrsService for FsrsSrsService {
    async fn rate(
        &self,
        mode: RateMode,
        rating: Rating,
        memory_history: &MemoryHistory,
    ) -> Result<NextReview, KeikakuError> {
        let now = Utc::now();
        let card = if let Some(memory_state) = memory_history.memory_state() {
            let last_review_date = memory_history
                .reviews()
                .back()
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

            let reps = memory_history.reviews().len() as i32;
            let lapses = memory_history
                .reviews()
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

        let scheduling_info = match mode {
            RateMode::Fixation => self.short_term_fsrs.next(card, now, fsrs_rating),
            RateMode::Standard => self.long_term_fsrs.next(card, now, fsrs_rating),
        };

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
