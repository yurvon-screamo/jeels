use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    grammar::{GrammarRule, verb_forms::to_masu_stem},
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbTaiRule;

impl GrammarRule for VerbTaiRule {
    fn apply_to(&self) -> Vec<PartOfSpeech> {
        vec![PartOfSpeech::Verb]
    }

    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError> {
        match part_of_speech {
            PartOfSpeech::Verb => Ok(format!("{}たいです", to_masu_stem(word))),
            _ => Err(KeikakuError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            }),
        }
    }

    fn title(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => "Форма ～たいです",
            NativeLanguage::English => "Form ～たいです",
        }
        .to_string()
    }

    fn md_description(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => {
                r#"# Форма ～たいです (Выражение желания)

Форма ～たいです выражает **личное желание** сделать что-то. Она образуется от глагола и означает "хотеть/желать" совершить действие.

## Как образуется
Основа глагола в ます-форме + たいです

## Примеры
- 日本に**行きたいです** (Я **хочу поехать** в Японию)
- 寿司を**食べたいです** (Я **хочу съесть** суши)
- 日本語を**勉強したいです** (Я **хочу изучать** японский)

## Важные особенности
- Это **личное желание** (я хочу), а не предложение другим
- Можно использовать в отрицании: 行きたくないです (Не хочу идти)
- Можно использовать в прошедшем времени: 行きたかったです (Хотел пойти)

## Отличие от ほしい
- ～たい - желание **сделать** что-то
- ほしい - желание **получить** что-то: 車がほしい (Хочу машину)"#
            }
            NativeLanguage::English => {
                r#"# Form ～たいです

Desire ("want to").

## Examples
- 日本に行きたいです (I want to go to Japan)
- 食べたいです (I want to eat)"#
            }
        }
        .to_string()
    }

    fn level(&self) -> JapaneseLevel {
        JapaneseLevel::N5
    }
}
