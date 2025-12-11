use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyHistoryItem {
    timestamp: DateTime<Utc>,
    avg_stability: Option<f64>,
    avg_difficulty: Option<f64>,

    total_words: usize,
    known_words: usize,
    new_words: usize,
    lessons_completed: usize,
}

impl DailyHistoryItem {
    pub fn new(
        timestamp: DateTime<Utc>,
        avg_stability: Option<f64>,
        avg_difficulty: Option<f64>,
        total_words: usize,
        known_words: usize,
        new_words: usize,
    ) -> Self {
        Self {
            timestamp,
            avg_stability,
            avg_difficulty,
            total_words,
            known_words,
            new_words,
            lessons_completed: 1,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn avg_stability(&self) -> Option<f64> {
        self.avg_stability
    }

    pub fn avg_difficulty(&self) -> Option<f64> {
        self.avg_difficulty
    }

    pub fn total_words(&self) -> usize {
        self.total_words
    }

    pub fn known_words(&self) -> usize {
        self.known_words
    }

    pub fn new_words(&self) -> usize {
        self.new_words
    }

    pub fn update(
        &mut self,
        avg_stability: Option<f64>,
        avg_difficulty: Option<f64>,
        total_words: usize,
        known_words: usize,
        new_words: usize,
    ) {
        self.avg_stability = avg_stability;
        self.avg_difficulty = avg_difficulty;
        self.total_words = total_words;
        self.known_words = known_words;
        self.new_words = new_words;
        self.lessons_completed += 1;
    }
}
