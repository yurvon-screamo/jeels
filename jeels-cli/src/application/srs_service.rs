use crate::domain::error::JeersError;
use crate::domain::review::Review;
use crate::domain::value_objects::{Interval, MemoryState, Rating, Stability};

pub trait SrsService: Send + Sync {
    fn calculate_next_review(
        &self,
        rating: Rating,
        previous_state: Option<MemoryState>,
        reviews: &[Review],
        elapsed_days: u32,
    ) -> impl Future<Output = Result<(Interval, Stability, MemoryState), JeersError>> + Send;
}
