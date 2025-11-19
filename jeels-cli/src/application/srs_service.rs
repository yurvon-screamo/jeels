use crate::domain::error::JeersError;
use crate::domain::review::Review;
use crate::domain::value_objects::{Interval, MemoryState, Rating, Stability};

pub struct NextReview {
    pub interval: Interval,
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
