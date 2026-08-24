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

#[test]
fn retired_card_freezes_answers() {
    // Arrange: слово выведено из руки («Уже знаю»)
    let [word] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![(word, CardType::Vocabulary, 0, 0)],
        Some(AcquaintanceSubphase::Forward),
    );
    assert!(hand.retire_card(word));

    // Act / Assert: ответы заморожены независимо от исхода
    assert_eq!(
        hand.record_answer(word, true).unwrap(),
        AnswerOutcome::ProgressFrozen
    );
    assert_eq!(
        hand.record_answer(word, false).unwrap(),
        AnswerOutcome::ProgressFrozen
    );
}

#[test]
fn retired_word_does_not_block_subphase_advance() {
    // Arrange: два слова, второе выведено с незакрытым forward
    let [a, b] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![
            (a, CardType::Vocabulary, 2, 0),
            (b, CardType::Vocabulary, 0, 0),
        ],
        Some(AcquaintanceSubphase::Forward),
    );
    assert!(hand.retire_card(b));

    // Act: третий успех `a` закрывает forward — подфаза меняется, хотя `b`
    // свой критерий не выполнял никогда
    let outcome = hand.record_answer(a, true).unwrap();

    // Assert
    assert_eq!(outcome, AnswerOutcome::SubphaseAdvanced);
}

#[test]
fn retired_card_does_not_block_hand_completion() {
    // Arrange: слову `a` остался один reverse-успех; `b` выведена до
    // каких-либо успехов
    let [a, b] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![
            (a, CardType::Vocabulary, 3, 2),
            (b, CardType::Vocabulary, 0, 0),
        ],
        Some(AcquaintanceSubphase::Reverse),
    );
    assert!(hand.retire_card(b));

    // Act: успех `a` закрывает её критерий — все entries закрыты (`b`
    // выведена и считается закрытой автоматически)
    let outcome = hand.record_answer(a, true).unwrap();

    // Assert
    assert_eq!(outcome, AnswerOutcome::HandCompleted);
}

#[test]
fn retire_unknown_card_returns_false() {
    let mut hand = AcquaintanceHand::new_test(vec![(Ulid::new(), CardType::Kanji, 0, 0)], None);
    assert!(!hand.retire_card(Ulid::new()));
}
