use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    grammar::{GrammarRule, verb_forms::to_ta_form},
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerbTaKotoGaAruVariant {
    Plain,  // ことがある
    Polite, // ことがあります
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerbTaKotoGaAruRule;

impl VerbTaKotoGaAruRule {
    fn random_variant() -> VerbTaKotoGaAruVariant {
        match rand::rng().random_range(0..=1) {
            0 => VerbTaKotoGaAruVariant::Plain,
            1 => VerbTaKotoGaAruVariant::Polite,
            _ => VerbTaKotoGaAruVariant::Plain,
        }
    }
}

impl GrammarRule for VerbTaKotoGaAruRule {
    fn apply_to(&self) -> Vec<PartOfSpeech> {
        vec![PartOfSpeech::Verb]
    }

    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError> {
        match part_of_speech {
            PartOfSpeech::Verb => {
                let random_variant = Self::random_variant();
                let suffix = match random_variant {
                    VerbTaKotoGaAruVariant::Plain => "ことがある",
                    VerbTaKotoGaAruVariant::Polite => "ことがあります",
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
            NativeLanguage::Russian => "Конструкция ～たことがある/～たことがあります".to_string(),
            NativeLanguage::English => "Construction ～たことがある/～たことがあります".to_string(),
        }
    }

    fn md_description(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => r#"# Конструкция ～たことがある/～たことがあります

Конструкция выражает **опыт** - то, что случалось делать в прошлом. Отвечает на вопрос "бывал ли ты когда-нибудь...?"

## Как образуется
Глагол в た-форме + ことがある (неформ.) / ことがあります (вежл.)

## Примеры
- そこには**前に行ったことがある** (Бывал там раньше)
- 寿司を**食べたことがある** (Ел суши)
- ヨーロッパに**行ったことがあります** (Бывал в Европе - вежл.)

## В отрицании
- 寿司を食べた**ことがない** (Никогда не ел суши)
- 寿司を食べた**ことがありません** (Никогда не ел суши - вежл.)

## Важно
- Подчеркивает личный опыт, а не факт
- Не используется для недавних действий
- Для недавних действий: たばこをやめた (бросил курить)"#,
            NativeLanguage::English => r#"# Construction ～たことがある/～たことがあります

The construction expresses **experience** - something that happened to do in the past. Answers the question "have you ever...?"

## How it is formed
Verb in ta-form + ことがある (informal) / ことがあります (polite)

## Examples
- そこには**前に行ったことがある** (Have been there before)
- 寿司を**食べたことがある** (Have eaten sushi)
- ヨーロッパに**行ったことがあります** (Have been to Europe - polite)

## In negation
- 寿司を食べた**ことがない** (Have never eaten sushi)
- 寿司を食べた**ことがありません** (Have never eaten sushi - polite)

## Important
- Emphasizes personal experience, not just fact
- Not used for recent actions
- For recent actions: たばこをやめた (quit smoking)"#,
        }
        .to_string()
    }

    fn level(&self) -> JapaneseLevel {
        JapaneseLevel::N5
    }
}
