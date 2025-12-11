use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Serialize};

use crate::domain::{
    dictionary::kanji::parse_jlpt_level,
    value_objects::{ExamplePhrase, JapaneseLevel, NativeLanguage},
};

pub static VOCABULARY_DB: LazyLock<VocabularyDatabase> = LazyLock::new(VocabularyDatabase::new);

#[derive(Debug, Clone)]
pub struct VocabularyInfo {
    word: String,
    level: JapaneseLevel,
    russian_translation: String,
    english_translation: String,
    russian_examples: Vec<ExamplePhrase>,
    english_examples: Vec<ExamplePhrase>,
}

impl VocabularyInfo {
    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn level(&self) -> &JapaneseLevel {
        &self.level
    }

    pub fn russian_translation(&self) -> &str {
        &self.russian_translation
    }

    pub fn english_translation(&self) -> &str {
        &self.english_translation
    }

    pub fn russian_examples(&self) -> &[ExamplePhrase] {
        &self.russian_examples
    }

    pub fn english_examples(&self) -> &[ExamplePhrase] {
        &self.english_examples
    }
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

static N5_PART1_STR: &str = include_str!("../../../../vocabulary/n5_part1.json");
static N5_PART2_STR: &str = include_str!("../../../../vocabulary/n5_part2.json");
static N5_PART3_STR: &str = include_str!("../../../../vocabulary/n5_part3.json");
static N4_PART1_STR: &str = include_str!("../../../../vocabulary/n4_part1.json");
static N4_PART2_STR: &str = include_str!("../../../../vocabulary/n4_part2.json");
static N3_PART1_STR: &str = include_str!("../../../../vocabulary/n3_part1.json");
static N3_PART2_STR: &str = include_str!("../../../../vocabulary/n3_part2.json");
static N2_PART1_STR: &str = include_str!("../../../../vocabulary/n2_part1.json");
static N2_PART2_STR: &str = include_str!("../../../../vocabulary/n2_part2.json");
static N1_PART1_STR: &str = include_str!("../../../../vocabulary/n1_part1.json");
static N1_PART2_STR: &str = include_str!("../../../../vocabulary/n1_part2.json");
static N1_PART3_STR: &str = include_str!("../../../../vocabulary/n1_part3.json");
static N1_PART4_STR: &str = include_str!("../../../../vocabulary/n1_part4.json");

impl VocabularyDatabase {
    pub fn new() -> Self {
        let vocabulary_data: HashMap<_, _> = serde_json::from_str::<
            HashMap<String, VocabularyEntryStoredType>,
        >(N1_PART1_STR)
        .unwrap()
        .into_iter()
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(N1_PART2_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(N1_PART3_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(N1_PART4_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(N2_PART1_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(N2_PART2_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(N3_PART1_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(N3_PART2_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(N4_PART1_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(N4_PART2_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(N5_PART1_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(N5_PART2_STR)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(N5_PART3_STR)
                .unwrap(),
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
