use crate::application::SrsService;
use crate::application::srs_service::NextReview;
use crate::domain::error::JeersError;
use crate::domain::review::Review;
use crate::domain::value_objects::{Difficulty, MemoryState, Rating, Stability};
use chrono::{Duration, Utc};
use fsrs::FSRS;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct FsrsSrsService {
    fsrs: Arc<Mutex<FSRS>>,
    desired_retention: f64,
}

impl FsrsSrsService {
    pub fn new() -> Result<Self, JeersError> {
        Ok(Self {
            fsrs: Arc::new(Mutex::new(FSRS::new(Some(&[])).map_err(|e| {
                JeersError::SrsCalculationFailed {
                    reason: format!("Failed to create FSRS: {:?}", e),
                }
            })?)),
            desired_retention: 0.9,
        })
    }
}

impl SrsService for FsrsSrsService {
    async fn calculate_next_review(
        &self,
        rating: Rating,
        previous_memory_state: Option<MemoryState>,
        reviews: &[Review],
    ) -> Result<NextReview, JeersError> {
        let elapsed_days = if let Some(last_review) = reviews.last() {
            (Utc::now() - last_review.timestamp()).num_days().max(0) as u32
        } else {
            0
        };

        let fsrs_memory_state =
            previous_memory_state.map(|previous_memory_state| fsrs::MemoryState {
                stability: previous_memory_state.stability().value() as f32,
                difficulty: previous_memory_state.difficulty().value() as f32,
            });

        let fsrs = self.fsrs.lock().await;

        let next_states = fsrs
            .next_states(
                fsrs_memory_state,
                self.desired_retention as f32,
                elapsed_days,
            )
            .map_err(|e| JeersError::SrsCalculationFailed {
                reason: format!("Failed to calculate next states: {:?}", e),
            })?;

        let next_state = match rating {
            Rating::Again => &next_states.again,
            Rating::Hard => &next_states.hard,
            Rating::Good => &next_states.good,
            Rating::Easy => &next_states.easy,
        };

        let interval_days = next_state.interval.round() as i64;
        let stability = Stability::new(next_state.memory.stability as f64)?;
        let difficulty = Difficulty::new(next_state.memory.difficulty as f64)?;
        let memory_state = MemoryState::new(stability, difficulty);

        let interval = if interval_days == 0 && rating != Rating::Again {
            Duration::hours(1)
        } else {
            Duration::days(interval_days)
        };

        Ok(NextReview {
            interval,
            memory_state,
        })
    }
}
