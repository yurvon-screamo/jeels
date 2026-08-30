//! Сидирование первого ревью при закрытии руки.
//!
//! Тренировочные ответы режим знакомства не рейтингует; единственный момент
//! планирования — закрытие руки: каждой карте создаётся начальное состояние
//! памяти так, чтобы завтрашнее ревью было обычным молодым `Review`-картой.
//!
//! Контракт (docs/acquaintance-mode.md §9.2): после сидирования карта
//! `!is_new()`, `next_review_date == first_due`, `!is_known_card()`.

use chrono::{DateTime, Utc};

use crate::domain::OrigaError;
use crate::domain::memory::{CardState, Difficulty, MemoryHistory, MemoryState, Stability};

/// Стабильность сидирования: заведомо ниже порога known-карты (21.0), чтобы
/// карта не считалась выученной до первого настоящего ревью.
const SEED_STABILITY: f64 = 3.0;

/// Средняя сложность: ниже порога high-difficulty (7.0).
const SEED_DIFFICULTY: f64 = 5.0;

/// Создаёт карте начальное состояние памяти с первым ревью в `first_due`.
///
/// В отличие от рейтингового пути (`apply_review`) счётчики повторений,
/// `last_review_date` и `last_rating` не трогаются: тренировка не является
/// ревью. Завтрашний рейтинг через штатный `rate_memory` эволюционирует
/// состояние как у молодой Review-карты.
pub fn seed_first_review(
    history: &mut MemoryHistory,
    first_due: DateTime<Utc>,
) -> Result<(), OrigaError> {
    history.seed(build_seeded_memory_state(first_due)?);
    Ok(())
}

/// Создаёт начальное состояние памяти для карты, закрывшей руку знакомства:
/// значения подобраны так, чтобы карта не считалась known/high-difficulty, а
/// завтрашний рейтинг эволюционировал её как обычную молодую `Review`-карту
/// (см. тесты ниже).
pub fn build_seeded_memory_state(first_due: DateTime<Utc>) -> Result<MemoryState, OrigaError> {
    Ok(MemoryState::with_card_state(
        Stability::new(SEED_STABILITY)?,
        Difficulty::new(SEED_DIFFICULTY)?,
        first_due,
        CardState::Review,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::memory::Rating;
    use crate::domain::srs::{RateMode, rate_memory};
    use chrono::Duration;

    const DAY: Duration = Duration::days(1);
    /// Потолок разумного интервала для молодой карты после одного успеха.
    const INTERVAL_CEILING_DAYS: i64 = 60;

    fn seeded_history(first_due: DateTime<Utc>) -> MemoryHistory {
        let mut history = MemoryHistory::new();
        seed_first_review(&mut history, first_due).unwrap();
        history
    }

    #[test]
    fn seeded_card_is_not_new_due_tomorrow_and_not_known() {
        // Arrange
        let now = Utc::now();
        let first_due = now + DAY;

        // Act
        let history = seeded_history(first_due);

        // Assert
        assert!(!history.is_new(), "seeded card must leave the New state");
        assert_eq!(history.next_review_date(), Some(&first_due));
        assert!(
            !history.is_known_card(),
            "seed stability is far below the known threshold"
        );
        assert!(
            !history.is_high_difficulty(),
            "seed difficulty is below the high-difficulty threshold"
        );
    }

    #[test]
    fn seeding_does_not_touch_review_counters_or_last_review() {
        // Arrange
        let mut history = MemoryHistory::new();

        // Act
        seed_first_review(&mut history, Utc::now() + DAY).unwrap();

        // Assert: тренировка не является ревью — журнал чист
        assert_eq!(history.reps(), 0);
        assert_eq!(history.lapses(), 0);
        assert_eq!(history.good_review_count(), 0);
        assert_eq!(history.easy_review_count(), 0);
        assert_eq!(history.last_review_date(), None);
        assert_eq!(history.last_rating(), None);
    }

    #[test]
    fn good_rating_on_seeded_card_returns_interval_of_at_least_one_day() {
        // Arrange
        let before = Utc::now();
        let history = seeded_history(before + DAY);

        // Act
        let next = rate_memory(RateMode::StandardLesson, Rating::Good, &history).unwrap();
        let interval = next.next_review_date().signed_duration_since(before);

        // Assert: нижняя граница против минутного learning-интервала,
        // верхняя — против необоснованного прыжка на месяцы
        assert!(
            interval >= DAY,
            "interval after Good must be at least 1 day, got {:?}",
            interval
        );
        assert!(
            interval <= Duration::days(INTERVAL_CEILING_DAYS),
            "interval after Good must stay within {} days for a young card, got {:?}",
            INTERVAL_CEILING_DAYS,
            interval
        );
    }

    #[test]
    fn good_rating_on_seeded_card_stays_review() {
        // Arrange
        let history = seeded_history(Utc::now() + DAY);

        // Act
        let next = rate_memory(RateMode::StandardLesson, Rating::Good, &history).unwrap();

        // Assert
        assert_eq!(
            next.card_state(),
            CardState::Review,
            "Good on a seeded Review card must not downgrade to Learning"
        );
    }

    #[test]
    fn card_remains_not_known_after_first_good_review() {
        // Arrange
        let history = seeded_history(Utc::now() + DAY);

        // Act
        let next = rate_memory(RateMode::StandardLesson, Rating::Good, &history).unwrap();

        // Assert: один успех на молодой карте не дотягивает до known-порога
        assert!(next.stability().value() < 21.0);
    }
}
