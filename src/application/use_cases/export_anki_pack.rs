use crate::application::{CreateCardUseCase, EmbeddingService, LlmService, UserRepository};
use crate::domain::error::JeersError;
use regex::Regex;
use rusqlite::Connection;
use serde_json::Value;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;
use ulid::Ulid;
use zip::ZipArchive;

const ANKI_DATABASE_FILE: &str = "collection.anki21";
const FIELD_SEPARATOR: char = '\x1f';

#[derive(Debug, Clone)]
pub struct AnkiCard {
    pub word: String,
    pub translation: Option<String>,
}

pub struct ExportAnkiPackResult {
    pub total_created_count: usize,
    pub skipped_words: Vec<String>,
}

pub struct ExportAnkiPackUseCase<'a, R: UserRepository, E: EmbeddingService, L: LlmService> {
    repository: &'a R,
    create_card_use_case: CreateCardUseCase<'a, R, E, L>,
}

impl<'a, R: UserRepository, E: EmbeddingService, L: LlmService> ExportAnkiPackUseCase<'a, R, E, L> {
    pub fn new(repository: &'a R, embedding_service: &'a E, llm_service: &'a L) -> Self {
        Self {
            repository,
            create_card_use_case: CreateCardUseCase::new(
                repository,
                embedding_service,
                llm_service,
            ),
        }
    }

    pub async fn extract_cards(
        &self,
        file_path: &str,
        word_tag: &str,
        translation_tag: Option<&str>,
    ) -> Result<Vec<AnkiCard>, JeersError> {
        let bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| JeersError::RepositoryError {
                reason: format!("Failed to read file: {}", e),
            })?;

        Self::extract_anki_cards(&bytes[..], word_tag, translation_tag).map_err(|e| {
            JeersError::RepositoryError {
                reason: format!("Failed to extract Anki cards: {}", e),
            }
        })
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        file_path: String,
        word_tag: String,
        translation_tag: Option<String>,
    ) -> Result<ExportAnkiPackResult, JeersError> {
        self.repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let cards = self
            .extract_cards(&file_path, &word_tag, translation_tag.as_deref())
            .await?;

        self.create_cards_from_anki_cards(user_id, cards).await
    }

    async fn create_cards_from_anki_cards(
        &self,
        user_id: Ulid,
        cards: Vec<AnkiCard>,
    ) -> Result<ExportAnkiPackResult, JeersError> {
        let mut total_created_count = 0;
        let mut total_skipped_words = Vec::new();

        for anki_card in cards {
            let question = anki_card.word.clone();
            let answer = anki_card.translation.clone();

            match self
                .create_card_use_case
                .execute(user_id, question.clone(), answer, None)
                .await
            {
                Ok(_) => {
                    total_created_count += 1;
                }
                Err(JeersError::DuplicateCard { .. }) => {
                    total_skipped_words.push(question);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(ExportAnkiPackResult {
            total_created_count,
            skipped_words: total_skipped_words,
        })
    }

    fn extract_anki_cards(
        data: &[u8],
        word_tag: &str,
        translation_tag: Option<&str>,
    ) -> Result<Vec<AnkiCard>, Box<dyn std::error::Error>> {
        let db_path = Self::extract_database_from_zip(data)?;
        let conn = Connection::open(&db_path)?;
        let (word_index, translation_index) =
            Self::find_field_indices(&conn, word_tag, translation_tag)?;
        let cards = Self::read_cards_from_database(&conn, word_index, translation_index)?;
        Ok(cards)
    }

    fn extract_database_from_zip(data: &[u8]) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)?;
        let mut db_file_entry = archive.by_name(ANKI_DATABASE_FILE)?;

        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join(ANKI_DATABASE_FILE);
        let mut temp_db_file = File::create(&db_path)?;

        std::io::copy(&mut db_file_entry, &mut temp_db_file)?;
        Ok(db_path)
    }

    fn find_field_indices(
        conn: &Connection,
        word_tag: &str,
        translation_tag: Option<&str>,
    ) -> Result<(usize, Option<usize>), Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare("SELECT models FROM col")?;
        let json_str: String = stmt.query_row([], |row| row.get(0))?;
        let models: Value = serde_json::from_str(&json_str)?;

        let mut word_index = None;
        let mut translation_index = None;

        if let Some(models_map) = models.as_object() {
            for (_model_id, model_data) in models_map {
                if let Some(fields) = model_data["flds"].as_array() {
                    for (index, field) in fields.iter().enumerate() {
                        if let Some(field_name) = field["name"].as_str() {
                            let field_name_lower = field_name.to_lowercase();
                            if field_name_lower == word_tag.to_lowercase() {
                                word_index = Some(index);
                            }
                            if let Some(trans_tag) = translation_tag
                                && field_name_lower == trans_tag.to_lowercase()
                            {
                                translation_index = Some(index);
                            }
                        }
                    }

                    if word_index.is_some()
                        && (translation_tag.is_none() || translation_index.is_some())
                    {
                        break;
                    }
                }
            }
        }

        let word_index = word_index
            .ok_or_else(|| format!("Field '{}' not found in Anki deck models", word_tag))?;

        Ok((word_index, translation_index))
    }

    fn read_cards_from_database(
        conn: &Connection,
        word_index: usize,
        translation_index: Option<usize>,
    ) -> Result<Vec<AnkiCard>, Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare("SELECT flds FROM notes")?;
        let rows = stmt.query_map([], |row| {
            let flds: String = row.get(0)?;
            Ok(flds)
        })?;

        let re_html = Regex::new(r"<[^>]*>")?;
        let re_nbsp = Regex::new(r"&nbsp;")?;

        let mut cards = Vec::new();

        for row in rows {
            let flds_str = row?;
            let fields: Vec<&str> = flds_str.split(FIELD_SEPARATOR).collect();

            let raw_word = fields.get(word_index).unwrap_or(&"");
            let word = Self::clean_html_text(raw_word, &re_html, &re_nbsp);

            let translation = if let Some(translation_index) = translation_index {
                let raw_translation = fields.get(translation_index).unwrap_or(&"");
                Some(Self::clean_html_text(raw_translation, &re_html, &re_nbsp))
            } else {
                None
            };

            if !word.is_empty() {
                cards.push(AnkiCard { word, translation });
            }
        }

        Ok(cards)
    }

    fn clean_html_text(raw: &str, re_html: &Regex, re_nbsp: &Regex) -> String {
        let no_html = re_html.replace_all(raw, " ");
        let no_nbsp = re_nbsp.replace_all(&no_html, " ");
        no_nbsp.trim().to_string()
    }
}
