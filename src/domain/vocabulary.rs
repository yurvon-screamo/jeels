use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{ExamplePhrase, JapaneseLevel, NativeLanguage};

const VOCABULARY_N5_DATA: &str = include_str!("./vocabulary_n5.json");
const VOCABULARY_N4_DATA: &str = include_str!("./vocabulary_n4.json");
const VOCABULARY_N3_DATA: &str = include_str!("./vocabulary_n3.json");
const VOCABULARY_N2_DATA: &str = include_str!("./vocabulary_n2.json");
const VOCABULARY_N1_DATA: &str = include_str!("./vocabulary_n1.json");

pub static VOCABULARY_DB: LazyLock<VocabularyDatabase> = LazyLock::new(VocabularyDatabase::new);

#[derive(Debug, Clone)]
pub struct VocabularyInfo {
    pub word: String,
    pub level: JapaneseLevel,
    pub russian_translation: String,
    pub english_translation: String,
    pub russian_examples: Vec<ExamplePhrase>,
    pub english_examples: Vec<ExamplePhrase>,
}

#[derive(Serialize, Deserialize)]
struct ExamplePhraseStoredType {
    text: String,
    translation: String,
}

#[derive(Serialize, Deserialize)]
struct VocabularyEntryStoredType {
    level: String,
    russian_translation: String,
    english_translation: String,
    russian_examples: Vec<ExamplePhraseStoredType>,
    english_examples: Vec<ExamplePhraseStoredType>,
}

pub struct VocabularyDatabase {
    vocabulary_map: HashMap<String, VocabularyInfo>,
}

impl Default for VocabularyDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl VocabularyDatabase {
    pub fn new() -> Self {
        let vocabulary_data: HashMap<_, _> = serde_json::from_str::<
            HashMap<String, VocabularyEntryStoredType>,
        >(VOCABULARY_N5_DATA)
        .unwrap()
        .into_iter()
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(VOCABULARY_N4_DATA)
                .unwrap()
                .into_iter(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(VOCABULARY_N3_DATA)
                .unwrap()
                .into_iter(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(VOCABULARY_N2_DATA)
                .unwrap()
                .into_iter(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(VOCABULARY_N1_DATA)
                .unwrap()
                .into_iter(),
        )
        .collect();

        let vocabulary_map = vocabulary_data
            .into_iter()
            .map(|(word, entry)| {
                let level = parse_jlpt_level(&entry.level);
                let russian_examples = entry
                    .russian_examples
                    .into_iter()
                    .map(|e| ExamplePhrase::new(e.text, e.translation))
                    .collect();
                let english_examples = entry
                    .english_examples
                    .into_iter()
                    .map(|e| ExamplePhrase::new(e.text, e.translation))
                    .collect();

                (
                    word.clone(),
                    VocabularyInfo {
                        word,
                        level,
                        russian_translation: entry.russian_translation,
                        english_translation: entry.english_translation,
                        russian_examples,
                        english_examples,
                    },
                )
            })
            .collect::<HashMap<String, VocabularyInfo>>();

        Self { vocabulary_map }
    }

    pub fn get_translation(&self, word: &str, native_language: &NativeLanguage) -> Option<String> {
        self.vocabulary_map
            .get(word)
            .map(|info| match native_language {
                NativeLanguage::Russian => info.russian_translation.clone(),
                NativeLanguage::English => info.english_translation.clone(),
            })
    }

    pub fn get_examples(
        &self,
        word: &str,
        native_language: &NativeLanguage,
    ) -> Option<Vec<ExamplePhrase>> {
        self.vocabulary_map
            .get(word)
            .map(|info| match native_language {
                NativeLanguage::Russian => info.russian_examples.clone(),
                NativeLanguage::English => info.english_examples.clone(),
            })
    }

    pub fn get_vocabulary_info(&self, word: &str) -> Option<&VocabularyInfo> {
        self.vocabulary_map.get(word)
    }
}

fn parse_jlpt_level(s: &str) -> JapaneseLevel {
    match s {
        "N5" => JapaneseLevel::N5,
        "N4" => JapaneseLevel::N4,
        "N3" => JapaneseLevel::N3,
        "N2" => JapaneseLevel::N2,
        "N1" => JapaneseLevel::N1,
        _ => JapaneseLevel::N1,
    }
}
