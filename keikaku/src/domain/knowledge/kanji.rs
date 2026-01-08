use crate::domain::{
    KeikakuError,
    dictionary::{KANJI_DB, RadicalInfo, VOCABULARY_DB},
    value_objects::{Answer, JapaneseLevel, NativeLanguage, Question},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KanjiCard {
    kanji: Question,
    description: Answer,
    example_words: Vec<ExampleKanjiWord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExampleKanjiWord {
    word: String,
    meaning: String,
}

impl KanjiCard {
    pub fn new(kanji: String, native_language: &NativeLanguage) -> Result<Self, KeikakuError> {
        let kanji_info = KANJI_DB.get_kanji_info(&kanji)?;
        let description = kanji_info.description();
        let example_words = kanji_info
            .popular_words()
            .iter()
            .map(|word| {
                let meaning = VOCABULARY_DB
                    .get_vocabulary_info(word)
                    .map(|kanji_info| match native_language {
                        NativeLanguage::Russian => kanji_info.russian_translation().to_string(),
                        NativeLanguage::English => kanji_info.english_translation().to_string(),
                    })
                    .unwrap_or_default();

                ExampleKanjiWord {
                    word: word.clone(),
                    meaning,
                }
            })
            .collect();

        Ok(Self {
            kanji: Question::new(kanji.to_string())?,
            description: Answer::new(description.to_string())?,
            example_words,
        })
    }

    pub fn kanji(&self) -> &Question {
        &self.kanji
    }

    pub fn description(&self) -> &Answer {
        &self.description
    }

    pub fn example_words(&self) -> &[ExampleKanjiWord] {
        &self.example_words
    }

    pub fn jlpt(&self) -> JapaneseLevel {
        KANJI_DB
            .get_kanji_info(self.kanji.text())
            .map(|kanji_info| kanji_info.jlpt().to_owned())
            .unwrap_or(JapaneseLevel::N1)
    }

    pub fn used_in(&self) -> u32 {
        KANJI_DB
            .get_kanji_info(self.kanji.text())
            .map(|kanji_info| kanji_info.used_in())
            .unwrap_or(0)
    }

    pub fn radicals_info(&self) -> Result<Vec<&RadicalInfo>, KeikakuError> {
        Ok(KANJI_DB.get_kanji_info(self.kanji.text())?.radicals())
    }
}

impl ExampleKanjiWord {
    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn meaning(&self) -> &str {
        &self.meaning
    }
}
