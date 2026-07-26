use super::*;
use chrono::Duration;
use rstest::rstest;

fn create_test_item(
    timestamp: DateTime<Utc>,
    lessons: usize,
    known: usize,
    total: usize,
) -> DailyHistoryItem {
    let mut item = DailyHistoryItem::new();
    item.update_stats(DailyStatsUpdate {
        avg_stability: 0.5,
        avg_difficulty: 0.3,
        total_words: total,
        known_words: known,
        new_words: total - known,
        in_progress_words: 0,
        high_difficulty_words: 0,
        positive_ratings: 0,
        negative_ratings: 0,
        total_ratings: 0,
        new_cards_studied_today: 0,
        phrase_cards_studied_today: 0,
    });
    for _ in 0..lessons {
        item.lessons_completed += 1;
    }
    item.timestamp = timestamp;
    item
}

fn filler_stats() -> DailyStatsUpdate {
    DailyStatsUpdate {
        avg_stability: 0.5,
        avg_difficulty: 0.3,
        total_words: 5,
        known_words: 10,
        new_words: 2,
        in_progress_words: 3,
        high_difficulty_words: 1,
        positive_ratings: 0,
        negative_ratings: 0,
        total_ratings: 0,
        new_cards_studied_today: 0,
        phrase_cards_studied_today: 0,
    }
}

#[test]
fn daily_history_item_new_has_zero_defaults() {
    let item = DailyHistoryItem::new();

    assert_eq!(item.lessons_completed(), 0);
    assert_eq!(item.known_words(), 0);
    assert_eq!(item.total_words(), 0);
    assert_eq!(item.avg_stability(), None);
    assert_eq!(item.avg_difficulty(), None);
}

#[test]
fn daily_history_item_getters_return_set_values() {
    let now = Utc::now();
    let item = create_test_item(now, 1, 3, 8);

    assert_eq!(item.avg_stability(), Some(0.5));
    assert_eq!(item.avg_difficulty(), Some(0.3));
    assert_eq!(item.new_words(), 5); // 8 - 3
    assert_eq!(item.in_progress_words(), 0);
    assert_eq!(item.high_difficulty_words(), 0);
}

#[test]
fn update_with_good_rating_sets_all_fields() {
    let mut item = DailyHistoryItem::new();

    item.update(
        DailyStatsUpdate {
            avg_stability: 0.5,
            avg_difficulty: 0.3,
            total_words: 5,
            known_words: 10,
            new_words: 2,
            in_progress_words: 3,
            high_difficulty_words: 1,
            positive_ratings: 0,
            negative_ratings: 0,
            total_ratings: 0,
            new_cards_studied_today: 0,
            phrase_cards_studied_today: 0,
        },
        Rating::Good,
    );

    assert_eq!(item.lessons_completed(), 1);
    assert_eq!(item.total_words(), 5);
    assert_eq!(item.known_words(), 10);
    assert_eq!(item.new_words(), 2);
    assert_eq!(item.in_progress_words(), 3);
    assert_eq!(item.high_difficulty_words(), 1);
    assert_eq!(item.avg_stability(), Some(0.5));
    assert_eq!(item.avg_difficulty(), Some(0.3));
    assert_eq!(item.positive_ratings(), 1);
    assert_eq!(item.negative_ratings(), 0);
    assert_eq!(item.total_ratings(), 1);
}

#[test]
fn merge_with_takes_higher_lessons() {
    let now = Utc::now();
    let mut item1 = create_test_item(now, 2, 5, 10);
    let item2 = create_test_item(now, 5, 3, 8);

    item1.merge_with(&item2);

    assert_eq!(item1.lessons_completed(), 5);
}

#[test]
fn merge_with_preserves_known_words_when_other_older() {
    let now = Utc::now();
    let older = now - Duration::seconds(100);
    let mut item1 = create_test_item(now, 2, 5, 10);
    let item2 = create_test_item(older, 5, 8, 12);

    let known_before = item1.known_words();
    item1.merge_with(&item2);

    assert_eq!(item1.known_words(), known_before);
}

#[test]
fn merge_with_updates_timestamp_when_newer() {
    let now = Utc::now();
    let newer = now + Duration::seconds(100);
    let mut item1 = create_test_item(now, 2, 5, 10);
    let item2 = create_test_item(newer, 5, 3, 8);

    item1.merge_with(&item2);

    assert_eq!(item1.timestamp(), newer);
    assert_eq!(item1.avg_stability(), Some(0.5));
    assert_eq!(item1.avg_difficulty(), Some(0.3));
}

#[test]
fn merge_with_does_not_update_timestamp_when_older() {
    let now = Utc::now();
    let older = now - Duration::seconds(100);
    let mut item1 = create_test_item(now, 2, 5, 10);
    let item2 = create_test_item(older, 5, 3, 8);

    let timestamp_before = item1.timestamp();
    item1.merge_with(&item2);

    assert_eq!(item1.timestamp(), timestamp_before);
}

#[test]
fn merge_with_updates_stats_when_newer() {
    let now = Utc::now();
    let newer = now + Duration::seconds(100);
    let mut item1 = create_test_item(now, 2, 5, 10);
    let mut item2 = create_test_item(newer, 5, 3, 8);
    item2.update_stats(DailyStatsUpdate {
        avg_stability: 0.8,
        avg_difficulty: 0.6,
        total_words: 8,
        known_words: 3,
        new_words: 5,
        in_progress_words: 0,
        high_difficulty_words: 0,
        positive_ratings: 0,
        negative_ratings: 0,
        total_ratings: 0,
        new_cards_studied_today: 0,
        phrase_cards_studied_today: 0,
    });

    item1.merge_with(&item2);

    assert_eq!(item1.avg_stability(), Some(0.8));
    assert_eq!(item1.avg_difficulty(), Some(0.6));
}

#[test]
fn merge_with_preserves_stats_when_other_older() {
    let now = Utc::now();
    let older = now - Duration::seconds(100);
    let mut item1 = create_test_item(now, 2, 5, 10);
    let mut item2 = create_test_item(older, 5, 3, 8);
    item2.update_stats(DailyStatsUpdate {
        avg_stability: 0.8,
        avg_difficulty: 0.6,
        total_words: 8,
        known_words: 3,
        new_words: 5,
        in_progress_words: 0,
        high_difficulty_words: 0,
        positive_ratings: 0,
        negative_ratings: 0,
        total_ratings: 0,
        new_cards_studied_today: 0,
        phrase_cards_studied_today: 0,
    });

    let stability_before = item1.avg_stability();
    let difficulty_before = item1.avg_difficulty();
    item1.merge_with(&item2);

    assert_eq!(item1.avg_stability(), stability_before);
    assert_eq!(item1.avg_difficulty(), difficulty_before);
}

#[rstest]
#[case(Rating::Good, true)]
#[case(Rating::Easy, true)]
#[case(Rating::Again, false)]
#[case(Rating::Hard, false)]
fn update_classifies_rating_sentiment(#[case] rating: Rating, #[case] positive: bool) {
    let mut item = DailyHistoryItem::new();
    item.update(filler_stats(), rating);

    let (expected_positive, expected_negative) = if positive { (1, 0) } else { (0, 1) };
    assert_eq!(item.positive_ratings(), expected_positive);
    assert_eq!(item.negative_ratings(), expected_negative);
    assert_eq!(item.total_ratings(), 1);
}

#[test]
fn update_accumulates_ratings() {
    let mut item = DailyHistoryItem::new();
    item.update(filler_stats(), Rating::Good);
    item.update(filler_stats(), Rating::Easy);
    item.update(filler_stats(), Rating::Again);

    assert_eq!(item.positive_ratings(), 2);
    assert_eq!(item.negative_ratings(), 1);
    assert_eq!(item.total_ratings(), 3);
}

#[test]
fn merge_with_takes_max_ratings() {
    let now = Utc::now();
    let mut item1 = DailyHistoryItem::new();
    item1.update_stats(DailyStatsUpdate {
        avg_stability: 0.5,
        avg_difficulty: 0.3,
        total_words: 10,
        known_words: 5,
        new_words: 5,
        in_progress_words: 0,
        high_difficulty_words: 0,
        positive_ratings: 3,
        negative_ratings: 2,
        total_ratings: 5,
        new_cards_studied_today: 0,
        phrase_cards_studied_today: 0,
    });
    item1.timestamp = now;

    let mut item2 = DailyHistoryItem::new();
    item2.update_stats(DailyStatsUpdate {
        avg_stability: 0.6,
        avg_difficulty: 0.4,
        total_words: 12,
        known_words: 6,
        new_words: 6,
        in_progress_words: 0,
        high_difficulty_words: 0,
        positive_ratings: 5,
        negative_ratings: 3,
        total_ratings: 8,
        new_cards_studied_today: 0,
        phrase_cards_studied_today: 0,
    });
    item2.timestamp = now;

    item1.merge_with(&item2);

    assert_eq!(item1.positive_ratings(), 5);
    assert_eq!(item1.negative_ratings(), 3);
    assert_eq!(item1.total_ratings(), 8);
}

#[test]
fn new_cards_studied_today_default_is_zero() {
    let item = DailyHistoryItem::new();
    assert_eq!(item.new_cards_studied_today(), 0);
}

#[test]
fn increment_new_cards_studied_accumulates() {
    let mut item = DailyHistoryItem::new();
    item.increment_new_cards_studied();
    item.increment_new_cards_studied();
    assert_eq!(item.new_cards_studied_today(), 2);
}

#[test]
fn merge_with_takes_max_new_cards_studied() {
    let now = Utc::now();
    let mut item1 = create_test_item(now, 2, 5, 10);
    let mut item2 = create_test_item(now, 5, 3, 8);
    item2.increment_new_cards_studied();
    item2.increment_new_cards_studied();
    item2.increment_new_cards_studied();
    item1.merge_with(&item2);
    assert_eq!(item1.new_cards_studied_today(), 3);
}

#[test]
fn phrase_cards_studied_today_default_is_zero() {
    let item = DailyHistoryItem::new();
    assert_eq!(item.phrase_cards_studied_today(), 0);
}

#[test]
fn increment_phrase_cards_studied_accumulates() {
    let mut item = DailyHistoryItem::new();
    item.increment_phrase_cards_studied();
    item.increment_phrase_cards_studied();
    assert_eq!(item.phrase_cards_studied_today(), 2);
}

#[test]
fn merge_with_takes_max_phrase_cards_studied() {
    let now = Utc::now();
    let mut item1 = create_test_item(now, 2, 5, 10);
    let mut item2 = create_test_item(now, 5, 3, 8);
    item2.increment_phrase_cards_studied();
    item2.increment_phrase_cards_studied();
    item1.merge_with(&item2);
    assert_eq!(item1.phrase_cards_studied_today(), 2);
}

fn create_test_item_with_new_studied(
    timestamp: DateTime<Utc>,
    new_studied: u32,
) -> DailyHistoryItem {
    let mut item = DailyHistoryItem::new();
    item.timestamp = timestamp;
    for _ in 0..new_studied {
        item.increment_new_cards_studied();
    }
    item
}

#[test]
fn estimate_returns_none_when_no_new_cards() {
    let history = vec![create_test_item_with_new_studied(
        Utc::now() - Duration::days(1),
        5,
    )];
    assert!(estimate_completion_date(&history, 0).is_none());
}

#[test]
fn estimate_returns_none_when_no_history() {
    assert!(estimate_completion_date(&[], 100).is_none());
}

#[test]
fn estimate_returns_none_when_all_zero_studied() {
    let history = vec![create_test_item_with_new_studied(
        Utc::now() - Duration::days(1),
        0,
    )];
    assert!(estimate_completion_date(&history, 100).is_none());
}

#[test]
fn estimate_returns_none_when_excludes_today() {
    let history = vec![create_test_item_with_new_studied(Utc::now(), 10)];
    assert!(estimate_completion_date(&history, 100).is_none());
}

#[test]
fn estimate_returns_expected_date_for_simple_history() {
    let history = vec![create_test_item_with_new_studied(
        Utc::now() - Duration::days(1),
        5,
    )];
    let result = estimate_completion_date(&history, 50);
    assert!(result.is_some());
    let expected = Utc::now() + Duration::days(12);
    assert_eq!(result.unwrap().date_naive(), expected.date_naive());
}

#[test]
fn estimate_averages_over_multiple_days() {
    let history = vec![
        create_test_item_with_new_studied(Utc::now() - Duration::days(2), 10),
        create_test_item_with_new_studied(Utc::now() - Duration::days(1), 20),
    ];
    let result = estimate_completion_date(&history, 30);
    assert!(result.is_some());
    let expected = Utc::now() + Duration::days(3);
    assert_eq!(result.unwrap().date_naive(), expected.date_naive());
}

#[test]
fn estimate_skips_zero_study_days() {
    let history = vec![
        create_test_item_with_new_studied(Utc::now() - Duration::days(3), 10),
        create_test_item_with_new_studied(Utc::now() - Duration::days(2), 0),
        create_test_item_with_new_studied(Utc::now() - Duration::days(1), 10),
    ];
    let result = estimate_completion_date(&history, 20);
    assert!(result.is_some());
    let expected = Utc::now() + Duration::days(3);
    assert_eq!(result.unwrap().date_naive(), expected.date_naive());
}

#[test]
fn estimate_uses_last_10_non_today_records() {
    let history: Vec<DailyHistoryItem> = (1..=15)
        .map(|i| create_test_item_with_new_studied(Utc::now() - Duration::days(i), 5))
        .collect();
    let result = estimate_completion_date(&history, 50);
    assert!(result.is_some());
    let expected = Utc::now() + Duration::days(12);
    assert_eq!(result.unwrap().date_naive(), expected.date_naive());
}
