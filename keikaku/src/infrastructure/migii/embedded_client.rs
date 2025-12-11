use std::collections::HashMap;
use std::sync::OnceLock;

use crate::application::{MigiiClient, MigiiMeaning, MigiiWord};
use crate::domain::JeersError;
use crate::domain::value_objects::{JapaneseLevel, NativeLanguage};
use async_trait::async_trait;

type MigiiLessons = HashMap<String, Vec<String>>;
type MigiiLevels = HashMap<String, MigiiLessons>;

fn load_embedded_data() -> Result<&'static MigiiLevels, JeersError> {
    static DATA: OnceLock<Result<MigiiLevels, String>> = OnceLock::new();

    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("migii_words.json"))
            .map_err(|e| format!("Failed to parse embedded Migii JSON: {e}"))
    })
    .as_ref()
    .map_err(|reason| JeersError::RepositoryError {
        reason: reason.clone(),
    })
}

fn level_key(level: &JapaneseLevel) -> &'static str {
    match level {
        JapaneseLevel::N5 => "n5",
        JapaneseLevel::N4 => "n4",
        JapaneseLevel::N3 => "n3",
        JapaneseLevel::N2 => "n2",
        JapaneseLevel::N1 => "n1",
    }
}

fn lesson_key(lesson: u32) -> String {
    format!("lesson_{}", lesson)
}

pub struct EmbeddedMigiiClient;

impl EmbeddedMigiiClient {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmbeddedMigiiClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MigiiClient for EmbeddedMigiiClient {
    async fn get_words(
        &self,
        _native_lang: &NativeLanguage,
        level: &JapaneseLevel,
        lesson: u32,
    ) -> Result<Vec<MigiiWord>, JeersError> {
        let data = load_embedded_data()?;

        let lessons = data
            .get(level_key(level))
            .ok_or_else(|| JeersError::RepositoryError {
                reason: format!("Level {:?} not found in embedded Migii data", level),
            })?;

        let lesson_words =
            lessons
                .get(&lesson_key(lesson))
                .ok_or_else(|| JeersError::RepositoryError {
                    reason: format!("Lesson {} not found in embedded Migii data", lesson),
                })?;

        Ok(lesson_words
            .iter()
            .map(|word| MigiiWord {
                word: word.clone(),
                short_mean: word.clone(),
                mean: vec![MigiiMeaning { mean: word.clone() }],
            })
            .collect())
    }
}
