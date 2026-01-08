use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    grammar::{GrammarRule, verb_forms::to_masu_stem},
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum VerbSugiruVariant {
    Plain,  // すぎる
    Polite, // すぎます
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbSugiruRule;

impl VerbSugiruRule {
    fn random_variant() -> VerbSugiruVariant {
        match rand::rng().random_range(0..=1) {
            0 => VerbSugiruVariant::Plain,
            1 => VerbSugiruVariant::Polite,
            _ => VerbSugiruVariant::Plain,
        }
    }
}

impl GrammarRule for VerbSugiruRule {
    fn apply_to(&self) -> Vec<PartOfSpeech> {
        vec![PartOfSpeech::Verb]
    }

    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError> {
        match part_of_speech {
            PartOfSpeech::Verb => {
                let random_variant = Self::random_variant();
                let suffix = match random_variant {
                    VerbSugiruVariant::Plain => "すぎる",
                    VerbSugiruVariant::Polite => "すぎます",
                };
                Ok(format!("{}{}", to_masu_stem(word), suffix))
            }
            _ => Err(KeikakuError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            }),
        }
    }

    fn title(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => "Форма ～すぎる/～すぎます".to_string(),
            NativeLanguage::English => "Form ～すぎる/～すぎます".to_string(),
        }
    }

    fn md_description(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => {
                r#"# Форма ～すぎる/～すぎます

Перебор ("слишком").

## Примеры
- この宿題の中にミスがありすぎます (Слишком много ошибок)
- 食べすぎる (Съесть слишком много)"#
            }
            NativeLanguage::English => {
                r#"# Form ～すぎる/～すぎます

Excess ("too much").

## Examples
- この宿題の中にミスがありすぎます (There are too many mistakes)
- 食べすぎる (Eat too much)"#
            }
        }
        .to_string()
    }

    fn level(&self) -> JapaneseLevel {
        JapaneseLevel::N5
    }
}
