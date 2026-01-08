use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    grammar::{GrammarRule, verb_forms::to_te_form},
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbTeKudasaiRule;

impl GrammarRule for VerbTeKudasaiRule {
    fn apply_to(&self) -> Vec<PartOfSpeech> {
        vec![PartOfSpeech::Verb]
    }

    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError> {
        match part_of_speech {
            PartOfSpeech::Verb => Ok(format!("{}ください", to_te_form(word))),
            _ => Err(KeikakuError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            }),
        }
    }

    fn title(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => "Форма ～てください",
            NativeLanguage::English => "Form ～てください",
        }
        .to_string()
    }

    fn md_description(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => {
                r#"# Форма ～てください (Вежливая просьба)

Форма ～てください используется для **вежливой просьбы** к собеседнику совершить какое-то действие. Это стандартный способ попросить о чем-то в японском языке.

## Как образуется
Глагол в て-форме + ください

## Примеры
- **座って**ください (Пожалуйста, **сядьте**)
- この本を**読んで**ください (Пожалуйста, **прочитайте** эту книгу)
- ちょっと**待って**ください (Пожалуйста, **подождите** немного)

## Важные особенности
- Очень вежливая форма, подходит для просьб к старшим, незнакомым людям
- В неформальной речи может использоваться て (座って)
- Можно смягчить просьбу, добавив ちょっと (немного) или すみませんが (извините)

## Отличие от других форм просьбы
- ～てください - вежливая просьба
- ～てくれ - неформальная просьба (друзьям, младшим)
- ～て - команда (только близким)"#
            }
            NativeLanguage::English => {
                r#"# Form ～てください

Form for request ("Please do").

## Examples
- このノートで書いてください (Please write in this notebook)
- 待ってください (Please wait)"#
            }
        }
        .to_string()
    }

    fn level(&self) -> JapaneseLevel {
        JapaneseLevel::N5
    }
}
