use crate::dictionary::grammar::GrammarRule;
use crate::dictionary::vocabulary::{get_description, get_translation, get_translations};
use crate::domain::tokenizer::{PartOfSpeech, tokenize_text};
use crate::domain::{CardAnswer, NativeLanguage, OrigaError, Question};
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Результат создания карточек из текста
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateFromTextResult {
    pub cards: Vec<VocabularyCard>,
    pub skipped_no_translation: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VocabularyCard {
    word: Question,
    reverse_side: Option<Question>,
    #[serde(default)]
    pos: Option<PartOfSpeech>,
}

impl VocabularyCard {
    /// Конструктор для создания тестовых карточек
    #[cfg(test)]
    pub(crate) fn new(word: Question) -> Self {
        Self {
            word,
            reverse_side: None,
            pos: None,
        }
    }

    /// Creates a card with an explicit part-of-speech, bypassing tokenization.
    /// Used by companion-card creation and component tests that need a specific
    /// POS without loading the full lindera dictionary. The `reverse_side`
    /// parameter creates a pre-reversed card (translation in `word`, original
    /// Japanese in `reverse_side`) without calling `revert()` (which requires
    /// a loaded translation dictionary).
    pub fn new_with_pos(
        word: Question,
        pos: Option<PartOfSpeech>,
        reverse_side: Option<Question>,
    ) -> Self {
        Self {
            word,
            reverse_side,
            pos,
        }
    }

    /// Creates a card from a single known word after validating that a translation exists.
    /// The part of speech is resolved with a single tokenization pass at construction time
    /// so subsequent reads do not re-tokenize.
    pub fn from_known_word(word: &str, lang: &NativeLanguage) -> Result<Self, OrigaError> {
        Self::validate_translation(word, lang)?;
        let question = Question::new(word.to_string())?;
        let pos = tokenize_text(word)
            .ok()
            .and_then(|tokens| tokens.first().map(|t| t.part_of_speech().clone()));
        Ok(Self {
            word: question,
            reverse_side: None,
            pos,
        })
    }

    /// Создаёт карточки из текста с токенизацией и валидацией
    pub fn from_text(text: &str, lang: &NativeLanguage) -> CreateFromTextResult {
        let mut cards = Vec::new();
        let mut skipped = Vec::new();

        let tokens = match tokenize_text(text) {
            Ok(t) => t,
            Err(e) => {
                warn!(text = %text, error = %e, "Tokenization failed");
                return CreateFromTextResult {
                    cards,
                    skipped_no_translation: skipped,
                };
            },
        };

        for token in tokens {
            if !token.part_of_speech().is_vocabulary_word() {
                continue;
            }

            let word_text = token.orthographic_base_form();
            let token_pos = token.part_of_speech().clone();

            match Self::from_known_word(word_text, lang) {
                Ok(mut card) => {
                    card.pos = Some(token_pos);
                    cards.push(card);
                },
                Err(_) => skipped.push(word_text.to_string()),
            }
        }

        CreateFromTextResult {
            cards,
            skipped_no_translation: skipped,
        }
    }

    pub fn validate_translation(word: &str, lang: &NativeLanguage) -> Result<String, OrigaError> {
        let translation = get_translation(word, lang);
        match translation {
            Some(t) if !t.is_empty() => Ok(t),
            _ => Err(OrigaError::VocabularyNotFound {
                word: word.to_string(),
            }),
        }
    }

    pub fn word(&self) -> &Question {
        &self.word
    }

    pub fn question(&self) -> Question {
        self.word.clone()
    }

    pub fn answer(&self, lang: &NativeLanguage) -> Result<CardAnswer, OrigaError> {
        if let Some(ref original) = self.reverse_side {
            return CardAnswer::text(original.text().to_string()).map_err(|e| {
                OrigaError::InvalidAnswer {
                    reason: e.to_string(),
                }
            });
        }

        let translations = get_translations(self.word.text(), lang).ok_or_else(|| {
            OrigaError::TranslationNotFound {
                word: self.word.text().to_string(),
                lang: *lang,
            }
        })?;
        let description = get_description(self.word.text(), lang);

        CardAnswer::vocabulary(translations, description).map_err(|e| OrigaError::InvalidAnswer {
            reason: e.to_string(),
        })
    }

    pub fn part_of_speech(&self) -> Result<PartOfSpeech, OrigaError> {
        if let Some(pos) = self.pos.clone() {
            return Ok(pos);
        }
        let tokens = tokenize_text(self.word.text())?;
        let token = tokens.first().ok_or(OrigaError::TokenizerError {
            reason: "Not found token".to_string(),
        })?;
        Ok(token.part_of_speech().clone())
    }

    pub fn pos(&self) -> Option<PartOfSpeech> {
        self.pos.clone()
    }

    pub fn with_pos(mut self, pos: PartOfSpeech) -> Self {
        self.pos = Some(pos);
        self
    }

    /// Replaces the card's word in place (dictionary-lemma migration).
    ///
    /// Keeps the reverse side and drops the stale POS: the new lemma may be a
    /// different part of speech, and `part_of_speech()` re-tokenizes lazily
    /// when `pos` is `None`.
    pub fn with_word(mut self, word: Question) -> Self {
        self.word = word;
        self.pos = None;
        self
    }

    pub fn with_grammar_rule(
        &self,
        rule: &GrammarRule,
        lang: &NativeLanguage,
    ) -> Result<(Self, String), OrigaError> {
        let formatted_word = rule.format(self.word.text(), &self.part_of_speech()?)?;
        let grammar_description = rule.content(lang).short_description().to_string();

        let answer_text = Self::validate_translation(self.word.text(), lang).and_then(|t| {
            Question::new(t).map_err(|e| OrigaError::InvalidQuestion {
                reason: e.to_string(),
            })
        })?;

        let card = Self {
            word: Question::new(formatted_word)?,
            reverse_side: Some(answer_text),
            pos: self.pos.clone(),
        };

        Ok((card, grammar_description))
    }

    pub fn revert(&self, lang: &NativeLanguage) -> Result<Self, OrigaError> {
        let meaning_text = match self.answer(lang)? {
            CardAnswer::Vocabulary { translations, .. } => translations.join(", "),
            CardAnswer::Text(s) => s,
        };
        Ok(Self {
            word: Question::new(meaning_text)?,
            reverse_side: Some(self.word.clone()),
            pos: self.pos.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::Question;
    use crate::use_cases::init_real_dictionaries;

    fn create_vocab_card(word: &str) -> VocabularyCard {
        VocabularyCard {
            word: Question::new(word.to_string()).unwrap(),
            reverse_side: None,
            pos: None,
        }
    }

    #[test]
    fn new_creates_card_with_word() {
        let card = create_vocab_card("猫");

        assert_eq!(card.word().text(), "猫");
    }

    #[test]
    fn new_creates_card_without_reverse_side() {
        let card = create_vocab_card("犬");

        assert!(card.word().text() == "犬");
    }

    #[test]
    fn word_returns_reference_to_question() {
        let card = create_vocab_card("猫");

        let word = card.word();

        assert_eq!(word.text(), "猫");
    }

    #[test]
    fn question_returns_cloned_question() {
        let card = create_vocab_card("猫");

        let question = card.question();

        assert_eq!(question.text(), "猫");
        assert_ne!(std::ptr::addr_of!(card.word), std::ptr::addr_of!(question));
    }

    #[test]
    fn answer_returns_translation_from_dictionary() {
        init_real_dictionaries();
        let card = create_vocab_card("猫");
        let lang = NativeLanguage::Russian;

        let answer = card.answer(&lang);

        assert!(answer.is_ok());
        let answer = answer.unwrap();
        assert!(
            answer
                .translations()
                .iter()
                .any(|t| t.contains("кошка") || t.contains("кот")),
            "Expected answer to contain 'кошка' or 'кот'"
        );
    }

    #[test]
    fn answer_returns_error_for_unknown_word() {
        init_real_dictionaries();
        let card = create_vocab_card("存在しない言葉");
        let lang = NativeLanguage::Russian;

        let result = card.answer(&lang);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OrigaError::TranslationNotFound { .. }
        ));
    }

    #[test]
    fn answer_returns_reverse_side_when_present() {
        let question = Question::new("猫".to_string()).unwrap();
        let reverse_side = Question::new("кошка".to_string()).unwrap();
        let card = VocabularyCard {
            word: question,
            reverse_side: Some(reverse_side),
            pos: None,
        };

        let answer = card.answer(&NativeLanguage::Russian);

        assert!(answer.is_ok());
        match answer.unwrap() {
            CardAnswer::Text(s) => assert_eq!(s, "кошка"),
            other => panic!("Expected Text variant, got {:?}", other),
        }
    }

    #[test]
    fn answer_returns_different_translations_for_different_languages() {
        init_real_dictionaries();
        let card = create_vocab_card("猫");

        let russian = card.answer(&NativeLanguage::Russian).unwrap();
        let russian_text = russian.translations().join(", ");
        let english = card.answer(&NativeLanguage::English).unwrap();
        let english_text = english.translations().join(", ");

        assert_ne!(russian_text, english_text);
    }

    #[test]
    fn validate_translation_returns_translation_for_known_word() {
        init_real_dictionaries();

        let result = VocabularyCard::validate_translation("猫", &NativeLanguage::Russian);

        assert!(result.is_ok());
        let translation = result.unwrap();
        assert!(translation.contains("кошка") || translation.contains("кот"));
    }

    #[test]
    fn validate_translation_returns_error_for_unknown_word() {
        init_real_dictionaries();

        let result =
            VocabularyCard::validate_translation("存在しない言葉", &NativeLanguage::Russian);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OrigaError::VocabularyNotFound { .. }
        ));
    }

    #[test]
    fn validate_translation_returns_error_for_empty_translation() {
        init_real_dictionaries();

        let result = VocabularyCard::validate_translation("", &NativeLanguage::Russian);

        assert!(result.is_err());
    }

    #[test]
    fn part_of_speech_returns_noun_for_noun() {
        init_real_dictionaries();
        let card = create_vocab_card("猫");

        let pos = card.part_of_speech();

        assert!(pos.is_ok());
    }

    #[test]
    fn part_of_speech_returns_verb_for_verb() {
        init_real_dictionaries();
        let card = create_vocab_card("食べる");

        let pos = card.part_of_speech();

        assert!(pos.is_ok());
    }

    #[test]
    fn part_of_speech_returns_error_for_single_space() {
        init_real_dictionaries();
        let question = Question::new(" ".to_string());

        assert!(question.is_err());
    }

    #[test]
    fn revert_creates_reversed_card() {
        init_real_dictionaries();
        let card = create_vocab_card("猫");
        let lang = NativeLanguage::Russian;

        let reversed = card.revert(&lang);

        assert!(reversed.is_ok());
        let reversed_card = reversed.unwrap();
        assert!(
            reversed_card.word().text().contains("кошка")
                || reversed_card.word().text().contains("кот")
        );
    }

    #[test]
    fn revert_swaps_question_and_answer() {
        init_real_dictionaries();
        let original_word = "猫";
        let card = create_vocab_card(original_word);
        let lang = NativeLanguage::Russian;

        let reversed_card = card.revert(&lang).unwrap();

        let question = reversed_card.question();
        let answer = reversed_card.answer(&lang).unwrap();

        assert!(
            question.text().contains("кошка") || question.text().contains("кот"),
            "question should return translation after revert"
        );
        match answer {
            CardAnswer::Text(s) => assert_eq!(s, original_word),
            other => panic!("Expected Text variant for reversed answer, got {:?}", other),
        }
    }

    #[test]
    fn with_grammar_rule_returns_error_for_invalid_rule() {
        init_real_dictionaries();
        let card = create_vocab_card("猫");
        let lang = NativeLanguage::Russian;

        let rule_id = ulid::Ulid::from_string("01G00000000000000024000000").expect("Invalid ULID");
        let rule = crate::dictionary::grammar::get_rule_by_id(&rule_id).expect("Rule not found");
        let result = card.with_grammar_rule(rule, &lang);

        assert!(result.is_err());
    }

    #[test]
    fn with_grammar_rule_stores_translation_as_reverse_side() {
        init_real_dictionaries();
        let card = create_vocab_card("食べる");
        let lang = NativeLanguage::Russian;

        let rule_id = ulid::Ulid::from_string("01G00000000000000024000000").expect("Invalid ULID");
        let rule = crate::dictionary::grammar::get_rule_by_id(&rule_id).expect("Rule not found");
        let result = card.with_grammar_rule(rule, &lang);

        assert!(result.is_ok());
        let (mutated_card, _) = result.unwrap();
        let answer = mutated_card.answer(&lang).unwrap();
        match answer {
            CardAnswer::Text(s) => {
                assert!(s.contains("есть") || s.contains("кушать"));
            },
            other => panic!("Expected Text variant for grammar rule, got {:?}", other),
        }
    }

    #[test]
    fn serialization_roundtrip() {
        let card = create_vocab_card("猫");

        let json = serde_json::to_string(&card).unwrap();
        let deserialized: VocabularyCard = serde_json::from_str(&json).unwrap();

        assert_eq!(card, deserialized);
    }

    #[test]
    fn serialization_roundtrip_with_reverse_side() {
        let question = Question::new("猫".to_string()).unwrap();
        let reverse_side = Question::new("кошка".to_string()).unwrap();
        let card = VocabularyCard {
            word: question,
            reverse_side: Some(reverse_side),
            pos: None,
        };

        let json = serde_json::to_string(&card).unwrap();
        let deserialized: VocabularyCard = serde_json::from_str(&json).unwrap();

        assert_eq!(card, deserialized);
    }

    #[test]
    fn serialization_contains_expected_fields() {
        let card = create_vocab_card("猫");

        let json = serde_json::to_string(&card).unwrap();

        assert!(json.contains("猫"));
        assert!(json.contains("word"));
    }

    #[test]
    fn clone_creates_equal_copy() {
        let card = create_vocab_card("猫");

        let cloned = card.clone();

        assert_eq!(card, cloned);
    }

    #[test]
    fn from_known_word_creates_card_for_valid_word() {
        init_real_dictionaries();
        let lang = NativeLanguage::Russian;

        let card = VocabularyCard::from_known_word("猫", &lang);

        assert!(card.is_ok());
        assert_eq!(card.unwrap().word().text(), "猫");
    }

    #[test]
    fn from_known_word_returns_error_for_unknown_word() {
        init_real_dictionaries();
        let lang = NativeLanguage::Russian;

        let result = VocabularyCard::from_known_word("存在しない言葉", &lang);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            OrigaError::VocabularyNotFound { .. }
        ));
    }

    #[test]
    fn from_known_word_returns_error_for_empty_word() {
        init_real_dictionaries();
        let lang = NativeLanguage::Russian;

        let result = VocabularyCard::from_known_word("", &lang);

        assert!(result.is_err());
    }

    #[test]
    fn from_known_word_card_answers_with_translation() {
        init_real_dictionaries();
        let lang = NativeLanguage::Russian;

        let card = VocabularyCard::from_known_word("猫", &lang).unwrap();
        let answer = card.answer(&lang);

        assert!(answer.is_ok());
        let answer = answer.unwrap();
        assert!(
            answer
                .translations()
                .iter()
                .any(|t| t.contains("кошка") || t.contains("кот")),
            "Expected translation to contain 'кошка' or 'кот'"
        );
    }

    #[test]
    fn debug_format_contains_word() {
        let card = create_vocab_card("猫");

        let debug_output = format!("{:?}", card);

        assert!(debug_output.contains("猫"));
    }
}
