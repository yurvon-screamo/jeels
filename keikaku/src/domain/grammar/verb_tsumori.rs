use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    grammar::{GrammarRule, verb_forms::to_masu_stem},
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstructionTsumoriRule;

impl GrammarRule for ConstructionTsumoriRule {
    fn apply_to(&self) -> Vec<PartOfSpeech> {
        vec![PartOfSpeech::Verb]
    }

    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError> {
        match part_of_speech {
            PartOfSpeech::Verb => Ok(format!("{}つもりです", to_masu_stem(word))),
            _ => Err(KeikakuError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            }),
        }
    }

    fn title(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => "Конструкция つもりです",
            NativeLanguage::English => "Construction つもりです",
        }
        .to_string()
    }

    fn md_description(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => {
                r#"# Конструкция つもりです

Намерение ("собираюсь").

## Примеры
- 会議には出ないつもりです (Не собираюсь на встречу)
- 留学するつもりです (Собираюсь учиться за границей)"#
            }
            NativeLanguage::English => {
                r#"# Construction つもりです

Intention ("plan to").

## Examples
- 会議には出ないつもりです (I don't plan to attend the meeting)
- 留学するつもりです (I plan to study abroad)"#
            }
        }
        .to_string()
    }

    fn level(&self) -> JapaneseLevel {
        JapaneseLevel::N5
    }
}
