use std::collections::HashMap;

use chrono::Utc;
use ulid::Ulid;

use super::daily_history::DailyStatsUpdate;
use super::{Card, DailyHistoryItem, StudyCard};
use crate::domain::{RateMode, Rating};

struct ComputedStats {
    avg_stability: f64,
    avg_difficulty: f64,
    total_words: usize,
    known_words: usize,
    new_words: usize,
    in_progress_words: usize,
    high_difficulty_words: usize,
}

impl ComputedStats {
    fn compute(study_cards: &HashMap<Ulid, StudyCard>) -> Option<Self> {
        let mut stats = Self {
            avg_stability: 0.0,
            avg_difficulty: 0.0,
            total_words: 0,
            known_words: 0,
            new_words: 0,
            in_progress_words: 0,
            high_difficulty_words: 0,
        };

        for study_card in study_cards.values() {
            if matches!(study_card.card(), Card::Phrase(_)) {
                continue;
            }
            let memory = study_card.memory();
            stats.avg_stability += memory.stability().map(|x| x.value()).unwrap_or(0.0);
            stats.avg_difficulty += memory.difficulty().map(|x| x.value()).unwrap_or(0.0);
            stats.total_words += 1;
            stats.known_words += memory.is_known_card() as usize;
            stats.new_words += memory.is_new() as usize;
            stats.in_progress_words += memory.is_in_progress() as usize;
            stats.high_difficulty_words += memory.is_high_difficulty() as usize;
        }

        if stats.total_words == 0 {
            return None;
        }

        stats.avg_stability /= stats.total_words as f64;
        stats.avg_difficulty /= stats.total_words as f64;
        Some(stats)
    }

    fn to_daily_update(
        &self,
        positive_ratings: usize,
        negative_ratings: usize,
        total_ratings: usize,
        new_cards_studied_today: u32,
        phrase_cards_studied_today: u32,
    ) -> DailyStatsUpdate {
        DailyStatsUpdate {
            avg_stability: self.avg_stability,
            avg_difficulty: self.avg_difficulty,
            total_words: self.total_words,
            known_words: self.known_words,
            new_words: self.new_words,
            in_progress_words: self.in_progress_words,
            high_difficulty_words: self.high_difficulty_words,
            positive_ratings,
            negative_ratings,
            total_ratings,
            new_cards_studied_today,
            phrase_cards_studied_today,
        }
    }
}

pub(crate) fn update_history(
    study_cards: &HashMap<Ulid, StudyCard>,
    lesson_history: &mut Vec<DailyHistoryItem>,
    rating: Rating,
    was_new: bool,
    is_phrase: bool,
    mode: RateMode,
) {
    let stats = match ComputedStats::compute(study_cards) {
        Some(s) => s,
        None => return,
    };

    let today = Utc::now().date_naive();

    if let Some(existing_item) = lesson_history
        .iter_mut()
        .find(|item| item.timestamp().date_naive() == today)
    {
        if was_new && !is_phrase && mode != RateMode::OnboardingScoring {
            existing_item.increment_new_cards_studied();
        }
        if was_new && is_phrase {
            existing_item.increment_phrase_cards_studied();
        }

        let update = stats.to_daily_update(
            existing_item.positive_ratings(),
            existing_item.negative_ratings(),
            existing_item.total_ratings(),
            existing_item.new_cards_studied_today(),
            existing_item.phrase_cards_studied_today(),
        );

        if is_phrase {
            existing_item.update_stats(update);
        } else {
            existing_item.update(update, rating);
        }
    } else {
        let mut item = DailyHistoryItem::new();
        if was_new && !is_phrase && mode != RateMode::OnboardingScoring {
            item.increment_new_cards_studied();
        }
        if was_new && is_phrase {
            item.increment_phrase_cards_studied();
        }

        let update = stats.to_daily_update(
            0,
            0,
            0,
            item.new_cards_studied_today(),
            item.phrase_cards_studied_today(),
        );

        if is_phrase {
            item.update_stats(update);
        } else {
            item.update(update, rating);
        }
        lesson_history.push(item);
    }
}

/// Учёт закрытия руки знакомства (docs/acquaintance-mode.md §4): дневной
/// лимит тратится одной операцией на все карты руки; рейтинговый путь
/// (`update_history`) для этих карт не вызывается.
pub(crate) fn register_new_cards_without_rating(
    study_cards: &HashMap<Ulid, StudyCard>,
    lesson_history: &mut Vec<DailyHistoryItem>,
    count: usize,
) {
    let Some(stats) = ComputedStats::compute(study_cards) else {
        return;
    };

    let today = Utc::now().date_naive();
    if let Some(existing_item) = lesson_history
        .iter_mut()
        .find(|item| item.timestamp().date_naive() == today)
    {
        for _ in 0..count {
            existing_item.increment_new_cards_studied();
        }
        let update = stats.to_daily_update(
            existing_item.positive_ratings(),
            existing_item.negative_ratings(),
            existing_item.total_ratings(),
            existing_item.new_cards_studied_today(),
            existing_item.phrase_cards_studied_today(),
        );
        existing_item.update_stats(update);
    } else {
        let mut item = DailyHistoryItem::new();
        for _ in 0..count {
            item.increment_new_cards_studied();
        }
        let update = stats.to_daily_update(0, 0, 0, item.new_cards_studied_today(), 0);
        item.update_stats(update);
        lesson_history.push(item);
    }
}

pub(crate) fn recalculate_daily_stats(
    study_cards: &HashMap<Ulid, StudyCard>,
    lesson_history: &mut Vec<DailyHistoryItem>,
) {
    let stats = match ComputedStats::compute(study_cards) {
        Some(s) => s,
        None => return,
    };

    let today = Utc::now().date_naive();
    // Ratings are recomputed from each card's last_rating when it was reviewed
    // today, rather than iterating an array of individual review logs (which
    // no longer exists after the MemoryHistory denormalization). This is exact
    // for single-device operation and for cross-device merge where both devices
    // reviewed different cards; cards reviewed on both devices contribute one
    // rating (the later one via LWW), which matches the G-Set union semantics
    // for the common case.
    let (positive, negative, total) = study_cards
        .values()
        .filter(|card| !matches!(card.card(), Card::Phrase(_)))
        .filter(|card| {
            card.memory()
                .last_review_date()
                .is_some_and(|d| d.date_naive() == today)
        })
        .fold((0usize, 0usize, 0usize), |(pos, neg, tot), card| {
            let rating = card.memory().last_rating();
            match rating {
                Some(Rating::Easy) | Some(Rating::Good) => (pos + 1, neg, tot + 1),
                Some(Rating::Hard) | Some(Rating::Again) => (pos, neg + 1, tot + 1),
                None => (pos, neg, tot + 1),
            }
        });

    let preserved_new_cards = lesson_history
        .iter()
        .rev()
        .find(|item| item.timestamp().date_naive() == today)
        .map(|item| item.new_cards_studied_today())
        .unwrap_or(0);

    let preserved_phrase_cards = lesson_history
        .iter()
        .rev()
        .find(|item| item.timestamp().date_naive() == today)
        .map(|item| item.phrase_cards_studied_today())
        .unwrap_or(0);

    let update = stats.to_daily_update(
        positive,
        negative,
        total,
        preserved_new_cards,
        preserved_phrase_cards,
    );

    if let Some(existing_item) = lesson_history
        .iter_mut()
        .find(|item| item.timestamp().date_naive() == today)
    {
        existing_item.update_stats(update);
    } else {
        let mut item = DailyHistoryItem::new();
        item.update_stats(update);
        lesson_history.push(item);
    }
}
