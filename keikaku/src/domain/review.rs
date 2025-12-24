use core::fmt;
use std::collections::VecDeque;

use crate::domain::value_objects::{Difficulty, Rating, Stability};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

const LOW_STABILITY_THRESHOLD: f64 = 2.0;
const KNOWN_CARD_STABILITY_THRESHOLD: f64 = 7.0;
const HIGH_DIFFICULTY_THRESHOLD: f64 = 1.5;

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MemoryState {
    stability: Stability,
    difficulty: Difficulty,
    next_review_date: DateTime<Utc>,
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
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MemoryHistory {
    current_state: Option<MemoryState>,
    reviews: VecDeque<Review>,
}

impl Default for MemoryHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryHistory {
    pub fn new() -> Self {
        Self {
            current_state: None,
            reviews: VecDeque::new(),
        }
    }

    pub fn memory_state(&self) -> Option<&MemoryState> {
        self.current_state.as_ref()
    }

    pub fn stability(&self) -> Option<&Stability> {
        self.current_state.as_ref().map(|state| &state.stability)
    }

    pub fn difficulty(&self) -> Option<&Difficulty> {
        self.current_state.as_ref().map(|state| &state.difficulty)
    }

    pub fn next_review_date(&self) -> Option<&DateTime<Utc>> {
        self.current_state
            .as_ref()
            .map(|state| &state.next_review_date)
    }

    pub fn reviews(&self) -> &VecDeque<Review> {
        &self.reviews
    }

    pub(crate) fn add_review(&mut self, memory_state: MemoryState, review: Review) {
        self.current_state = Some(memory_state);
        self.reviews.push_back(review);
    }

    pub fn last_review_date(&self) -> Option<DateTime<Utc>> {
        self.reviews.back().map(|review| review.timestamp())
    }

    /// Карта которая требует повторения
    pub fn is_due(&self) -> bool {
        !self.is_new() && self.next_review_date() <= Some(&Utc::now())
    }

    /// Карта изучение которой еще не началось``
    pub fn is_new(&self) -> bool {
        self.current_state.is_none()
    }

    /// Карта которая имеет низкую стабильность
    pub fn is_low_stability(&self) -> bool {
        self.stability()
            .map(|stability| stability.value() < LOW_STABILITY_THRESHOLD)
            .unwrap_or(false)
    }

    /// Карта которая имеет высокую сложность
    pub fn is_high_difficulty(&self) -> bool {
        self.difficulty()
            .map(|difficulty| difficulty.value() >= HIGH_DIFFICULTY_THRESHOLD)
            .unwrap_or(false)
    }

    /// Карта которая еще не была изучена до стабильного уровня, но уже начала изучаться
    pub fn is_in_progress(&self) -> bool {
        self.stability()
            .map(|stability| {
                stability.value() < KNOWN_CARD_STABILITY_THRESHOLD
                    && stability.value() >= LOW_STABILITY_THRESHOLD
            })
            .unwrap_or(false)
    }

    /// Карта которая уже изучена до стабильного уровня
    pub fn is_known_card(&self) -> bool {
        self.stability()
            .map(|stability| stability.value() > KNOWN_CARD_STABILITY_THRESHOLD)
            .unwrap_or(false)
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
pub struct Review {
    id: Ulid,
    rating: Rating,
    timestamp: DateTime<Utc>,
    interval: Duration,
}

impl Review {
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
