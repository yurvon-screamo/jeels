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
    // Act / Assert: первый ответ считается, второй закрывает руку;
    // смены направления не возникает — в руках без слов подфаз нет
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

#[test]
fn offer_replacement_puts_new_card_in_retired_slot() {
    // Arrange: слово выведено, порядок [word(retired), kanji, grammar]
    let [word, kanji, grammar] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![
            (word, CardType::Vocabulary, 0, 0),
            (kanji, CardType::Kanji, 1, 0),
            (grammar, CardType::Grammar, 2, 0),
        ],
        Some(AcquaintanceSubphase::Forward),
    );
    assert!(hand.retire_card(word));
    let new_word = Ulid::new();

    // Act
    assert!(
        hand.offer_replacement(word, new_word, CardType::Vocabulary)
            .is_ok()
    );

    // Assert: новая карта занимает слот выбывшей, размер руки сохраняется
    let order = hand.presentation_order();
    assert_eq!(order[0], new_word, "замена на месте выбывшей");
    assert_eq!(order.len(), 3, "рука не уменьшается и не растёт");
    assert_eq!(hand.len(), 3);
    assert_eq!(
        hand.entry(new_word).map(|e| e.card_type()),
        Some(CardType::Vocabulary)
    );
}

#[test]
fn offer_replacement_new_card_trains_from_zero() {
    // Arrange
    let [word, new_word] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![(word, CardType::Vocabulary, 3, 2)],
        Some(AcquaintanceSubphase::Forward),
    );
    assert!(hand.retire_card(word));

    // Act
    hand.offer_replacement(word, new_word, CardType::Vocabulary)
        .unwrap();

    // Assert: прогресс замены с нуля, ответы считаются
    assert_eq!(
        hand.record_answer(new_word, true).unwrap(),
        AnswerOutcome::Counted { progress: 1 }
    );
    // Рука не завершена, пока замена не закрыла критерии
    assert!(!hand.entry(new_word).unwrap().criterion_met(None));
}

#[test]
fn offer_replacement_unknown_retired_or_duplicate_new_errors() {
    // Arrange
    let [word, other] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![(word, CardType::Vocabulary, 0, 0)],
        Some(AcquaintanceSubphase::Forward),
    );
    assert!(hand.retire_card(word));

    // Act / Assert: замена неизвестной карты
    assert!(
        hand.offer_replacement(Ulid::new(), other, CardType::Vocabulary)
            .is_err()
    );
    // Дубликат новой карты (уже в руке)
    assert!(
        hand.offer_replacement(word, other, CardType::Vocabulary)
            .is_ok()
    );
    assert!(
        hand.offer_replacement(word, other, CardType::Vocabulary)
            .is_err()
    );
    // Фразы в руке быть не может
    let mut phrase_hand =
        AcquaintanceHand::new_test(vec![(word, CardType::Vocabulary, 0, 0)], None);
    assert!(phrase_hand.retire_card(word));
    assert!(
        phrase_hand
            .offer_replacement(word, Ulid::new(), CardType::Phrase)
            .is_err()
    );
}

#[test]
fn offer_replacement_active_retired_still_completes_via_criterion() {
    // Замена не ломает завершение: рука закрывается, когда замена закрыла
    // критерий (retired-записи больше нет — двойника в entries нет).
    let [word, new_word] = ids();
    let mut hand = AcquaintanceHand::new_test(vec![(word, CardType::Kanji, 3, 0)], None);
    assert!(hand.retire_card(word));
    hand.offer_replacement(word, new_word, CardType::Kanji)
        .unwrap();
    for _ in 0..2 {
        hand.record_answer(new_word, true).unwrap();
    }
    assert!(matches!(
        hand.record_answer(new_word, true).unwrap(),
        AnswerOutcome::HandCompleted
    ));
}
