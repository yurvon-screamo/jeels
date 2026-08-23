//! Тесты тренировки: подсчёт успехов, провалы и заморозка закрывших
//! критерий карт (правило «Тренировка» docs/acquaintance-mode.md).

use crate::domain::CardType;
use crate::domain::acquaintance::{AcquaintanceHand, AcquaintanceSubphase, AnswerOutcome};
use ulid::Ulid;

fn ids<const N: usize>() -> [Ulid; N] {
    std::array::from_fn(|_| Ulid::new())
}

#[test]
fn success_counts_forward_progress_until_criterion() {
    // Arrange: одно слово
    let [word] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![(word, CardType::Vocabulary, 0, 0)],
        Some(AcquaintanceSubphase::Forward),
    );

    // Act / Assert: 1-й и 2-й успехи считаются, третий закрывает подфазу —
    // но рука из одного слова после смены подфазы продолжает с Reverse,
    // поэтому исход третьего ответа — SubphaseAdvanced
    assert_eq!(
        hand.record_answer(word, true).unwrap(),
        AnswerOutcome::Counted { progress: 1 }
    );
    assert_eq!(
        hand.record_answer(word, true).unwrap(),
        AnswerOutcome::Counted { progress: 2 }
    );
    assert_eq!(
        hand.record_answer(word, true).unwrap(),
        AnswerOutcome::SubphaseAdvanced
    );
    assert_eq!(
        hand.subphase(),
        Some(AcquaintanceSubphase::Reverse),
        "все слова закрыли Forward — направление сменилось"
    );
}

#[test]
fn failed_answer_returns_failed_without_progress_change() {
    // Arrange
    let [word] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![(word, CardType::Vocabulary, 1, 0)],
        Some(AcquaintanceSubphase::Forward),
    );

    // Act
    let outcome = hand.record_answer(word, false).unwrap();

    // Assert
    assert_eq!(outcome, AnswerOutcome::Failed);
    assert_eq!(
        hand.entry(word)
            .unwrap()
            .progress_in(Some(AcquaintanceSubphase::Forward)),
        1,
        "провал не меняет счётчик"
    );
}

#[test]
fn success_beyond_criterion_is_frozen() {
    // Arrange: несловесная карта уже набрала критерий
    let [grammar] = ids();
    let mut hand = AcquaintanceHand::new_test(vec![(grammar, CardType::Grammar, 3, 0)], None);

    // Act
    let outcome = hand.record_answer(grammar, true).unwrap();

    // Assert: критерий общий для несловесных карт — прогресс заморожен
    assert_eq!(outcome, AnswerOutcome::ProgressFrozen);
}

#[test]
fn failed_answer_on_closed_card_is_frozen() {
    // Arrange: слово закрыло обе подфазы
    let [word] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![(word, CardType::Vocabulary, 3, 3)],
        Some(AcquaintanceSubphase::Reverse),
    );

    // Act
    let outcome = hand.record_answer(word, false).unwrap();

    // Assert: провал не «размораживает» и не регрессирует карту
    assert_eq!(outcome, AnswerOutcome::ProgressFrozen);
}

#[test]
fn word_closed_in_forward_is_frozen_while_neighbors_unclosed() {
    // Arrange: каноническая расстановка — А закрыла критерий текущей
    // подфазы, у Б остались успехи
    let [a, b] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![
            (a, CardType::Vocabulary, 3, 0),
            (b, CardType::Vocabulary, 1, 0),
        ],
        Some(AcquaintanceSubphase::Forward),
    );

    // Act
    let outcome = hand.record_answer(a, true).unwrap();

    // Assert: успех не двигает прогресс за пределы критерия подфазы,
    // пока соседи не закрыли её же
    assert_eq!(outcome, AnswerOutcome::ProgressFrozen);
    assert_eq!(
        hand.entry(a)
            .unwrap()
            .progress_in(Some(AcquaintanceSubphase::Forward)),
        3
    );
}
