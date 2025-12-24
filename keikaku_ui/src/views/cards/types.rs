use chrono::{DateTime, Duration, Utc};
use keikaku::domain::value_objects::Rating;

#[derive(Clone, PartialEq)]
pub struct ReviewInfo {
    pub timestamp: DateTime<Utc>,
    pub rating: Rating,
    pub interval: Duration,
}

#[derive(Clone, PartialEq)]
pub enum FilterStatus {
    All,
    New,
    LowStability,
    HighDifficulty,
    InProgress,
    Learned,
}

#[derive(Clone, PartialEq)]
pub enum SortBy {
    Date,
    Question,
    Answer,
    Difficulty,
    Stability,
}

#[derive(Clone, PartialEq)]
pub struct UiCard {
    pub id: String,
    pub question: String,
    pub answer: String,
    pub examples: Vec<(String, String)>, // (text, translation)
    pub difficulty: Option<f64>,
    pub stability: Option<f64>,
    pub next_review: String,
    pub due: bool,
    pub is_new: bool,
    pub is_in_progress: bool,
    pub is_learned: bool,
    pub is_low_stability: bool,
    pub is_high_difficulty: bool,
    pub reviews: Vec<ReviewInfo>,
}
