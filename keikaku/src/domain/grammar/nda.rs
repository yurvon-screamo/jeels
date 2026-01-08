use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    grammar::GrammarRule,
    tokenizer::PartOfSpeech,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct んだRule;

impl GrammarRule for んだRule {
    fn apply_to(&self) -> Vec<PartOfSpeech> {
        vec![
            PartOfSpeech::Verb,
            PartOfSpeech::Noun,
            PartOfSpeech::IAdjective,
            PartOfSpeech::NaAdjective,
        ]
    }

    fn format(&self, word: &str, part_of_speech: &PartOfSpeech) -> Result<String, KeikakuError> {
        match part_of_speech {
            PartOfSpeech::Verb | PartOfSpeech::IAdjective => Ok(format!("{word}んだ")),
            PartOfSpeech::Noun | PartOfSpeech::NaAdjective => Ok(format!("{word}なんだ")),
            _ => Err(KeikakuError::GrammarFormatError {
                reason: "Not supported part of speech".to_string(),
            }),
        }
    }

    fn title(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => "Конструкция んだ・んです",
            NativeLanguage::English => "Construction んだ・んです",
        }
        .to_string()
    }

    fn md_description(&self, lang: &NativeLanguage) -> String {
        match lang {
            NativeLanguage::Russian => r#"# Конструкция んだ・んです

Конструкция んだ (неформальная) и んです (вежливая) используется для объяснения причины, подтверждения факта или выражения эмоций. Это как русское "ведь" или "потому что".

## Как образуется
- После глаголов и い-прилагательных: основа + んだ/んです
- После существительных и な-прилагательных: слово + なんだ/なんです

## Примеры объяснения причины
- 今日は休みなんです (Сегодня ведь выходной)
- 疲れたんです (Потому что устал)

## Примеры подтверждения
- 学生なんです (Я ведь студент)
- おいしいんです (Ведь вкусно)

## Примеры с эмоциями
- 嬉しいんです (Я рад!)
- 残念なんです (Жаль...)"#,
            NativeLanguage::English => r#"# Construction んだ・んです

The んだ (informal) and んです (polite) construction is used to explain reasons, confirm facts, or express emotions. It's like English "you know" or "because".

## How it is formed
- After verbs and い-adjectives: base + んだ/んです
- After nouns and な-adjectives: word + なんだ/なんです

## Examples of explanation
- 今日は休みなんです (Today is a holiday, you know)
- 疲れたんです (Because I'm tired)

## Examples of confirmation
- 学生なんです (I'm a student, you see)
- おいしいんです (It's delicious, right?)

## Examples with emotions
- 嬉しいんです (I'm happy!)
- 残念なんです (That's a shame...)"#,
        }
        .to_string()
    }

    fn level(&self) -> JapaneseLevel {
        JapaneseLevel::N5
    }
}
