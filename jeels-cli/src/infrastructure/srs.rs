use crate::application::SrsService;
use crate::domain::error::JeersError;
use crate::domain::review::Review;
use crate::domain::value_objects::{Interval, MemoryState, Rating, Stability};
use chrono::{DateTime, Utc};
use fsrs::{FSRSItem, FSRSReview, FSRS};

// Default FSRS parameters (from fsrs crate)
// TODO: KILME
const DEFAULT_PARAMETERS: [f32; 21] = [
    0.212, 1.2931, 2.3065, 8.2956, 6.4133, 0.8334, 3.0194, 0.001, 1.8722, 0.1666, 0.796, 1.4835,
    0.0614, 0.2629, 1.6483, 0.6014, 1.8729, 0.5425, 0.0912, 0.0658, 0.1542,
];

pub struct FsrsSrsService {
    fsrs: FSRS,
    desired_retention: f64,
}

impl FsrsSrsService {
    pub fn new() -> Result<Self, JeersError> {
        Ok(Self {
            fsrs: FSRS::new(Some(&DEFAULT_PARAMETERS)).map_err(|e| {
                JeersError::SrsCalculationFailed {
                    reason: format!("Failed to create FSRS: {:?}", e),
                }
            })?,
            desired_retention: 0.9,
        })
    }

    pub fn with_retention(desired_retention: f64) -> Result<Self, JeersError> {
        Ok(Self {
            fsrs: FSRS::new(Some(&DEFAULT_PARAMETERS)).map_err(|e| {
                JeersError::SrsCalculationFailed {
                    reason: format!("Failed to create FSRS: {:?}", e),
                }
            })?,
            desired_retention,
        })
    }

    fn rating_to_fsrs_rating(rating: Rating) -> u32 {
        match rating {
            Rating::Again => 1,
            Rating::Hard => 2,
            Rating::Good => 3,
            Rating::Easy => 4,
        }
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
                rating: Self::rating_to_fsrs_rating(review.rating()),
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
    fn calculate_next_review(
        &self,
        rating: Rating,
        previous_state: Option<MemoryState>,
        reviews: &[Review],
        elapsed_days: u32,
    ) -> Result<(Interval, Stability, MemoryState), JeersError> {
        let fsrs_rating = Self::rating_to_fsrs_rating(rating);

        // Determine current memory state:
        // 1. Use previous_state if available (most reliable)
        // 2. Otherwise, calculate from reviews if available
        // 3. Otherwise, use None for first review (no history)
        let current_memory_state = if let Some(state) = previous_state {
            Some(Self::to_fsrs_memory_state(state))
        } else if !reviews.is_empty() {
            let item = Self::build_fsrs_item(reviews);
            Some(self.fsrs.memory_state(item, None).map_err(|e| {
                JeersError::SrsCalculationFailed {
                    reason: format!("Failed to calculate memory state from reviews: {:?}", e),
                }
            })?)
        } else {
            None
        };

        let next_states = self
            .fsrs
            .next_states(
                current_memory_state,
                self.desired_retention as f32,
                elapsed_days,
            )
            .map_err(|e| JeersError::SrsCalculationFailed {
                reason: format!("Failed to calculate next states: {:?}", e),
            })?;

        let review_state = match fsrs_rating {
            1 => next_states.again,
            2 => next_states.hard,
            3 => next_states.good,
            4 => next_states.easy,
            _ => {
                return Err(JeersError::SrsCalculationFailed {
                    reason: format!("Invalid rating: {}", fsrs_rating),
                })
            }
        };

        let interval_days = review_state.interval.round().max(1.0) as u32;
        let interval = Interval::new(interval_days);

        let domain_memory_state = Self::from_fsrs_memory_state(review_state.memory)?;
        let stability = domain_memory_state.stability();

        Ok((interval, stability, domain_memory_state))
    }
}
