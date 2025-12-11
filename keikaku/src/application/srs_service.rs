use crate::domain::error::JeersError;
use crate::domain::review::MemoryState;
use crate::domain::review::Review;
use crate::domain::value_objects::Rating;
use chrono::Duration;

pub struct NextReview {
    pub interval: Duration,
    pub memory_state: MemoryState,
}

pub trait SrsService: Send + Sync {
    fn calculate_next_review(
        &self,
        rating: Rating,
        previous_memory_state: Option<&MemoryState>,
        reviews: &[Review],
    ) -> impl Future<Output = Result<NextReview, JeersError>> + Send;
}
