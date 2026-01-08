use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl Default for DailyHistoryItem {
    fn default() -> Self {
        Self::new()
    }
}

impl DailyHistoryItem {
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            avg_stability: None,
            avg_difficulty: None,
            total_words: 0,
            known_words: 0,
            new_words: 0,
            in_progress_words: 0,
            low_stability_words: 0,
            high_difficulty_words: 0,
            lessons_completed: 0,
            total_duration: Duration::zero(),
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
    }

    pub fn add_lesson_duration(&mut self, lesson_duration: Duration) {
        self.total_duration += lesson_duration;
    }
}
