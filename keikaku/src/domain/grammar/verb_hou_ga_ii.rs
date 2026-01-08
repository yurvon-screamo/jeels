use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    grammar::{GrammarRule, verb_forms::to_ta_form},
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerbHouGaIiVariant {
    Plain,  // ほうがいい
    Polite, // ほうがいいです
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbHouGaIiRule;

impl VerbHouGaIiRule {
    fn random_variant() -> VerbHouGaIiVariant {
        match rand::rng().random_range(0..=1) {
            0 => VerbHouGaIiVariant::Plain,
            1 => VerbHouGaIiVariant::Polite,
            _ => VerbHouGaIiVariant::Plain,
        }
    }
}

impl GrammarRule for VerbHouGaIiRule {
    fn apply_to(&self) -> Vec<PartOfSpeech> {
        vec![PartOfSpeech::Verb]
    }

    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError> {
        match part_of_speech {
            PartOfSpeech::Verb => {
                let random_variant = Self::random_variant();
                let suffix = match random_variant {
                    VerbHouGaIiVariant::Plain => "ほうがいい",
                    VerbHouGaIiVariant::Polite => "ほうがいいです",
                };
                Ok(format!("{}{}", to_ta_form(word), suffix))
            }
            _ => Err(KeikakuError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            }),
        }
    }

    fn title(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => "Конструкция ～たほうがいい/～たほうがいいです".to_string(),
            NativeLanguage::English => "Construction ～たほうがいい/～たほうがいいです".to_string(),
        }
    }

    fn md_description(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => r#"# Конструкция ～たほうがいい/～たほうがいいです

Конструкция выражает **совет** или **рекомендацию** - что лучше сделать. Подчеркивает, что один вариант предпочтительнее другого.

## Как образуется
Глагол в た-форме + ほうがいい (неформ.) / ほうがいいです (вежл.)

## Примеры
- **早く寝た**ほうがいい (Лучше **лечь спать рано**)
- **勉強した**ほうがいいです (Лучше **поучить** - вежл.)
- 歩いて**行った**ほうがいい (Лучше **пойти пешком**)

## Сравнение с другими конструкциями
- ほうがいい - совет (лучше сделать)
- たらどうか - предложение (как насчет того, чтобы?)
- たら - условие + совет

## В отрицании
- 行かない**ほうがいい** (Лучше **не идти**)
- 食べない**ほうがいいです** (Лучше **не есть** - вежл.)"#,
            NativeLanguage::English => r#"# Construction ～たほうがいい/～たほうがいいです

The construction expresses **advice** or **recommendation** - what is better to do. Emphasizes that one option is preferable to another.

## How it is formed
Verb in ta-form + ほうがいい (informal) / ほうがいいです (polite)

## Examples
- **早く寝た**ほうがいい (Better to **go to bed early**)
- **勉強した**ほうがいいです (Better to **study** - polite)
- 歩いて**行った**ほうがいい (Better to **go on foot**)

## Comparison with other constructions
- ほうがいい - advice (better to do)
- たらどうか - suggestion (how about...?)
- たら - condition + advice

## In negation
- 行かない**ほうがいい** (Better **not to go**)
- 食べない**ほうがいいです** (Better **not to eat** - polite)"#,
        }
        .to_string()
    }

    fn level(&self) -> JapaneseLevel {
        JapaneseLevel::N5
    }
}
