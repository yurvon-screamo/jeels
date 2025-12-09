use crate::application::{MigiiClient, MigiiMeaning, MigiiWord};
use crate::domain::JeersError;
use crate::domain::value_objects::{JapaneseLevel, NativeLanguage};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MigiiResponse {
    data: Vec<MigiiWordDto>,
}

#[derive(Debug, Deserialize)]
struct MigiiWordDto {
    word: String,
    short_mean: String,
    mean: Vec<MigiiMeaningDto>,
}

#[derive(Debug, Deserialize)]
struct MigiiMeaningDto {
    mean: String,
}

pub struct HttpMigiiClient;

impl Default for HttpMigiiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpMigiiClient {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MigiiClient for HttpMigiiClient {
    async fn get_words(
        &self,
        native_lang: &NativeLanguage,
        level: &JapaneseLevel,
        lesson: u32,
    ) -> Result<Vec<MigiiWord>, JeersError> {
        let level_num = level.as_number() as u32;
        let native_lang_str = match native_lang {
            NativeLanguage::Russian => "ru",
            NativeLanguage::English => "en",
        };

        let url = format!(
            "https://jlpt.migii.net/api/theory/word/javi/{}/{}/{}",
            native_lang_str, level_num, lesson
        );

        let response = reqwest::get(&url)
            .await
            .map_err(|e| JeersError::RepositoryError {
                reason: format!("Failed to fetch Migii data: {}", e),
            })?;

        let migii_data: MigiiResponse =
            response
                .json()
                .await
                .map_err(|e| JeersError::RepositoryError {
                    reason: format!("Failed to parse Migii JSON: {}", e),
                })?;

        Ok(migii_data
            .data
            .into_iter()
            .map(|dto| MigiiWord {
                word: dto.word,
                short_mean: dto.short_mean,
                mean: dto
                    .mean
                    .into_iter()
                    .map(|m| MigiiMeaning { mean: m.mean })
                    .collect(),
            })
            .collect())
    }
}
