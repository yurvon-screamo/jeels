use crate::application::SrsService;
use crate::domain::error::JeersError;
use crate::domain::review::Review;
use crate::domain::value_objects::{Interval, MemoryState, Rating, Stability};
use chrono::{DateTime, Utc};
use fsrs::{FSRS, FSRSItem, FSRSReview};
use std::sync::Arc;
use tokio::sync::Mutex;

// Default FSRS parameters (from fsrs crate)
// TODO: KILME
const DEFAULT_PARAMETERS: [f32; 21] = [
    0.212, 1.2931, 2.3065, 8.2956, 6.4133, 0.8334, 3.0194, 0.001, 1.8722, 0.1666, 0.796, 1.4835,
    0.0614, 0.2629, 1.6483, 0.6014, 1.8729, 0.5425, 0.0912, 0.0658, 0.1542,
];

pub struct FsrsSrsService {
    fsrs: Arc<Mutex<FSRS>>,
    desired_retention: f64,
}

impl FsrsSrsService {
    pub fn new() -> Result<Self, JeersError> {
        Ok(Self {
            fsrs: Arc::new(Mutex::new(FSRS::new(Some(&DEFAULT_PARAMETERS)).map_err(
                |e| JeersError::SrsCalculationFailed {
                    reason: format!("Failed to create FSRS: {:?}", e),
                },
            )?)),
            desired_retention: 0.9,
        })
    }

    pub fn with_retention(desired_retention: f64) -> Result<Self, JeersError> {
        Ok(Self {
            fsrs: Arc::new(Mutex::new(FSRS::new(Some(&DEFAULT_PARAMETERS)).map_err(
                |e| JeersError::SrsCalculationFailed {
                    reason: format!("Failed to create FSRS: {:?}", e),
                },
            )?)),
            desired_retention,
        })
    }

    fn build_fsrs_item(reviews: &[Review]) -> FSRSItem {
        let mut fsrs_reviews = Vec::new();
        let mut last_timestamp: Option<DateTime<Utc>> = None;

        for review in reviews {
            let delta_t = if let Some(last) = last_timestamp {
                let duration = review.timestamp().signed_duration_since(last);
                duration.num_days().max(0) as u32
            } else {
                0
            };

            fsrs_reviews.push(FSRSReview {
                // FSRS rating mapping: 1=Again, 2=Hard, 3=Good, 4=Easy
                rating: match review.rating() {
                    Rating::Again => 1,
                    Rating::Hard => 2,
                    Rating::Good => 3,
                    Rating::Easy => 4,
                },
                delta_t,
            });

            last_timestamp = Some(review.timestamp());
        }

        FSRSItem {
            reviews: fsrs_reviews,
        }
    }
}

impl FsrsSrsService {
    fn to_fsrs_memory_state(state: MemoryState) -> fsrs::MemoryState {
        fsrs::MemoryState {
            stability: state.stability().value() as f32,
            difficulty: state.difficulty() as f32,
        }
    }

    fn from_fsrs_memory_state(state: fsrs::MemoryState) -> Result<MemoryState, JeersError> {
        let stability = Stability::new(state.stability as f64)?;
        MemoryState::new(stability, state.difficulty as f64)
    }
}

impl SrsService for FsrsSrsService {
    async fn calculate_next_review(
        &self,
        rating: Rating,
        previous_state: Option<MemoryState>,
        reviews: &[Review],
        elapsed_days: u32,
    ) -> Result<(Interval, Stability, MemoryState), JeersError> {
        let fsrs = self.fsrs.lock().await;

        let current_memory_state = if !reviews.is_empty() {
            let item = Self::build_fsrs_item(reviews);
            Some(
                fsrs.memory_state(item, None)
                    .map_err(|e| JeersError::SrsCalculationFailed {
                        reason: format!("Failed to calculate memory state from reviews: {:?}", e),
                    })?,
            )
        } else if let Some(state) = previous_state {
            Some(Self::to_fsrs_memory_state(state))
        } else {
            None
        };

        let next_states = fsrs
            .next_states(
                current_memory_state,
                self.desired_retention as f32,
                elapsed_days,
            )
            .map_err(|e| JeersError::SrsCalculationFailed {
                reason: format!("Failed to calculate next states: {:?}", e),
            })?;

        let review_state = match rating {
            Rating::Again => next_states.again,
            Rating::Hard => next_states.hard,
            Rating::Good => next_states.good,
            Rating::Easy => next_states.easy,
        };

        let days = review_state.interval.floor() as u32;
        let interval = Interval::new(days);

        let domain_memory_state = Self::from_fsrs_memory_state(review_state.memory)?;
        let stability = domain_memory_state.stability();

        Ok((interval, stability, domain_memory_state))
    }
}
