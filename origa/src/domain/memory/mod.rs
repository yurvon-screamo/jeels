mod value;

pub use value::{CardState, Difficulty, MemoryState, Rating, ReviewLog, Stability};

use std::collections::{HashSet, VecDeque};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub(crate) const KNOWN_CARD_STABILITY_THRESHOLD: f64 = 21.0;
const HIGH_DIFFICULTY_THRESHOLD: f64 = 7.0;
const HIGH_DIFFICULTY_STABILITY_CAP: f64 = 7.0;

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MemoryHistory {
    current_state: Option<MemoryState>,
    reviews: VecDeque<ReviewLog>,
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
        self.current_state.as_ref().map(|state| state.stability())
    }

    pub fn difficulty(&self) -> Option<&Difficulty> {
        self.current_state.as_ref().map(|state| state.difficulty())
    }

    pub fn next_review_date(&self) -> Option<&DateTime<Utc>> {
        self.current_state
            .as_ref()
            .map(|state| state.next_review_date())
    }

    pub fn reviews(&self) -> &VecDeque<ReviewLog> {
        &self.reviews
    }

    pub fn easy_review_count(&self) -> usize {
        self.reviews
            .iter()
            .filter(|review| review.rating() == Rating::Easy)
            .count()
    }

    pub fn good_review_count(&self) -> usize {
        self.reviews
            .iter()
            .filter(|review| review.rating() == Rating::Good)
            .count()
    }

    pub(crate) fn add_review(&mut self, memory_state: MemoryState, review: ReviewLog) {
        self.current_state = Some(memory_state);
        self.reviews.push_back(review);
    }

    pub fn last_review_date(&self) -> Option<DateTime<Utc>> {
        self.reviews.back().map(|review| review.timestamp())
    }

    /// Карточка которая требует повторения
    pub fn is_due(&self) -> bool {
        !self.is_new() && self.next_review_date() <= Some(&Utc::now())
    }

    /// Карточка, изучение которой еще не началось
    pub fn is_new(&self) -> bool {
        self.current_state.is_none()
    }

    /// Карточка которая имеет высокую сложность
    pub fn is_high_difficulty(&self) -> bool {
        self.difficulty()
            .map(|d| d.value() >= HIGH_DIFFICULTY_THRESHOLD)
            .unwrap_or(false)
            && self
                .stability()
                .map(|s| s.value() < HIGH_DIFFICULTY_STABILITY_CAP)
                .unwrap_or(false)
    }

    /// Карточка которая уже изучена до стабильного уровня
    pub fn is_known_card(&self) -> bool {
        self.stability()
            .map(|stability| stability.value() > KNOWN_CARD_STABILITY_THRESHOLD)
            .unwrap_or(false)
            && !self.is_high_difficulty()
    }

    /// Карточка которая еще не была изучена до стабильного уровня, но уже начала изучаться
    pub fn is_in_progress(&self) -> bool {
        !self.is_known_card() && !self.is_high_difficulty() && !self.is_new()
    }

    pub fn merge(&mut self, other: &MemoryHistory) {
        self.current_state = select_later_state(
            &self.current_state,
            &other.current_state,
            self.last_review_date(),
            other.last_review_date(),
        );

        let existing_ids: HashSet<Ulid> = self.reviews.iter().map(|r| r.id()).collect();

        for review in &other.reviews {
            if !existing_ids.contains(&review.id()) {
                self.reviews.push_back(*review);
            }
        }

        self.reviews
            .make_contiguous()
            .sort_by_key(|r| r.timestamp());
    }
}

fn select_later_state(
    left: &Option<MemoryState>,
    right: &Option<MemoryState>,
    left_last_review: Option<DateTime<Utc>>,
    right_last_review: Option<DateTime<Utc>>,
) -> Option<MemoryState> {
    match (left, right) {
        (None, None) => None,
        (Some(l), None) => Some(l.clone()),
        (None, Some(r)) => Some(r.clone()),
        (Some(l), Some(r)) => match (left_last_review, right_last_review) {
            (None, None) => Some(r.clone()),
            (Some(_), None) => Some(l.clone()),
            (None, Some(_)) => Some(r.clone()),
            (Some(left_date), Some(right_date)) => {
                if right_date >= left_date {
                    Some(r.clone())
                } else {
                    Some(l.clone())
                }
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_review(rating: Rating) -> ReviewLog {
        ReviewLog::new(rating, Duration::days(1))
    }

    fn make_state() -> MemoryState {
        MemoryState::new(
            Stability::new(5.0).unwrap(),
            Difficulty::new(3.0).unwrap(),
            Utc::now(),
        )
    }

    #[test]
    fn easy_review_count_empty_history() {
        let history = MemoryHistory::new();
        assert_eq!(history.easy_review_count(), 0);
    }

    #[test]
    fn easy_review_count_no_easy_reviews() {
        let mut history = MemoryHistory::new();
        let state = make_state();
        history.add_review(state.clone(), make_review(Rating::Good));
        history.add_review(state.clone(), make_review(Rating::Hard));
        history.add_review(state.clone(), make_review(Rating::Again));
        assert_eq!(history.easy_review_count(), 0);
    }

    #[test]
    fn easy_review_count_mixed_reviews() {
        let mut history = MemoryHistory::new();
        let state = make_state();
        history.add_review(state.clone(), make_review(Rating::Easy));
        history.add_review(state.clone(), make_review(Rating::Good));
        history.add_review(state.clone(), make_review(Rating::Easy));
        history.add_review(state.clone(), make_review(Rating::Easy));
        history.add_review(state.clone(), make_review(Rating::Hard));
        assert_eq!(history.easy_review_count(), 3);
    }

    #[test]
    fn easy_review_count_all_easy() {
        let mut history = MemoryHistory::new();
        let state = make_state();
        for _ in 0..5 {
            history.add_review(state.clone(), make_review(Rating::Easy));
        }
        assert_eq!(history.easy_review_count(), 5);
    }

    #[test]
    fn good_review_count_empty_history() {
        let history = MemoryHistory::new();
        assert_eq!(history.good_review_count(), 0);
    }

    #[test]
    fn good_review_count_no_good_reviews() {
        let mut history = MemoryHistory::new();
        let state = make_state();
        history.add_review(state.clone(), make_review(Rating::Easy));
        history.add_review(state.clone(), make_review(Rating::Hard));
        assert_eq!(history.good_review_count(), 0);
    }

    #[test]
    fn good_review_count_mixed_reviews() {
        let mut history = MemoryHistory::new();
        let state = make_state();
        history.add_review(state.clone(), make_review(Rating::Good));
        history.add_review(state.clone(), make_review(Rating::Easy));
        history.add_review(state.clone(), make_review(Rating::Good));
        history.add_review(state.clone(), make_review(Rating::Hard));
        history.add_review(state.clone(), make_review(Rating::Good));
        assert_eq!(history.good_review_count(), 3);
    }

    // --- is_due ---

    #[test]
    fn is_due_true_when_next_review_in_past() {
        // Arrange
        let past = Utc::now() - Duration::days(1);
        let state = MemoryState::new(
            Stability::new(5.0).unwrap(),
            Difficulty::new(3.0).unwrap(),
            past,
        );
        let mut history = MemoryHistory::new();
        history.add_review(state, make_review(Rating::Good));

        // Act & Assert
        assert!(history.is_due());
    }

    #[test]
    fn is_due_false_when_next_review_in_future() {
        // Arrange
        let future = Utc::now() + Duration::days(1);
        let state = MemoryState::new(
            Stability::new(5.0).unwrap(),
            Difficulty::new(3.0).unwrap(),
            future,
        );
        let mut history = MemoryHistory::new();
        history.add_review(state, make_review(Rating::Good));

        // Act & Assert
        assert!(!history.is_due());
    }

    #[test]
    fn is_due_false_when_no_memory_state() {
        // Arrange
        let history = MemoryHistory::new();

        // Act & Assert
        assert!(!history.is_due());
    }

    // --- is_high_difficulty ---

    #[test]
    fn is_high_difficulty_true_above_threshold() {
        // Arrange
        let state = MemoryState::new(
            Stability::new(5.0).unwrap(),
            Difficulty::new(HIGH_DIFFICULTY_THRESHOLD + 0.1).unwrap(),
            Utc::now(),
        );
        let mut history = MemoryHistory::new();
        history.add_review(state, make_review(Rating::Hard));

        // Act & Assert
        assert!(history.is_high_difficulty());
    }

    #[test]
    fn is_high_difficulty_false_below_threshold() {
        // Arrange
        let state = MemoryState::new(
            Stability::new(5.0).unwrap(),
            Difficulty::new(HIGH_DIFFICULTY_THRESHOLD - 0.1).unwrap(),
            Utc::now(),
        );
        let mut history = MemoryHistory::new();
        history.add_review(state, make_review(Rating::Good));

        // Act & Assert
        assert!(!history.is_high_difficulty());
    }

    #[test]
    fn is_high_difficulty_false_when_no_memory_state() {
        // Arrange
        let history = MemoryHistory::new();

        // Act & Assert
        assert!(!history.is_high_difficulty());
    }

    #[test]
    fn is_high_difficulty_false_when_stability_above_cap() {
        let state = MemoryState::new(
            Stability::new(HIGH_DIFFICULTY_STABILITY_CAP + 1.0).unwrap(),
            Difficulty::new(HIGH_DIFFICULTY_THRESHOLD + 0.1).unwrap(),
            Utc::now(),
        );
        let mut history = MemoryHistory::new();
        history.add_review(state, make_review(Rating::Hard));

        assert!(!history.is_high_difficulty());
    }

    #[test]
    fn is_high_difficulty_true_when_stability_below_cap() {
        let state = MemoryState::new(
            Stability::new(HIGH_DIFFICULTY_STABILITY_CAP - 1.0).unwrap(),
            Difficulty::new(HIGH_DIFFICULTY_THRESHOLD + 0.1).unwrap(),
            Utc::now(),
        );
        let mut history = MemoryHistory::new();
        history.add_review(state, make_review(Rating::Hard));

        assert!(history.is_high_difficulty());
    }

    // --- is_in_progress ---

    #[test]
    fn is_in_progress_true_when_stability_below_threshold() {
        // Arrange
        let state = MemoryState::new(
            Stability::new(KNOWN_CARD_STABILITY_THRESHOLD - 0.1).unwrap(),
            Difficulty::new(3.0).unwrap(),
            Utc::now() + Duration::days(1),
        );
        let mut history = MemoryHistory::new();
        history.add_review(state, make_review(Rating::Good));

        // Act & Assert
        assert!(history.is_in_progress());
    }

    #[test]
    fn is_in_progress_false_when_stability_above_threshold() {
        // Arrange
        let state = MemoryState::new(
            Stability::new(KNOWN_CARD_STABILITY_THRESHOLD + 0.1).unwrap(),
            Difficulty::new(3.0).unwrap(),
            Utc::now() + Duration::days(1),
        );
        let mut history = MemoryHistory::new();
        history.add_review(state, make_review(Rating::Good));

        // Act & Assert
        assert!(!history.is_in_progress());
    }

    #[test]
    fn is_in_progress_false_when_no_memory_state() {
        // Arrange
        let history = MemoryHistory::new();

        // Act & Assert
        assert!(!history.is_in_progress());
    }

    // --- merge ---

    #[test]
    fn merge_empty_with_non_empty_result_is_non_empty() {
        // Arrange
        let mut empty = MemoryHistory::new();
        let state = make_state();
        let mut non_empty = MemoryHistory::new();
        non_empty.add_review(state, make_review(Rating::Good));

        // Act
        empty.merge(&non_empty);

        // Assert
        assert!(empty.memory_state().is_some());
        assert_eq!(empty.reviews().len(), 1);
    }

    #[test]
    fn merge_non_empty_with_empty_result_is_non_empty() {
        // Arrange
        let state = make_state();
        let mut non_empty = MemoryHistory::new();
        non_empty.add_review(state, make_review(Rating::Good));
        let original_state = non_empty.memory_state().cloned();
        let empty = MemoryHistory::new();

        // Act
        non_empty.merge(&empty);

        // Assert
        assert_eq!(non_empty.memory_state(), original_state.as_ref());
        assert_eq!(non_empty.reviews().len(), 1);
    }

    #[test]
    fn merge_both_with_reviews_combines_and_deduplicates() {
        // Arrange
        let state1 = MemoryState::new(
            Stability::new(3.0).unwrap(),
            Difficulty::new(2.0).unwrap(),
            Utc::now() - Duration::days(2),
        );
        let state2 = MemoryState::new(
            Stability::new(5.0).unwrap(),
            Difficulty::new(4.0).unwrap(),
            Utc::now(),
        );

        let mut history_a = MemoryHistory::new();
        history_a.add_review(state1, make_review(Rating::Good));

        let mut history_b = MemoryHistory::new();
        history_b.add_review(state2, make_review(Rating::Easy));

        // Act
        history_a.merge(&history_b);

        // Assert — два уникальных review
        assert_eq!(history_a.reviews().len(), 2);
        // select_later_state выбирает state с более поздним last_review_date
        assert_eq!(history_a.memory_state().unwrap().difficulty().value(), 4.0);
    }
}
