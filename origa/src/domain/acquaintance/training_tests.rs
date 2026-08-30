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

    // Act / Assert: все три успеха считаются; смена направления — не
    // побочный эффект ответа, а явный вызов на границе витка
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
        AnswerOutcome::Counted { progress: 3 }
    );
    assert_eq!(
        hand.subphase(),
        Some(AcquaintanceSubphase::Forward),
        "закрытие forward само по себе направление не меняет"
    );
    assert!(hand.advance_subphase_if_words_done());
    assert_eq!(hand.subphase(), Some(AcquaintanceSubphase::Reverse));
}

#[test]
fn failed_answer_resets_progress_in_current_subphase() {
    // Arrange: слово набрало 2/3 forward и 1/3 reverse
    let [word] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![(word, CardType::Vocabulary, 2, 1)],
        Some(AcquaintanceSubphase::Reverse),
    );

    // Act: ошибка в Reverse
    let outcome = hand.record_answer(word, false).unwrap();

    // Assert: исход Failed, reverse-прогресс сброшен в ноль (шкала
    // в полосе честно уходит в 0), forward не тронут.
    assert!(matches!(outcome, AnswerOutcome::Failed));
    assert_eq!(
        hand.entry(word)
            .unwrap()
            .progress_in(Some(AcquaintanceSubphase::Reverse)),
        0,
        "ошибка сбрасывает прогресс текущей подфазы"
    );
    assert_eq!(
        hand.entry(word)
            .unwrap()
            .progress_in(Some(AcquaintanceSubphase::Forward)),
        2,
        "прогресс другой подфазы не трогается"
    );
}

#[test]
fn failed_answer_in_forward_resets_forward_progress() {
    // Arrange
    let [word] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![(word, CardType::Vocabulary, 2, 0)],
        Some(AcquaintanceSubphase::Forward),
    );

    // Act
    let outcome = hand.record_answer(word, false).unwrap();

    // Assert: набор forward-критерия начинается заново.
    assert!(matches!(outcome, AnswerOutcome::Failed));
    assert_eq!(
        hand.entry(word)
            .unwrap()
            .progress_in(Some(AcquaintanceSubphase::Forward)),
        0,
        "ошибка в Forward сбрасывает forward-прогресс"
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
fn failed_answer_on_fully_closed_word_resets_current_subphase() {
    // Arrange: слово закрыло обе подфазы
    let [word] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![(word, CardType::Vocabulary, 3, 3)],
        Some(AcquaintanceSubphase::Reverse),
    );

    // Act
    let outcome = hand.record_answer(word, false).unwrap();

    // Assert: «Не помню» переоткрывает даже закрывшую критерий карту —
    // шкала текущей подфазы уходит в ноль, закрытая ранее подфаза
    // не регрессирует
    assert!(matches!(outcome, AnswerOutcome::Failed));
    assert_eq!(
        hand.entry(word)
            .unwrap()
            .progress_in(Some(AcquaintanceSubphase::Reverse)),
        0,
        "провал закрывшей критерий карты сбрасывает шкалу текущей подфазы"
    );
    assert_eq!(
        hand.entry(word)
            .unwrap()
            .progress_in(Some(AcquaintanceSubphase::Forward)),
        3,
        "закрытая предыдущая подфаза не регрессирует"
    );
}

#[test]
fn failed_answer_on_word_closed_in_forward_reopens_it() {
    // Arrange: А закрыла forward-критерий, соседняя Б — ещё нет
    let [a, b] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![
            (a, CardType::Vocabulary, 3, 0),
            (b, CardType::Vocabulary, 1, 0),
        ],
        Some(AcquaintanceSubphase::Forward),
    );

    // Act: «Не помню» по закрывшей критерий карте
    let outcome = hand.record_answer(a, false).unwrap();

    // Assert: карта переоткрыта — набор критерия подфазы начинается
    // заново, соседка не затронута (баг-репорт: сброс не срабатывал
    // для достигших порога карт)
    assert!(matches!(outcome, AnswerOutcome::Failed));
    assert_eq!(
        hand.entry(a)
            .unwrap()
            .progress_in(Some(AcquaintanceSubphase::Forward)),
        0,
        "шкала закрывшей критерий карты сброшена в ноль"
    );
    assert_eq!(
        hand.entry(b)
            .unwrap()
            .progress_in(Some(AcquaintanceSubphase::Forward)),
        1,
        "провал по одной карте не трогает соседей"
    );
}

#[test]
fn failed_answer_on_nonword_closed_card_resets_shared_criterion() {
    // Arrange: кандзи закрыл общий критерий (рука без подфаз)
    let [kanji] = ids();
    let mut hand = AcquaintanceHand::new_test(vec![(kanji, CardType::Kanji, 3, 0)], None);

    // Act
    let outcome = hand.record_answer(kanji, false).unwrap();

    // Assert: единый счётчик несловесной карты обнулён — карта
    // переоткрыта, рука снова ждёт её критерия
    assert!(matches!(outcome, AnswerOutcome::Failed));
    assert_eq!(
        hand.entry(kanji).unwrap().progress_in(None),
        0,
        "общий критерий несловесной карты сбрасывается целиком"
    );
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

    // Act: третий успех `a` закрывает forward — на границе витка подфаза
    // меняется, хотя `b` свой критерий не выполнял никогда
    assert_eq!(
        hand.record_answer(a, true).unwrap(),
        AnswerOutcome::Counted { progress: 3 }
    );

    // Assert
    assert!(hand.advance_subphase_if_words_done());
}

#[test]
fn subphase_advances_only_at_rotation_boundary() {
    // Arrange: два слова; в середине витка первое закрывает forward
    let [a, b] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![
            (a, CardType::Vocabulary, 2, 0),
            (b, CardType::Vocabulary, 1, 0),
        ],
        Some(AcquaintanceSubphase::Forward),
    );

    // Act: `a` закрывает свой forward посреди витка — `b` ещё отвечается
    let outcome = hand.record_answer(a, true).unwrap();

    // Assert: направление стабильно до конца витка
    assert_eq!(outcome, AnswerOutcome::Counted { progress: 3 });
    assert_eq!(hand.subphase(), Some(AcquaintanceSubphase::Forward));
    assert!(
        !hand.advance_subphase_if_words_done(),
        "b ещё не закрыл forward"
    );

    // Act: `b` добирает критерий — на границе витка направление меняется
    hand.record_answer(b, true).unwrap();
    hand.record_answer(b, true).unwrap();

    // Assert
    assert!(hand.advance_subphase_if_words_done());
    assert_eq!(hand.subphase(), Some(AcquaintanceSubphase::Reverse));
}

#[rstest::rstest]
#[case::forward_not_all_closed(
    vec![(CardType::Vocabulary, 3, 0), (CardType::Vocabulary, 2, 0)],
    false
)]
#[case::forward_all_closed(
    vec![(CardType::Vocabulary, 3, 0), (CardType::Vocabulary, 3, 0)],
    true
)]
#[case::words_mixed_with_frozen_kanji(
    vec![(CardType::Kanji, 3, 0), (CardType::Vocabulary, 3, 0)],
    true
)]
fn advance_subphase_requires_every_active_word_closed_in_forward(
    #[case] words: Vec<(CardType, u8, u8)>,
    #[case] expected: bool,
) {
    // Arrange
    let ids: Vec<Ulid> = (0..words.len()).map(|_| Ulid::new()).collect();
    let entries: Vec<(Ulid, CardType, u8, u8)> = ids
        .iter()
        .zip(words)
        .map(|(id, (card_type, forward, reverse))| (*id, card_type, forward, reverse))
        .collect();
    let mut hand = AcquaintanceHand::new_test(entries, Some(AcquaintanceSubphase::Forward));

    // Act
    let advanced = hand.advance_subphase_if_words_done();

    // Assert
    assert_eq!(advanced, expected);
}

#[test]
fn advance_subphase_in_reverse_returns_false() {
    // Arrange: слова в Reverse (третьего направления не существует)
    let [word] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![(word, CardType::Vocabulary, 3, 3)],
        Some(AcquaintanceSubphase::Reverse),
    );

    // Act / Assert
    assert!(!hand.advance_subphase_if_words_done());
    assert_eq!(hand.subphase(), Some(AcquaintanceSubphase::Reverse));
}

#[test]
fn advance_subphase_without_words_returns_false() {
    // Arrange: рука без слов — подфаз нет
    let [kanji, grammar] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![
            (kanji, CardType::Kanji, 3, 0),
            (grammar, CardType::Grammar, 3, 0),
        ],
        None,
    );

    // Act / Assert
    assert!(!hand.advance_subphase_if_words_done());
    assert_eq!(hand.subphase(), None);
}

#[test]
fn advance_subphase_when_all_words_retired_returns_false() {
    // Arrange: оба слова выведены — переключать направление не на кого,
    // рука завершается общим критерием (retired = закрытые)
    let [a, b, kanji] = ids();
    let mut hand = AcquaintanceHand::new_test(
        vec![
            (a, CardType::Vocabulary, 0, 0),
            (b, CardType::Vocabulary, 0, 0),
            (kanji, CardType::Kanji, 2, 0),
        ],
        Some(AcquaintanceSubphase::Forward),
    );
    assert!(hand.retire_card(a));
    assert!(hand.retire_card(b));

    // Act / Assert: тег направления не должен ложно показать «РУС→ЯП»,
    // пока кандзи добирает общий критерий
    assert!(!hand.advance_subphase_if_words_done());
    assert_eq!(hand.subphase(), Some(AcquaintanceSubphase::Forward));
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
