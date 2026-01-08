use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    grammar::{GrammarRule, verb_forms::to_mashou_form},
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbMashouRule;

impl GrammarRule for VerbMashouRule {
    fn apply_to(&self) -> Vec<PartOfSpeech> {
        vec![PartOfSpeech::Verb]
    }

    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError> {
        match part_of_speech {
            PartOfSpeech::Verb => Ok(to_mashou_form(word)),
            _ => Err(KeikakuError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            }),
        }
    }

    fn title(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => "Форма ～ましょう",
            NativeLanguage::English => "Form ～ましょう",
        }
        .to_string()
    }

    fn md_description(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => {
                r#"# Форма ～ましょう

Форма для предложения или приглашения ("Давайте сделаем").

## Примеры
- 一緒に行きましょう (Давайте пойдем вместе)
- 食べましょう (Давайте поедим)"#
            }
            NativeLanguage::English => {
                r#"# Form ～ましょう

Form for suggestion or invitation ("Let's do").

## Examples
- 一緒に行きましょう (Let's go together)
- 食べましょう (Let's eat)"#
            }
        }
        .to_string()
    }

    fn level(&self) -> JapaneseLevel {
        JapaneseLevel::N5
    }
}
