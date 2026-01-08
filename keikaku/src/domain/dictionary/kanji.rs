use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Serialize};

use crate::domain::{
    KeikakuError,
    dictionary::{
        radical::{RADICAL_DB, RadicalInfo},
        vocabulary::VOCABULARY_DB,
    },
    value_objects::{JapaneseLevel, NativeLanguage},
};

const KANJI_DATA: &str = include_str!("./kanji.json");
pub static KANJI_DB: LazyLock<KanjiDatabase> = LazyLock::new(KanjiDatabase::new);
pub struct KanjiDatabase {
    kanji_map: HashMap<String, KanjiInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PopularWord {
    word: String,
    translation: String,
}

impl PopularWord {
    pub fn new(word: String, translation: String) -> Self {
        Self { word, translation }
    }

    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn translation(&self) -> &str {
        &self.translation
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KanjiInfo {
    kanji: char,
    jlpt: JapaneseLevel,
    used_in: u32,
    description: String,
    radicals: Vec<char>,
    popular_words: Vec<String>,
}

impl KanjiInfo {
    pub fn kanji(&self) -> char {
        self.kanji
    }

    pub fn jlpt(&self) -> &JapaneseLevel {
        &self.jlpt
    }

    pub fn used_in(&self) -> u32 {
        self.used_in
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn radicals(&self) -> Vec<&RadicalInfo> {
        self.radicals
            .iter()
            .filter_map(|r| RADICAL_DB.get_radical_info(r).ok())
            .collect()
    }

    pub fn popular_words(&self) -> &[String] {
        &self.popular_words
    }

    pub fn popular_words_with_translations(
        &self,
        native_language: &NativeLanguage,
    ) -> Vec<PopularWord> {
        self.popular_words
            .iter()
            .map(|word| {
                let translation = VOCABULARY_DB
                    .get_translation(word, native_language)
                    .unwrap_or_else(|| "Перевод не найден".to_string());
                PopularWord::new(word.clone(), translation)
            })
            .collect()
    }
}

impl Default for KanjiDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl KanjiDatabase {
    pub fn new() -> Self {
        let kanji_db: KanjiDatabaseStoredType = serde_json::from_str(KANJI_DATA).unwrap();

        let kanji_map = kanji_db
            .kanji
            .into_iter()
            .map(|k| {
                let jlpt = parse_jlpt_level(&k.jlpt);
                let kanji_char = k.kanji.chars().next().unwrap();
                let radicals = k
                    .radicals
                    .into_iter()
                    .flat_map(|r| r.chars().collect::<Vec<_>>())
                    .collect::<Vec<char>>();

                (
                    kanji_char.to_string(),
                    KanjiInfo {
                        kanji: kanji_char,
                        jlpt,
                        used_in: k.used_in,
                        description: k.description,
                        radicals,
                        popular_words: k.popular_words,
                    },
                )
            })
            .collect::<HashMap<String, KanjiInfo>>();

        Self { kanji_map }
    }

    pub fn get_kanji_info(&self, kanji: &str) -> Result<&KanjiInfo, KeikakuError> {
        self.kanji_map
            .get(kanji)
            .ok_or(KeikakuError::KradfileError {
                reason: format!("Kanji {} not found in kanji database", kanji),
            })
    }
}

#[derive(Serialize, Deserialize)]
struct KanjiStoredType {
    kanji: String,
    jlpt: String,
    used_in: u32,
    description: String,
    radicals: Vec<String>,
    popular_words: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct KanjiDatabaseStoredType {
    kanji: Vec<KanjiStoredType>,
}

pub(crate) fn parse_jlpt_level(s: &str) -> JapaneseLevel {
    match s {
        "N5" => JapaneseLevel::N5,
        "N4" => JapaneseLevel::N4,
        "N3" => JapaneseLevel::N3,
        "N2" => JapaneseLevel::N2,
        "N1" => JapaneseLevel::N1,
        _ => JapaneseLevel::N1,
    }
}
