use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    grammar::{GrammarRule, verb_forms::to_masu_stem},
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum VerbNiIkuVariant {
    Plain,  // いく
    Polite, // に行きます
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbNiIkuRule;

impl VerbNiIkuRule {
    fn random_variant() -> VerbNiIkuVariant {
        match rand::rng().random_range(0..=1) {
            0 => VerbNiIkuVariant::Plain,
            1 => VerbNiIkuVariant::Polite,
            _ => VerbNiIkuVariant::Plain,
        }
    }
}

impl GrammarRule for VerbNiIkuRule {
    fn apply_to(&self) -> Vec<PartOfSpeech> {
        vec![PartOfSpeech::Verb]
    }

    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError> {
        match part_of_speech {
            PartOfSpeech::Verb => {
                let random_variant = Self::random_variant();
                let suffix = match random_variant {
                    VerbNiIkuVariant::Plain => "にいく",
                    VerbNiIkuVariant::Polite => "に行きます",
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
            NativeLanguage::Russian => "Конструкция ～にいく/に行きます".to_string(),
            NativeLanguage::English => "Construction ～にいく/に行きます".to_string(),
        }
    }

    fn md_description(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => {
                r#"# Конструкция ～にいく/に行きます

Конструкция цели ("идти, чтобы сделать").

## Примеры
- ちょうど今食べにいきます (Иду есть)
- 買い物に行きます (Иду за покупками)"#
            }
            .to_string(),
            NativeLanguage::English => {
                r#"# Construction ～にいく/に行きます

Construction for purpose ("go to do").

## Examples
- ちょうど今食べにいきます (I'm going to eat now)
- 買い物に行きます (I'm going shopping)"#
            }
            .to_string(),
        }
    }

    fn level(&self) -> JapaneseLevel {
        JapaneseLevel::N5
    }
}
