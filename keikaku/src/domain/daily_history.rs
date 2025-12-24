use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyHistoryItem {
    timestamp: DateTime<Utc>,
    avg_stability: Option<f64>,
    avg_difficulty: Option<f64>,

    total_words: usize,
    new_words: usize,
    known_words: usize,
    in_progress_words: usize,
    low_stability_words: usize,
    high_difficulty_words: usize,
    lessons_completed: usize,

    total_duration: Duration,
}

impl DailyHistoryItem {
    pub fn new(
        timestamp: DateTime<Utc>,
        avg_stability: Option<f64>,
        avg_difficulty: Option<f64>,
        total_words: usize,
        known_words: usize,
        new_words: usize,
        in_progress_words: usize,
        low_stability_words: usize,
        high_difficulty_words: usize,
        lesson_duration: Duration,
    ) -> Self {
        Self {
            timestamp,
            avg_stability,
            avg_difficulty,
            total_words,
            known_words,
            new_words,
            in_progress_words,
            low_stability_words,
            high_difficulty_words,
            lessons_completed: 1,
            total_duration: lesson_duration,
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

    pub fn in_progress_words(&self) -> usize {
        self.in_progress_words
    }

    pub fn low_stability_words(&self) -> usize {
        self.low_stability_words
    }

    pub fn high_difficulty_words(&self) -> usize {
        self.high_difficulty_words
    }

    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }

    pub fn update(
        &mut self,
        avg_stability: Option<f64>,
        avg_difficulty: Option<f64>,
        total_words: usize,
        known_words: usize,
        new_words: usize,
        in_progress_words: usize,
        low_stability_words: usize,
        high_difficulty_words: usize,
        lesson_duration: Duration,
    ) {
        self.avg_stability = avg_stability;
        self.avg_difficulty = avg_difficulty;
        self.total_words = total_words;
        self.known_words = known_words;
        self.new_words = new_words;
        self.in_progress_words = in_progress_words;
        self.low_stability_words = low_stability_words;
        self.high_difficulty_words = high_difficulty_words;
        self.lessons_completed += 1;
        self.total_duration += lesson_duration;
    }
}
