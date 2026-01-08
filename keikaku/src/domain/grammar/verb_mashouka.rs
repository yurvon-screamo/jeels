use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    grammar::{GrammarRule, verb_forms::to_mashou_form},
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbMashoukaRule;

impl GrammarRule for VerbMashoukaRule {
    fn apply_to(&self) -> Vec<PartOfSpeech> {
        vec![PartOfSpeech::Verb]
    }

    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError> {
        match part_of_speech {
            PartOfSpeech::Verb => Ok(format!("{}か", to_mashou_form(word))),
            _ => Err(KeikakuError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            }),
        }
    }

    fn title(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => "Форма ～ましょうか",
            NativeLanguage::English => "Form ～ましょうか",
        }
        .to_string()
    }

    fn md_description(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => {
                r#"# Форма ～ましょうか

Форма для предложения помощи ("Давайте я сделаю?").

## Примеры
- あなたの荷物を運びましょうか (Поднести ваши вещи?)
- 手伝いましょうか (Помочь?)"#
            }
            NativeLanguage::English => {
                r#"# Form ～ましょうか

Form for offering help ("Shall I do...?").

## Examples
- あなたの荷物を運びましょうか (Shall I carry your luggage?)
- 手伝いましょうか (Shall I help?)"#
            }
        }
        .to_string()
    }

    fn level(&self) -> JapaneseLevel {
        JapaneseLevel::N5
    }
}
