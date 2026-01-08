use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    grammar::{GrammarRule, verb_forms::to_masen_form},
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbMasenkaRule;

impl GrammarRule for VerbMasenkaRule {
    fn apply_to(&self) -> Vec<PartOfSpeech> {
        vec![PartOfSpeech::Verb]
    }

    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError> {
        match part_of_speech {
            PartOfSpeech::Verb => Ok(format!("{}か", to_masen_form(word))),
            _ => Err(KeikakuError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            }),
        }
    }

    fn title(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => "Форма ～ませんか",
            NativeLanguage::English => "Form ～ませんか",
        }
        .to_string()
    }

    fn md_description(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => {
                r#"# Форма ～ませんか

Форма для предложения действия ("Не сделать ли?"). Добавляется к отрицательной форме глагола.

## Примеры
- ビールでも飲みませんか (Не выпить ли пива?)
- 散歩しませんか (Не прогуляться ли?)"#
            }
            NativeLanguage::English => {
                r#"# Form ～ませんか

Form for suggesting an action ("Shall we do...?"). Added to the negative form of the verb.

## Examples
- ビールでも飲みませんか (Shall we have some beer?)
- 散歩しませんか (Shall we take a walk?)"#
            }
        }
        .to_string()
    }

    fn level(&self) -> JapaneseLevel {
        JapaneseLevel::N5
    }
}
