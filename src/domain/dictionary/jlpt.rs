use std::{
    collections::{BTreeMap, HashMap},
    sync::LazyLock,
};

use serde::Deserialize;

use crate::domain::{dictionary::kanji::parse_jlpt_level, value_objects::JapaneseLevel};

const JLPT_WORDS_RAW: &str = include_str!("./jltp_words.json");

pub static JLPT_DB: LazyLock<JlptDatabase> = LazyLock::new(JlptDatabase::new);

#[derive(Deserialize)]
struct JlptWordsStoredType {
    #[serde(flatten)]
    levels: HashMap<String, Vec<String>>,
}

pub struct JlptDatabase {
    level_to_words: BTreeMap<JapaneseLevel, Vec<String>>,
    word_to_level: HashMap<String, JapaneseLevel>,
}

impl Default for JlptDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl JlptDatabase {
    pub fn new() -> Self {
        let stored: JlptWordsStoredType =
            serde_json::from_str(JLPT_WORDS_RAW).expect("Failed to parse JLPT words dictionary");

        let mut level_to_words: BTreeMap<JapaneseLevel, Vec<String>> = BTreeMap::new();
        let mut word_to_level: HashMap<String, JapaneseLevel> = HashMap::new();

        for (level_str, words) in stored.levels {
            let level = parse_jlpt_level(&level_str);
            let level_words = level_to_words.entry(level.clone()).or_default();

            for word in words {
                // If word already exists, keep the first encountered level.
                word_to_level
                    .entry(word.clone())
                    .or_insert_with(|| level.clone());
                level_words.push(word);
            }
        }

        Self {
            level_to_words,
            word_to_level,
        }
    }

    pub fn get_level(&self, word: &str) -> Option<JapaneseLevel> {
        self.word_to_level.get(word).copied()
    }

    pub fn get_words_for_level(&self, level: &JapaneseLevel) -> Vec<String> {
        self.level_to_words.get(level).cloned().unwrap_or_default()
    }

    pub fn available_levels(&self) -> Vec<JapaneseLevel> {
        self.level_to_words.keys().cloned().collect()
    }
}
