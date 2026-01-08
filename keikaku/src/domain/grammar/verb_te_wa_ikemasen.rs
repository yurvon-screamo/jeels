use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    grammar::{GrammarRule, verb_forms::to_te_form},
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbTeWaIkemasenRule;

impl GrammarRule for VerbTeWaIkemasenRule {
    fn apply_to(&self) -> Vec<PartOfSpeech> {
        vec![PartOfSpeech::Verb]
    }

    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError> {
        match part_of_speech {
            PartOfSpeech::Verb => Ok(format!("{}はいけません", to_te_form(word))),
            _ => Err(KeikakuError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            }),
        }
    }

    fn title(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => "Форма ～てはいけません",
            NativeLanguage::English => "Form ～てはいけません",
        }
        .to_string()
    }

    fn md_description(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => {
                r#"# Форма ～てはいけません

Форма запрета ("Нельзя делать").

## Примеры
- ２度と学校に遅れてはいけません (Не опаздывай снова)
- 触ってはいけません (Нельзя трогать)"#
            }
            NativeLanguage::English => {
                r#"# Form ～てはいけません

Form for prohibition ("Must not do").

## Examples
- ２度と学校に遅れてはいけません (You must not be late to school again)
- 触ってはいけません (You must not touch)"#
            }
        }
        .to_string()
    }

    fn level(&self) -> JapaneseLevel {
        JapaneseLevel::N5
    }
}
