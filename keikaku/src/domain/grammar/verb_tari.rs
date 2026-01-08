use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    grammar::{GrammarRule, verb_forms::to_ta_form},
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbTariRule;

impl GrammarRule for VerbTariRule {
    fn apply_to(&self) -> Vec<PartOfSpeech> {
        vec![PartOfSpeech::Verb]
    }

    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError> {
        match part_of_speech {
            PartOfSpeech::Verb => Ok(format!("{}りする", to_ta_form(word))),
            _ => Err(KeikakuError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            }),
        }
    }

    fn title(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => "Форма ～たり…～たりする",
            NativeLanguage::English => "Form ～たり…～たりする",
        }
        .to_string()
    }

    fn md_description(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => r#"# Форма ～たり…～たりする

Перечисление параллельных действий.

## Примеры
- 昨夜は歌ったり踊ったりした (Пела и танцевала прошлой ночью)
- 読んだり書いたりします (Читаю и пишу)

*Примечание: Это правило обычно используется с несколькими глаголами, но здесь показана базовая форма для одного глагола.*"#,
            NativeLanguage::English => r#"# Form ～たり…～たりする

Enumeration of parallel actions.

## Examples
- 昨夜は歌ったり踊ったりした (Sang and danced last night)
- 読んだり書いたりします (Read and write)

*Note: This rule is usually used with multiple verbs, but here the basic form for one verb is shown.*"#,
        }
        .to_string()
    }

    fn level(&self) -> JapaneseLevel {
        JapaneseLevel::N5
    }
}
