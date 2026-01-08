use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    grammar::GrammarRule,
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdjectivePastRule;

impl GrammarRule for AdjectivePastRule {
    fn apply_to(&self) -> Vec<PartOfSpeech> {
        vec![PartOfSpeech::NaAdjective, PartOfSpeech::IAdjective]
    }

    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError> {
        match part_of_speech {
            PartOfSpeech::IAdjective => {
                let word = word.trim_end_matches("い");
                Ok(format!("{word}かった"))
            }
            PartOfSpeech::NaAdjective => {
                let word = word.trim_end_matches("な");
                Ok(format!("{word}でした"))
            }
            _ => Err(KeikakuError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            }),
        }
    }

    fn title(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => "Прилагательное в прошедшей форме",
            NativeLanguage::English => "Adjective in past form",
        }
        .to_string()
    }

    fn md_description(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => {
                r#"# Прошедшая форма прилагательного

Прошедшая форма прилагательных используется для описания состояний или качеств в прошлом.

## な прилагательные

Для образования прошедшей формы な прилагательного к основе прилагательного добавляется "でした".

### Примеры
- 静か**でした** (Было тихо) - от 静かな (тихий)
- きれい**でした** (Было красиво) - от きれいな (красивый)

## い прилагательные

Для образования прошедшей формы い прилагательного к основе прилагательного (без い) добавляется "かった".

### Примеры
- 高**かった** (Было высоко) - от 高い (высокий)
- 寒**かった** (Было холодно) - от 寒い (холодный)
- 面白**かった** (Было интересно) - от 面白い (интересный)

## Важно
- い прилагательные меняют последний слог い на かった
- な прилагательные используют форму でした (та же, что и прошедшее время глагола です)"#
            }
            NativeLanguage::English => r#"# Adjective in past form

Past form of adjectives is used to describe states or qualities in the past.

## な adjectives

To form the past tense of a な adjective, add "でした" to the base adjective.

### Examples
- 静か**でした** (It was quiet) - from 静かな (quiet)
- きれい**でした** (It was beautiful) - from きれいな (beautiful)

## い adjectives

To form the past tense of an い adjective, remove the final い and add "かった".

### Examples
- 高**かった** (It was high) - from 高い (high)
- 寒**かった** (It was cold) - from 寒い (cold)
- 面白**かった** (It was interesting) - from 面白い (interesting)

## Important
- い adjectives change the final い to かった
- な adjectives use でした (same as the past tense of the copula です)"#,
        }
        .to_string()
    }

    fn level(&self) -> JapaneseLevel {
        JapaneseLevel::N5
    }
}
