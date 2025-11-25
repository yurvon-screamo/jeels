use crate::domain::error::JeersError;
use crate::domain::review::Review;
use crate::domain::value_objects::{MemoryState, Rating, Stability};
use chrono::Duration;

pub struct NextReview {
    pub interval: Duration,
    pub stability: Stability,
    pub memory_state: MemoryState,
}

pub trait SrsService: Send + Sync {
    fn calculate_next_review(
        &self,
        rating: Rating,
        previous_state: Option<MemoryState>,
        reviews: &[Review],
    ) -> impl Future<Output = Result<NextReview, JeersError>> + Send;
}
