use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use ulid::Ulid;

use crate::domain::{Card, JapaneseChar, OrigaError, StudyCard, tokenize_text};

pub struct ScoreContentResult {
    unknown_words: Vec<String>,
    unknown_kanji: Vec<String>,
    known_words: Vec<String>,
    known_kanji: Vec<String>,
}

impl ScoreContentResult {
    pub fn known_words(&self) -> &[String] {
        &self.known_words
    }

    pub fn unknown_words(&self) -> &[String] {
        &self.unknown_words
    }

    pub fn known_kanji(&self) -> &[String] {
        &self.known_kanji
    }

    pub fn unknown_kanji(&self) -> &[String] {
        &self.unknown_kanji
    }
}

fn classify_items<T: Eq + Hash>(
    items: impl IntoIterator<Item = T>,
    cards: &HashMap<Ulid, StudyCard>,
    is_known: impl Fn(&T, &StudyCard) -> bool,
) -> (Vec<T>, Vec<T>) {
    let mut known = vec![];
    let mut unknown = vec![];
    for item in items {
        if cards.values().any(|c| is_known(&item, c)) {
            known.push(item);
        } else {
            unknown.push(item);
        }
    }
    (known, unknown)
}

pub fn score_content(
    content: &str,
    cards: &HashMap<Ulid, StudyCard>,
) -> Result<ScoreContentResult, OrigaError> {
    let unique_kanji: HashSet<char> = content.chars().filter(|c| c.is_kanji()).collect();
    let (known_kanji, unknown_kanji) =
        classify_items(unique_kanji, cards, |kanji, card| match card.card() {
            Card::Kanji(c) => {
                c.kanji().text() == kanji.to_string() && card.memory().is_known_card()
            },
            _ => false,
        });

    let unique_words: HashSet<String> = tokenize_text(content)?
        .iter()
        .filter(|token| token.part_of_speech().is_vocabulary_word())
        .map(|token| token.orthographic_base_form().to_string())
        .collect();
    let (known_words, unknown_words) =
        classify_items(unique_words, cards, |word, card| match card.card() {
            Card::Vocabulary(c) => c.word().text() == *word && card.memory().is_known_card(),
            _ => false,
        });

    Ok(ScoreContentResult {
        known_kanji: known_kanji.into_iter().map(|c| c.to_string()).collect(),
        unknown_kanji: unknown_kanji.into_iter().map(|c| c.to_string()).collect(),
        known_words,
        unknown_words,
    })
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use std::collections::HashMap;
    use ulid::Ulid;

    use super::*;
    use crate::domain::{
        Card, Difficulty, KanjiCard, MemoryState, Rating, Stability, StudyCard, VocabularyCard,
        value_objects::Question,
    };
    use crate::use_cases::init_real_dictionaries;

    fn create_known_study_card_with_kanji(kanji: &str) -> StudyCard {
        let kanji_card = KanjiCard::new_test(kanji.to_string());
        let mut study_card = StudyCard::new(Card::Kanji(kanji_card));
        set_card_as_known(&mut study_card);
        study_card
    }

    fn create_unknown_study_card_with_kanji(kanji: &str) -> StudyCard {
        let kanji_card = KanjiCard::new_test(kanji.to_string());
        StudyCard::new(Card::Kanji(kanji_card))
    }

    fn create_known_study_card_with_vocab(word: &str) -> StudyCard {
        let question = Question::new(word.to_string()).unwrap();
        let vocab_card = VocabularyCard::new(question);
        let mut study_card = StudyCard::new(Card::Vocabulary(vocab_card));
        set_card_as_known(&mut study_card);
        study_card
    }

    fn create_unknown_study_card_with_vocab(word: &str) -> StudyCard {
        let question = Question::new(word.to_string()).unwrap();
        let vocab_card = VocabularyCard::new(question);
        StudyCard::new(Card::Vocabulary(vocab_card))
    }

    fn set_card_as_known(study_card: &mut StudyCard) {
        let stability = Stability::new(22.0).unwrap();
        let difficulty = Difficulty::new(3.0).unwrap();
        let next_review = Utc::now() + Duration::days(30);
        let memory_state = MemoryState::new(stability, difficulty, next_review);
        study_card.apply_review(memory_state, Rating::Good);
    }

    fn cards_to_hashmap(cards: Vec<StudyCard>) -> HashMap<Ulid, StudyCard> {
        cards
            .into_iter()
            .map(|card| (*card.card_id(), card))
            .collect()
    }

    #[test]
    fn score_content_returns_empty_for_plain_text() {
        init_real_dictionaries();

        let content = "Hello World";
        let cards = HashMap::new();

        let result = score_content(content, &cards).unwrap();

        assert!(result.known_words().is_empty());
        assert!(result.unknown_words().is_empty());
        assert!(result.known_kanji().is_empty());
        assert!(result.unknown_kanji().is_empty());
    }

    #[test]
    fn score_content_identifies_known_kanji() {
        init_real_dictionaries();

        let content = "日本語";
        let cards = cards_to_hashmap(vec![
            create_known_study_card_with_kanji("日"),
            create_known_study_card_with_kanji("本"),
        ]);

        let result = score_content(content, &cards).unwrap();

        assert!(result.known_kanji().contains(&"日".to_string()));
        assert!(result.known_kanji().contains(&"本".to_string()));
        assert!(!result.known_kanji().contains(&"語".to_string()));
    }

    #[test]
    fn score_content_identifies_unknown_kanji() {
        init_real_dictionaries();

        let content = "日本語";
        let cards = HashMap::new();

        let result = score_content(content, &cards).unwrap();

        assert!(result.unknown_kanji().contains(&"日".to_string()));
        assert!(result.unknown_kanji().contains(&"本".to_string()));
        assert!(result.unknown_kanji().contains(&"語".to_string()));
        assert!(result.known_kanji().is_empty());
    }

    #[test]
    fn score_content_mixed_kanji_knowledge() {
        init_real_dictionaries();

        let content = "日本語";
        let cards = cards_to_hashmap(vec![
            create_known_study_card_with_kanji("日"),
            create_known_study_card_with_kanji("本"),
        ]);

        let result = score_content(content, &cards).unwrap();

        assert!(result.known_kanji().contains(&"日".to_string()));
        assert!(result.known_kanji().contains(&"本".to_string()));
        assert!(result.unknown_kanji().contains(&"語".to_string()));
    }

    #[test]
    fn score_content_identifies_known_words() {
        init_real_dictionaries();

        let content = "猫と犬";
        let cards = cards_to_hashmap(vec![create_known_study_card_with_vocab("猫")]);

        let result = score_content(content, &cards).unwrap();

        assert!(result.known_words().contains(&"猫".to_string()));
    }

    #[test]
    fn score_content_identifies_unknown_words() {
        init_real_dictionaries();

        let content = "猫と犬";
        let cards = HashMap::new();

        let result = score_content(content, &cards).unwrap();

        assert!(result.unknown_words().iter().any(|w| w == "猫"));
    }

    #[test]
    fn score_content_mixed_word_knowledge() {
        init_real_dictionaries();

        let content = "猫と犬";
        let cards = cards_to_hashmap(vec![create_known_study_card_with_vocab("猫")]);

        let result = score_content(content, &cards).unwrap();

        assert!(result.known_words().contains(&"猫".to_string()));
        assert!(result.unknown_words().iter().any(|w| w == "犬"));
    }

    #[test]
    fn score_content_ignores_hiragana_for_kanji() {
        init_real_dictionaries();

        let content = "こんにちは";
        let cards = HashMap::new();

        let result = score_content(content, &cards).unwrap();

        assert!(result.known_kanji().is_empty());
        assert!(result.unknown_kanji().is_empty());
    }

    #[test]
    fn score_content_ignores_katakana_for_kanji() {
        init_real_dictionaries();

        let content = "コンニチハ";
        let cards = HashMap::new();

        let result = score_content(content, &cards).unwrap();

        assert!(result.known_kanji().is_empty());
        assert!(result.unknown_kanji().is_empty());
    }

    #[test]
    fn score_content_handles_empty_content() {
        init_real_dictionaries();

        let content = "";
        let cards = HashMap::new();

        let result = score_content(content, &cards).unwrap();

        assert!(result.known_words().is_empty());
        assert!(result.unknown_words().is_empty());
        assert!(result.known_kanji().is_empty());
        assert!(result.unknown_kanji().is_empty());
    }

    #[test]
    fn score_content_ignores_unknown_card_type_for_kanji() {
        init_real_dictionaries();

        let content = "日";
        let vocab_card = create_known_study_card_with_vocab("日");
        let cards = cards_to_hashmap(vec![vocab_card]);

        let result = score_content(content, &cards).unwrap();

        assert!(result.unknown_kanji().contains(&"日".to_string()));
        assert!(result.known_kanji().is_empty());
    }

    #[test]
    fn score_content_ignores_unknown_card_type_for_words() {
        init_real_dictionaries();

        let content = "猫";
        let kanji_card = create_known_study_card_with_kanji("猫");
        let cards = cards_to_hashmap(vec![kanji_card]);

        let result = score_content(content, &cards).unwrap();

        assert!(result.unknown_words().iter().any(|w| w == "猫"));
    }

    #[test]
    fn score_content_deduplicates_kanji() {
        init_real_dictionaries();

        let content = "日日日";
        let cards = HashMap::new();

        let result = score_content(content, &cards).unwrap();

        assert_eq!(result.unknown_kanji().len(), 1);
        assert!(result.unknown_kanji().contains(&"日".to_string()));
    }

    #[test]
    fn score_content_deduplicates_words() {
        init_real_dictionaries();

        let content = "猫猫猫";
        let cards = HashMap::new();

        let result = score_content(content, &cards).unwrap();

        let cat_count = result.unknown_words().iter().filter(|w| *w == "猫").count();
        assert_eq!(cat_count, 1);
    }

    #[test]
    fn score_content_unknown_kanji_card_not_counted_as_known() {
        init_real_dictionaries();

        let content = "日";
        let cards = cards_to_hashmap(vec![create_unknown_study_card_with_kanji("日")]);

        let result = score_content(content, &cards).unwrap();

        assert!(result.unknown_kanji().contains(&"日".to_string()));
        assert!(result.known_kanji().is_empty());
    }

    #[test]
    fn score_content_unknown_vocab_card_not_counted_as_known() {
        init_real_dictionaries();

        let content = "猫";
        let cards = cards_to_hashmap(vec![create_unknown_study_card_with_vocab("猫")]);

        let result = score_content(content, &cards).unwrap();

        assert!(result.unknown_words().iter().any(|w| w == "猫"));
        assert!(result.known_words().is_empty());
    }

    mod score_content_result_accessors {
        use super::*;

        #[test]
        fn known_words_returns_slice() {
            init_real_dictionaries();

            let content = "猫";
            let cards = cards_to_hashmap(vec![create_known_study_card_with_vocab("猫")]);

            let result = score_content(content, &cards).unwrap();

            assert!(!result.known_words().is_empty());
        }

        #[test]
        fn unknown_words_returns_slice() {
            init_real_dictionaries();

            let content = "猫";
            let cards = HashMap::new();

            let result = score_content(content, &cards).unwrap();

            assert!(!result.unknown_words().is_empty());
        }

        #[test]
        fn known_kanji_returns_slice() {
            init_real_dictionaries();

            let content = "日";
            let cards = cards_to_hashmap(vec![create_known_study_card_with_kanji("日")]);

            let result = score_content(content, &cards).unwrap();

            assert!(!result.known_kanji().is_empty());
        }

        #[test]
        fn unknown_kanji_returns_slice() {
            init_real_dictionaries();

            let content = "日";
            let cards = HashMap::new();

            let result = score_content(content, &cards).unwrap();

            assert!(!result.unknown_kanji().is_empty());
        }
    }
}
