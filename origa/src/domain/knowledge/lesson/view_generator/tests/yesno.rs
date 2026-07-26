use super::*;
use crate::domain::knowledge::KanjiCard;
use crate::domain::knowledge::lesson::types::{LessonCardView, YesNoCard};
use crate::domain::value_objects::NativeLanguage;
use crate::use_cases::init_real_dictionaries;
use rand::{SeedableRng, rngs::StdRng};
use rstest::rstest;

fn create_vocab_card_with_word(word: &str) -> Card {
    Card::Vocabulary(VocabularyCard::new(
        Question::new(word.to_string()).unwrap(),
    ))
}

fn create_yesno_card(is_correct: bool) -> YesNoCard {
    let card = create_vocab_card_with_word("テスト");
    YesNoCard::new(card, "テスト".to_string(), "тест".to_string(), is_correct)
}

#[rstest]
#[case::correct_yes(true, true, true)]
#[case::false_no(false, false, true)]
#[case::wrong_yes(false, true, false)]
#[case::wrong_no(true, false, false)]
fn check_answer_compares_user_answer_to_card_correctness(
    #[case] card_is_correct: bool,
    #[case] answer: bool,
    #[case] expected: bool,
) {
    let yesno = create_yesno_card(card_is_correct);
    assert_eq!(yesno.check_answer(answer), expected);
}

#[rstest]
#[case::seed_42(42)]
#[case::seed_123(123)]
fn generate_yesno_produces_statement_with_non_empty_text(#[case] seed: u64) {
    init_real_dictionaries();

    let vocab_words = ["猫", "犬", "鳥", "魚"];
    let cards: Vec<Card> = vocab_words
        .iter()
        .map(|w| create_vocab_card_with_word(w))
        .collect();

    let mut rng = StdRng::seed_from_u64(seed);
    let result = generation::generate_yesno(
        cards[0].clone(),
        &cards[1..],
        &NativeLanguage::Russian,
        &mut rng,
    );

    assert!(result.is_ok());
    match result.unwrap() {
        LessonCardView::YesNo(yesno) => assert!(!yesno.statement().is_empty()),
        _ => panic!("Expected YesNo view"),
    }
}

#[test]
fn generate_yesno_falls_back_to_normal_when_no_distractors() {
    init_real_dictionaries();

    let card = create_vocab_card_with_word("猫");
    let empty_cards: Vec<Card> = vec![];

    let mut rng = StdRng::seed_from_u64(42);
    let result = generation::generate_yesno(
        card.clone(),
        &empty_cards,
        &NativeLanguage::Russian,
        &mut rng,
    );

    assert!(result.is_ok());
    match result.unwrap() {
        LessonCardView::Normal(returned_card) => {
            assert_eq!(returned_card, card);
        },
        _ => panic!("Expected Normal fallback when no distractors available"),
    }
}

#[test]
fn generate_yesno_produces_yesno_in_most_cases() {
    init_real_dictionaries();

    let vocab_words = ["猫", "犬", "鳥", "魚", "馬", "牛", "羊", "豚"];
    let cards: Vec<Card> = vocab_words
        .iter()
        .map(|w| create_vocab_card_with_word(w))
        .collect();

    let iterations = 1000;
    let mut yesno_count = 0;

    for seed in 0..iterations {
        let mut rng = StdRng::seed_from_u64(seed);
        let result = generation::generate_yesno(
            cards[0].clone(),
            &cards[1..],
            &NativeLanguage::Russian,
            &mut rng,
        );

        if let Ok(LessonCardView::YesNo(_)) = result {
            yesno_count += 1;
        }
    }

    let ratio = yesno_count as f32 / iterations as f32;
    assert!(ratio > 0.95, "YesNo generation ratio too low: {ratio}");
}

#[test]
fn generate_yesno_balances_correct_and_incorrect_statements() {
    init_real_dictionaries();

    let vocab_words = ["猫", "犬", "鳥", "魚"];
    let cards: Vec<Card> = vocab_words
        .iter()
        .map(|w| create_vocab_card_with_word(w))
        .collect();

    let iterations = 1000;
    let mut correct_count = 0;
    let mut incorrect_count = 0;

    for seed in 0..iterations {
        let mut rng = StdRng::seed_from_u64(seed);
        let result = generation::generate_yesno(
            cards[0].clone(),
            &cards[1..],
            &NativeLanguage::Russian,
            &mut rng,
        );

        if let Ok(LessonCardView::YesNo(yesno)) = result {
            if yesno.is_correct() {
                correct_count += 1;
            } else {
                incorrect_count += 1;
            }
        }
    }

    let correct_ratio = correct_count as f32 / iterations as f32;
    let incorrect_ratio = incorrect_count as f32 / iterations as f32;

    assert!(
        (0.45..=0.55).contains(&correct_ratio),
        "is_correct ratio should be ~50%, got {correct_ratio}"
    );
    assert!(
        (0.45..=0.55).contains(&incorrect_ratio),
        "is_incorrect ratio should be ~50%, got {incorrect_ratio}"
    );
}

#[test]
fn generate_yesno_kanji_with_distractors() {
    init_real_dictionaries();
    let cards: Vec<Card> = ["日", "月", "水", "火"]
        .iter()
        .map(|k| Card::Kanji(KanjiCard::new_test(k.to_string())))
        .collect();

    let mut rng = StdRng::seed_from_u64(42);
    let result = generation::generate_yesno(
        cards[0].clone(),
        &cards[1..],
        &NativeLanguage::Russian,
        &mut rng,
    );

    match result.expect("should succeed") {
        LessonCardView::YesNo(yn) => {
            assert!(!yn.statement().is_empty());
        },
        other => panic!("Expected YesNo for kanji, got {:?}", other),
    }
}

#[test]
fn generate_yesno_kanji_fallback_no_distractors() {
    init_real_dictionaries();
    let card = Card::Kanji(KanjiCard::new_test("日".to_string()));
    let mut rng = StdRng::seed_from_u64(42);
    let result = generation::generate_yesno(card.clone(), &[], &NativeLanguage::Russian, &mut rng);
    match result.unwrap() {
        LessonCardView::Normal(c) => assert_eq!(c, card),
        other => panic!("Expected Normal fallback, got {:?}", other),
    }
}

#[test]
fn generate_yesno_stores_word_and_statement_separately() {
    init_real_dictionaries();

    let vocab_words = ["猫", "犬", "鳥", "魚"];
    let cards: Vec<Card> = vocab_words
        .iter()
        .map(|w| create_vocab_card_with_word(w))
        .collect();

    let mut rng = StdRng::seed_from_u64(42);
    let result = generation::generate_yesno(
        cards[0].clone(),
        &cards[1..],
        &NativeLanguage::Russian,
        &mut rng,
    );

    let yesno = match result.expect("should succeed") {
        LessonCardView::YesNo(yn) => yn,
        other => panic!("Expected YesNo, got {other:?}"),
    };

    let expected_word = cards[0]
        .question(&NativeLanguage::Russian)
        .expect("question")
        .text()
        .to_string();
    assert_eq!(yesno.word(), expected_word);
    assert!(
        !yesno.word().contains(" \n "),
        "word must not embed the legacy separator: {}",
        yesno.word()
    );
    assert!(
        !yesno.statement().is_empty(),
        "statement must carry the answer/distractor"
    );
}

#[test]
fn generate_yesno_uses_english_statement_for_english_locale() {
    init_real_dictionaries();

    let vocab_words = ["猫", "犬", "鳥", "魚"];
    let cards: Vec<Card> = vocab_words
        .iter()
        .map(|w| create_vocab_card_with_word(w))
        .collect();

    let english_texts: Vec<String> = cards
        .iter()
        .map(|c| answer_text(c, NativeLanguage::English))
        .collect();
    let russian_texts: Vec<String> = cards
        .iter()
        .map(|c| answer_text(c, NativeLanguage::Russian))
        .collect();

    let mut rng = StdRng::seed_from_u64(7);
    let result = generation::generate_yesno(
        cards[0].clone(),
        &cards[1..],
        &NativeLanguage::English,
        &mut rng,
    );

    let yesno = match result.expect("English-locale yesno must be generated") {
        LessonCardView::YesNo(yn) => yn,
        other => panic!("Expected YesNo, got {other:?}"),
    };

    assert!(
        english_texts.iter().any(|t| t == yesno.statement()),
        "statement must be an ENGLISH translation, got '{}'",
        yesno.statement()
    );
    assert!(
        !russian_texts.iter().any(|t| t == yesno.statement()),
        "regression guard: statement must not be a Russian translation, got '{}'",
        yesno.statement()
    );
}
