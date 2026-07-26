use core::fmt;

use crate::domain::OrigaError;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum CardState {
    New,
    Learning,
    #[default]
    Review,
    Relearning,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MemoryState {
    stability: Stability,
    difficulty: Difficulty,
    next_review_date: DateTime<Utc>,
    #[serde(default)]
    card_state: CardState,
}

impl MemoryState {
    pub fn new(
        stability: Stability,
        difficulty: Difficulty,
        next_review_date: DateTime<Utc>,
    ) -> Self {
        Self {
            stability,
            difficulty,
            next_review_date,
            card_state: CardState::Review,
        }
    }

    pub fn with_card_state(
        stability: Stability,
        difficulty: Difficulty,
        next_review_date: DateTime<Utc>,
        card_state: CardState,
    ) -> Self {
        Self {
            stability,
            difficulty,
            next_review_date,
            card_state,
        }
    }

    pub fn stability(&self) -> &Stability {
        &self.stability
    }

    pub fn difficulty(&self) -> &Difficulty {
        &self.difficulty
    }

    pub fn next_review_date(&self) -> &DateTime<Utc> {
        &self.next_review_date
    }

    pub fn card_state(&self) -> CardState {
        self.card_state
    }
}

impl fmt::Display for MemoryState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Stability: {}, Difficulty: {}, Next review date: {}",
            self.stability, self.difficulty, self.next_review_date
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ReviewLog {
    id: Ulid,
    rating: Rating,
    timestamp: DateTime<Utc>,
    interval: Duration,
}

impl ReviewLog {
    pub fn new(rating: Rating, interval: Duration) -> Self {
        Self {
            id: Ulid::new(),
            rating,
            timestamp: Utc::now(),
            interval,
        }
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn rating(&self) -> Rating {
        self.rating
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Stability {
    value: f64,
}

impl Stability {
    pub fn new(value: f64) -> Result<Self, OrigaError> {
        if value < 0.0 {
            return Err(OrigaError::InvalidStability {
                reason: "Stability cannot be negative".to_string(),
            });
        }
        Ok(Self { value })
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

impl fmt::Display for Stability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Difficulty {
    value: f64,
}

impl Difficulty {
    pub fn new(value: f64) -> Result<Self, OrigaError> {
        if value < 0.0 {
            return Err(OrigaError::InvalidDifficulty {
                reason: "Difficulty cannot be negative".to_string(),
            });
        }
        Ok(Self { value })
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Rating {
    Easy,
    Good,
    Hard,
    Again,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // MemoryState tests
    #[test]
    fn memory_state_new_stores_all_fields() {
        let stability = Stability::new(0.8).unwrap();
        let difficulty = Difficulty::new(0.3).unwrap();
        let next_review_date = Utc::now();

        let state = MemoryState::new(stability, difficulty, next_review_date);

        assert_eq!(state.stability().value(), 0.8);
        assert_eq!(state.difficulty().value(), 0.3);
        assert_eq!(*state.next_review_date(), next_review_date);
    }

    #[test]
    fn memory_state_display_includes_field_labels() {
        let stability = Stability::new(0.5).unwrap();
        let difficulty = Difficulty::new(0.5).unwrap();
        let next_review_date = Utc::now();
        let state = MemoryState::new(stability, difficulty, next_review_date);

        let display = format!("{}", state);
        assert!(display.contains("Stability"));
        assert!(display.contains("Difficulty"));
        assert!(display.contains("Next review date"));
    }

    // ReviewLog tests
    #[test]
    fn review_log_new_assigns_id_rating_and_interval() {
        let log = ReviewLog::new(Rating::Good, Duration::days(1));

        assert!(log.id() > Ulid::nil());
        assert_eq!(log.rating(), Rating::Good);
        assert_eq!(log.interval(), Duration::days(1));
    }

    #[test]
    fn review_log_preserves_all_rating_variants() {
        let logs = [
            ReviewLog::new(Rating::Easy, Duration::days(1)),
            ReviewLog::new(Rating::Good, Duration::days(1)),
            ReviewLog::new(Rating::Hard, Duration::days(1)),
            ReviewLog::new(Rating::Again, Duration::days(1)),
        ];

        assert_eq!(logs[0].rating(), Rating::Easy);
        assert_eq!(logs[1].rating(), Rating::Good);
        assert_eq!(logs[2].rating(), Rating::Hard);
        assert_eq!(logs[3].rating(), Rating::Again);
    }

    // Stability tests - validation
    #[test]
    fn stability_new_valid() {
        let stability = Stability::new(0.5).unwrap();
        assert_eq!(stability.value(), 0.5);
    }

    #[test]
    fn stability_new_zero() {
        let stability = Stability::new(0.0).unwrap();
        assert_eq!(stability.value(), 0.0);
    }

    #[test]
    fn stability_new_negative_fails() {
        let result = Stability::new(-0.1);
        assert!(result.is_err());
        assert!(matches!(result, Err(OrigaError::InvalidStability { .. })));
    }

    #[test]
    fn stability_display_formats_to_two_decimals() {
        let stability = Stability::new(0.7).unwrap();
        let display = format!("{}", stability);
        assert_eq!(display, "0.70");
    }

    // Difficulty tests - validation
    #[test]
    fn difficulty_new_valid() {
        let difficulty = Difficulty::new(0.3).unwrap();
        assert_eq!(difficulty.value(), 0.3);
    }

    #[test]
    fn difficulty_new_zero() {
        let difficulty = Difficulty::new(0.0).unwrap();
        assert_eq!(difficulty.value(), 0.0);
    }

    #[test]
    fn difficulty_new_negative_fails() {
        let result = Difficulty::new(-0.1);
        assert!(result.is_err());
        assert!(matches!(result, Err(OrigaError::InvalidDifficulty { .. })));
    }

    #[test]
    fn difficulty_display_formats_to_two_decimals() {
        let difficulty = Difficulty::new(0.5).unwrap();
        let display = format!("{}", difficulty);
        assert_eq!(display, "0.50");
    }

    // Rating enum tests
    #[rstest]
    #[case(Rating::Easy, "Easy")]
    #[case(Rating::Good, "Good")]
    #[case(Rating::Hard, "Hard")]
    #[case(Rating::Again, "Again")]
    fn rating_debug_contains_variant_name(#[case] rating: Rating, #[case] expected: &str) {
        let debug = format!("{:?}", rating);
        assert!(debug.contains(expected));
    }

    #[test]
    fn memory_state_deserialize_without_card_state_defaults_to_review() {
        let json = r#"{
            "stability": {"value": 10.0},
            "difficulty": {"value": 5.0},
            "next_review_date": "2025-01-01T00:00:00Z"
        }"#;

        let state: MemoryState = serde_json::from_str(json).unwrap();
        assert_eq!(state.card_state(), CardState::Review);
        assert_eq!(state.stability().value(), 10.0);
        assert_eq!(state.difficulty().value(), 5.0);
    }

    #[test]
    fn memory_state_roundtrip_with_card_state() {
        let original = MemoryState::with_card_state(
            Stability::new(15.0).unwrap(),
            Difficulty::new(3.0).unwrap(),
            chrono::Utc::now(),
            CardState::Learning,
        );

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: MemoryState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.card_state(), CardState::Learning);
        assert_eq!(deserialized.stability().value(), 15.0);
    }
}
