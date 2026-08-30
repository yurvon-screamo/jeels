use std::collections::HashMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{DailyHistoryItem, StudyCard};
use crate::domain::{RateMode, Rating};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StatsTracker {
    lesson_history: Vec<DailyHistoryItem>,
}

impl StatsTracker {
    pub fn new() -> Self {
        Self {
            lesson_history: Vec::new(),
        }
    }

    pub fn history(&self) -> &[DailyHistoryItem] {
        &self.lesson_history
    }

    pub fn new_cards_studied_today(&self) -> usize {
        let today = Utc::now().date_naive();
        self.lesson_history
            .iter()
            .rev()
            .find(|item| item.timestamp().date_naive() == today)
            .map(|item| item.new_cards_studied_today() as usize)
            .unwrap_or(0)
    }

    pub fn phrase_cards_studied_today(&self) -> usize {
        let today = Utc::now().date_naive();
        self.lesson_history
            .iter()
            .rev()
            .find(|item| item.timestamp().date_naive() == today)
            .map(|item| item.phrase_cards_studied_today() as usize)
            .unwrap_or(0)
    }

    pub fn update(
        &mut self,
        study_cards: &HashMap<Ulid, StudyCard>,
        rating: Rating,
        was_new: bool,
        is_phrase: bool,
        mode: RateMode,
    ) {
        super::stats_updater::update_history(
            study_cards,
            &mut self.lesson_history,
            rating,
            was_new,
            is_phrase,
            mode,
        );
    }

    /// Закрытие руки знакомства: списывает дневной лимит за `count` карт
    /// одной операцией, без рейтингового пути.
    pub fn register_acquaintance_completions(
        &mut self,
        study_cards: &HashMap<Ulid, StudyCard>,
        count: usize,
    ) {
        super::stats_updater::register_new_cards_without_rating(
            study_cards,
            &mut self.lesson_history,
            count,
        );
    }

    pub fn recalculate(&mut self, study_cards: &HashMap<Ulid, StudyCard>) {
        super::stats_updater::recalculate_daily_stats(study_cards, &mut self.lesson_history);
    }

    pub fn merge(&mut self, other: &StatsTracker) {
        for item in &other.lesson_history {
            let date = item.timestamp().date_naive();
            if let Some(existing_item) = self
                .lesson_history
                .iter_mut()
                .find(|h| h.timestamp().date_naive() == date)
            {
                existing_item.merge_with(item);
            } else {
                self.lesson_history.push(item.clone());
            }
        }

        self.lesson_history.sort_by_key(|h| h.timestamp());
    }
}

#[cfg(test)]
mod stats_tracker_tests {
    use super::*;
    use crate::domain::knowledge::daily_history::DailyStatsUpdate;
    use chrono::Duration;

    fn create_test_history_item(
        timestamp: chrono::DateTime<Utc>,
        new_studied: u32,
        phrase_studied: u32,
    ) -> DailyHistoryItem {
        let mut item = DailyHistoryItem::new();
        item.update_stats(DailyStatsUpdate {
            avg_stability: 0.5,
            avg_difficulty: 0.3,
            total_words: 10,
            known_words: 5,
            new_words: 5,
            in_progress_words: 0,
            high_difficulty_words: 0,
            positive_ratings: 1,
            negative_ratings: 0,
            total_ratings: 1,
            new_cards_studied_today: new_studied,
            phrase_cards_studied_today: phrase_studied,
        });
        item.set_timestamp_for_test(timestamp);
        item
    }

    #[test]
    fn new_tracker_has_empty_history() {
        let tracker = StatsTracker::new();
        assert!(tracker.history().is_empty());
        assert_eq!(tracker.new_cards_studied_today(), 0);
        assert_eq!(tracker.phrase_cards_studied_today(), 0);
    }

    #[test]
    fn merge_combines_history_from_different_days() {
        let now = Utc::now();
        let yesterday = now - Duration::days(1);

        let mut tracker1 = StatsTracker::new();
        tracker1
            .lesson_history
            .push(create_test_history_item(now, 3, 0));

        let mut tracker2 = StatsTracker::new();
        tracker2
            .lesson_history
            .push(create_test_history_item(yesterday, 5, 2));

        tracker1.merge(&tracker2);

        assert_eq!(tracker1.history().len(), 2);
        assert_eq!(tracker1.history()[0].new_cards_studied_today(), 5);
        assert_eq!(tracker1.history()[1].new_cards_studied_today(), 3);
    }

    #[test]
    fn merge_takes_max_for_same_day() {
        let now = Utc::now();

        let mut tracker1 = StatsTracker::new();
        tracker1
            .lesson_history
            .push(create_test_history_item(now, 3, 1));

        let mut tracker2 = StatsTracker::new();
        tracker2
            .lesson_history
            .push(create_test_history_item(now, 7, 4));

        tracker1.merge(&tracker2);

        assert_eq!(tracker1.history().len(), 1);
        assert_eq!(tracker1.history()[0].new_cards_studied_today(), 7);
        assert_eq!(tracker1.history()[0].phrase_cards_studied_today(), 4);
    }

    #[test]
    fn merge_sorts_history_by_timestamp() {
        let now = Utc::now();
        let day1 = now - Duration::days(2);
        let day2 = now - Duration::days(1);

        let mut tracker1 = StatsTracker::new();
        tracker1
            .lesson_history
            .push(create_test_history_item(now, 1, 0));

        let mut tracker2 = StatsTracker::new();
        tracker2
            .lesson_history
            .push(create_test_history_item(day1, 5, 0));
        tracker2
            .lesson_history
            .push(create_test_history_item(day2, 3, 0));

        tracker1.merge(&tracker2);

        assert_eq!(tracker1.history().len(), 3);
        assert!(tracker1.history()[0].timestamp() < tracker1.history()[1].timestamp());
        assert!(tracker1.history()[1].timestamp() < tracker1.history()[2].timestamp());
    }

    #[test]
    fn serialization_roundtrip_preserves_data() {
        let now = Utc::now();
        let mut tracker = StatsTracker::new();
        tracker
            .lesson_history
            .push(create_test_history_item(now, 4, 2));

        let json = serde_json::to_string(&tracker).unwrap();
        let deserialized: StatsTracker = serde_json::from_str(&json).unwrap();

        assert_eq!(tracker, deserialized);
    }

    #[test]
    fn knowledge_set_serialization_uses_lesson_history_key() {
        use crate::domain::knowledge::KnowledgeSet;

        let ks = KnowledgeSet::new();
        let json = serde_json::to_string(&ks).unwrap();

        assert!(
            json.contains("\"lesson_history\":[]"),
            "JSON must use 'lesson_history' key for backward compatibility, got: {json}"
        );
        assert!(
            !json.contains("\"stats\""),
            "JSON must not contain 'stats' key, got: {json}"
        );
    }

    #[test]
    fn knowledge_set_deserializes_old_format() {
        use crate::domain::knowledge::KnowledgeSet;

        let old_json = r#"{"study_cards":{},"deleted_cards":[],"lesson_history":[]}"#;
        let ks: KnowledgeSet = serde_json::from_str(old_json).unwrap();

        assert!(ks.lesson_history().is_empty());
        assert!(ks.study_cards().is_empty());
    }
}
