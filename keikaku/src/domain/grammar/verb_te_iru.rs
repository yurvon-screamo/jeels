use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    grammar::{GrammarRule, verb_forms::to_te_form},
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerbTeIruVariant {
    Plain,  // いる
    Polite, // います
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbTeIruRule;

impl VerbTeIruRule {
    fn random_variant() -> VerbTeIruVariant {
        match rand::rng().random_range(0..=1) {
            0 => VerbTeIruVariant::Plain,
            1 => VerbTeIruVariant::Polite,
            _ => VerbTeIruVariant::Plain,
        }
    }
}

impl GrammarRule for VerbTeIruRule {
    fn apply_to(&self) -> Vec<PartOfSpeech> {
        vec![PartOfSpeech::Verb]
    }

    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError> {
        match part_of_speech {
            PartOfSpeech::Verb => {
                let random_variant = Self::random_variant();
                let suffix = match random_variant {
                    VerbTeIruVariant::Plain => "いる",
                    VerbTeIruVariant::Polite => "います",
                };
                Ok(format!("{}{}", to_te_form(word), suffix))
            }
            _ => Err(KeikakuError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            }),
        }
    }

    fn title(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => "Форма ～ている/～ています".to_string(),
            NativeLanguage::English => "Form ～ている/～ています".to_string(),
        }
    }

    fn md_description(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => {
                r#"# Форма ～ている/～ています (Длительное действие / Состояние)

Форма ～ている (неформ.) / ～ています (вежл.) выражает:
1. **Длительное действие** в настоящем времени (что-то происходит сейчас)
2. **Состояние или результат** действия (что-то уже сделано и сохраняется)

## Как образуется
Глагол в て-форме + いる/います

## Примеры длительного действия
- 今**勉強している** (Сейчас **учусь**)
- テレビを**見ている** (Смотрю телевизор)
- 今勉強**しています** (Сейчас учусь - вежл.)

## Примеры состояния/результата
- 結婚**している** (Женат/замужем - состояние)
- ドアが開い**ている** (Дверь открыта - результат действия)
- 日本語を勉強**している** (Изучаю японский - длительное действие)

## В отрицании
- 勉強**していない** (Не учусь / Не изучаю)
- 勉強**していません** (Не учусь - вежл.)

## В прошедшем времени
- 勉強**していた** (Учился / Изучал раньше)
- 勉強**していました** (Учился - вежл.)"#
            }
            NativeLanguage::English => {
                r#"# Form ～ている/～ています (Continuous Action / State)

The ～ている (informal) / ～ています (polite) form expresses:
1. **Continuous action** in the present tense (something is happening now)
2. **State or result** of an action (something has been done and remains)

## How it is formed
Verb in te-form + いる/います

## Examples of continuous action
- 今**勉強している** (I am **studying** now)
- テレビを**見ている** (**Watching** TV)
- 今勉強**しています** (I am studying now - polite)

## Examples of state/result
- 結婚**している** (**Married** - state)
- ドアが開い**ている** (**Door is open** - result of action)
- 日本語を勉強**している** (**Studying** Japanese - continuous action)

## In negation
- 勉強**していない** (Not studying / Not learning)
- 勉強**していません** (Not studying - polite)

## In past tense
- 勉強**していた** (Was studying / Studied before)
- 勉強**していました** (Was studying - polite)"#
            }
        }
        .to_string()
    }

    fn level(&self) -> JapaneseLevel {
        JapaneseLevel::N5
    }
}
