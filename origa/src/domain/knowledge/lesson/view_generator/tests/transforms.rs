use super::*;
use crate::domain::knowledge::lesson::types::LessonCardView;
use crate::domain::knowledge::lesson::view_generator::transforms;
use crate::use_cases::init_real_dictionaries;
use rand::{SeedableRng, rngs::StdRng};

#[test]
fn apply_reversed_uses_english_translation_for_english_locale() {
    init_real_dictionaries();
    let card = create_vocab_card("猫");

    let english_text = answer_text(&card, NativeLanguage::English);
    let russian_text = answer_text(&card, NativeLanguage::Russian);
    assert_ne!(
        english_text, russian_text,
        "test fixture: EN and RU translations must differ"
    );

    let view = transforms::apply_reversed(&card, &NativeLanguage::English);

    let reversed_word = match view {
        LessonCardView::Reversed(Card::Vocabulary(reversed)) => reversed.word().text().to_string(),
        other => panic!("Expected Reversed, got {other:?}"),
    };

    assert_eq!(
        reversed_word, english_text,
        "reversed card question must be the ENGLISH translation"
    );
    assert_ne!(
        reversed_word, russian_text,
        "regression guard: reversed card must not use the Russian translation"
    );
}

#[test]
fn apply_grammar_mutated_uses_english_description_for_english_locale() {
    init_real_dictionaries();

    let vocab = VocabularyCard::from_known_word("食べる", &NativeLanguage::English).expect(
        "test fixture: 食べる must have an English translation for the grammar-mutated EN path",
    );
    let card = Card::Vocabulary(vocab);

    let rule_id = ulid::Ulid::from_string("01G00000000000000024000000").expect("Invalid ULID");
    let grammar_card = GrammarRuleCard::new(rule_id).expect("verb grammar rule must exist");

    let desc_en = short_description_text(&grammar_card, NativeLanguage::English);
    let desc_ru = short_description_text(&grammar_card, NativeLanguage::Russian);
    assert_ne!(
        desc_en, desc_ru,
        "test fixture: grammar rule EN and RU short descriptions must differ"
    );

    let mut rng = StdRng::seed_from_u64(11);
    let view = transforms::apply_grammar_mutated(
        &card,
        std::slice::from_ref(&grammar_card),
        &mut rng,
        &NativeLanguage::English,
    );

    let actual_description = match view {
        LessonCardView::GrammarMutated { grammar_info, .. } => {
            grammar_info.description().to_string()
        },
        other => panic!("Expected GrammarMutated, got {other:?}"),
    };

    assert_eq!(
        actual_description, desc_en,
        "grammar description must be the ENGLISH short description"
    );
    assert_ne!(
        actual_description, desc_ru,
        "regression guard: grammar description must not be the Russian short description"
    );
}

fn short_description_text(card: &GrammarRuleCard, lang: NativeLanguage) -> String {
    card.short_description(&lang)
        .expect("grammar rule must have a short description for fixture")
        .text_projection()
}
