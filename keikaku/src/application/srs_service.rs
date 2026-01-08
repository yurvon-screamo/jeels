use crate::domain::error::KeikakuError;
use crate::domain::review::MemoryHistory;
use crate::domain::review::MemoryState;
use crate::domain::value_objects::Rating;
use chrono::Duration;

pub struct NextReview {
    pub interval: Duration,
    pub memory_state: MemoryState,
}

pub enum RateMode {
    Standard,
    Fixation,
}

pub trait SrsService: Send + Sync {
    fn rate(
        &self,
        mode: RateMode,
        rating: Rating,
        memory_history: &MemoryHistory,
    ) -> impl Future<Output = Result<NextReview, KeikakuError>> + Send;
}
