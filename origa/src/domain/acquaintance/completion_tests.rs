//! Завершение руки: смена подфаз, HandCompleted, краевые случаи состава.

use crate::domain::acquaintance::{AcquaintanceHand, AcquaintanceSubphase, AnswerOutcome};
use crate::domain::{CardType, OrigaError};
use ulid::Ulid;

fn ids<const N: usize>() -> [Ulid; N] {
    std::array::from_fn(|_| Ulid::new())
}

#[test]
fn last_forward_success_advances_subphase_and_resets_visible_progress() {
    // Arrange: слово a закрыло Forward (3/3), слову b остался один успех
    let [a, b] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![
            (a, CardType::Vocabulary, 3, 0),
            (b, CardType::Vocabulary, 2, 0),
        ],
        Some(AcquaintanceSubphase::Forward),
    );
    // Act: последний успех Forward + смена направления на границе витка
    assert_eq!(
        hand.record_answer(b, true).unwrap(),
        AnswerOutcome::Counted { progress: 3 }
    );
    assert!(hand.advance_subphase_if_words_done());
    // Assert: подфаза сменилась, видимый прогресс слов обнулился
    assert_eq!(hand.subphase(), Some(AcquaintanceSubphase::Reverse));
    for id in [a, b] {
        assert_eq!(
            hand.entry(id)
                .unwrap()
                .progress_in(Some(AcquaintanceSubphase::Reverse)),
            0,
            "счётчики слов в новой подфазе начинаются с нуля"
        );
    }
}
#[test]
fn nonword_card_is_independent_of_word_subphases() {
    // Arrange: кандзи закрыл свой критерий ещё в Forward; слово — нет
    let [kanji, word] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![
            (kanji, CardType::Kanji, 3, 0),
            (word, CardType::Vocabulary, 1, 0),
        ],
        Some(AcquaintanceSubphase::Forward),
    );
    // Act / Assert: ответы по кандзи заморожены в любой подфазе
    assert_eq!(
        hand.record_answer(kanji, true).unwrap(),
        AnswerOutcome::ProgressFrozen
    );
}
#[test]
fn nonword_accumulates_progress_across_subphase_advance() {
    // Arrange: кандзи с 1 успехом в Forward, слово с 2 — сценарий
    // «несловесные копят единый счётчик во всех витках обеих подфаз»
    let [kanji, word] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![
            (kanji, CardType::Kanji, 1, 0),
            (word, CardType::Vocabulary, 2, 0),
        ],
        Some(AcquaintanceSubphase::Forward),
    );

    // Act: слово закрывает Forward за всех; смена направления — на границе
    assert_eq!(
        hand.record_answer(word, true).unwrap(),
        AnswerOutcome::Counted { progress: 3 }
    );
    assert!(hand.advance_subphase_if_words_done());

    // Act / Assert: успехи по кандзи в Reverse продолжают тот же счётчик
    // и завершают его критерий; дальше — заморозка
    assert_eq!(
        hand.record_answer(kanji, true).unwrap(),
        AnswerOutcome::Counted { progress: 2 }
    );
    assert_eq!(
        hand.record_answer(kanji, true).unwrap(),
        AnswerOutcome::Counted { progress: 3 }
    );
    assert_eq!(
        hand.record_answer(kanji, true).unwrap(),
        AnswerOutcome::ProgressFrozen
    );
}

#[test]
fn hand_completes_when_all_criteria_closed() {
    // Arrange: слово осталось с 2/3 в Reverse, кандзи уже закрыт
    let [kanji, word] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![
            (kanji, CardType::Kanji, 3, 0),
            (word, CardType::Vocabulary, 3, 2),
        ],
        Some(AcquaintanceSubphase::Reverse),
    );
    // Act: последний успешный ответ закрывает руку
    let outcome = hand.record_answer(word, true).unwrap();
    // Assert
    assert_eq!(outcome, AnswerOutcome::HandCompleted);
}
#[test]
fn hand_without_words_never_advances_and_completes_on_criteria() {
    // Arrange: кандзи + грамматика, у обоих 2 успеха
    let [kanji, grammar] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![
            (kanji, CardType::Kanji, 2, 0),
            (grammar, CardType::Grammar, 2, 0),
        ],
        None,
    );
    // Act / Assert: первый ответ считается, второй закрывает руку,
    // SubphaseAdvanced не возникает никогда
    assert_eq!(
        hand.record_answer(kanji, true).unwrap(),
        AnswerOutcome::Counted { progress: 3 }
    );
    assert_eq!(hand.subphase(), None);
    assert_eq!(
        hand.record_answer(grammar, true).unwrap(),
        AnswerOutcome::HandCompleted
    );
}
#[test]
fn single_word_hand_completes_after_both_subphases() {
    // Arrange: вырожденная рука из одной карты, 2/3 в Reverse
    let [word] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![(word, CardType::Vocabulary, 3, 2)],
        Some(AcquaintanceSubphase::Reverse),
    );
    // Act
    let outcome = hand.record_answer(word, true).unwrap();
    // Assert
    assert_eq!(outcome, AnswerOutcome::HandCompleted);
}
#[test]
fn unknown_card_id_returns_card_not_found() {
    // Arrange
    let [word] = ids();
    let stranger = Ulid::new();
    let mut hand = AcquaintanceHand::new_test(
        vec![(word, CardType::Vocabulary, 0, 0)],
        Some(AcquaintanceSubphase::Forward),
    );
    // Act
    let result = hand.record_answer(stranger, true);
    // Assert
    assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
}
