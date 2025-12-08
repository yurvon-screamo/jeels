use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::domain::{
    JeersError,
    dictionary::{KANJI_DB, RadicalInfo, VOCABULARY_DB},
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanjiCard {
    id: Ulid,
    kanji: char,
    description: String,
    example_words: Vec<ExampleKanjiWord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleKanjiWord {
    word: String,
    meaning: String,
}

impl KanjiCard {
    pub fn new(kanji: char, native_language: &NativeLanguage) -> Result<Self, JeersError> {
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
            id: Ulid::new(),
            kanji,
            description: description.to_string(),
            example_words,
        })
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn kanji(&self) -> char {
        self.kanji
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn example_words(&self) -> &[ExampleKanjiWord] {
        &self.example_words
    }

    pub fn jlpt(&self) -> JapaneseLevel {
        KANJI_DB
            .get_kanji_info(&self.kanji)
            .map(|kanji_info| kanji_info.jlpt().to_owned())
            .unwrap_or(JapaneseLevel::N1)
    }

    pub fn used_in(&self) -> u32 {
        KANJI_DB
            .get_kanji_info(&self.kanji)
            .map(|kanji_info| kanji_info.used_in())
            .unwrap_or(0)
    }

    pub fn radicals_info(&self) -> Result<Vec<&RadicalInfo>, JeersError> {
        Ok(KANJI_DB.get_kanji_info(&self.kanji)?.radicals())
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
