use std::{collections::HashMap, fs, sync::LazyLock};

use serde::{Deserialize, Serialize};

use crate::domain::value_objects::{Embedding, ExamplePhrase, JapaneseLevel, NativeLanguage};

pub static VOCABULARY_DB: LazyLock<VocabularyDatabase> = LazyLock::new(VocabularyDatabase::new);

#[derive(Debug, Clone)]
pub struct VocabularyInfo {
    pub word: String,
    pub level: JapaneseLevel,
    pub russian_translation: String,
    pub english_translation: String,
    pub russian_examples: Vec<ExamplePhrase>,
    pub english_examples: Vec<ExamplePhrase>,
    pub embedding: Embedding,
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
    embedding: Vec<f32>,
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
        let vocabulary_n5_str = fs::read_to_string("words/vocabulary_n5.json").unwrap();
        let vocabulary_n4_str = fs::read_to_string("words/vocabulary_n4.json").unwrap();
        let vocabulary_n3_str = fs::read_to_string("words/vocabulary_n3.json").unwrap();
        let vocabulary_n2_str = fs::read_to_string("words/vocabulary_n2.json").unwrap();
        let vocabulary_n1_str = fs::read_to_string("words/vocabulary_n1.json").unwrap();

        let vocabulary_data: HashMap<_, _> = serde_json::from_str::<
            HashMap<String, VocabularyEntryStoredType>,
        >(&vocabulary_n1_str)
        .unwrap()
        .into_iter()
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(&vocabulary_n2_str)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(&vocabulary_n3_str)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(&vocabulary_n4_str)
                .unwrap(),
        )
        .chain(
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(&vocabulary_n5_str)
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
                        embedding: Embedding(entry.embedding),
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

    pub fn get_embedding(&self, word: &str) -> Option<Embedding> {
        self.vocabulary_map
            .get(word)
            .map(|info| info.embedding.clone())
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
