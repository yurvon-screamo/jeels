use crate::domain::{Card, OrigaError, VocabularyCard};
use crate::traits::UserRepository;
use rusqlite::Connection;
use serde_json::Value;
use std::io::{Cursor, Read};
use tracing::{debug, info, warn};
use zip::ZipArchive;

const ANKI_DB_FILES: &[&str] = &[
    "collection.anki21b",
    "collection.anki21",
    "collection.anki2",
];
const FIELD_SEP: char = '\x1f';

#[derive(Debug, Clone)]
pub struct AnkiCard {
    pub word: String,
    pub translation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AnkiFieldInfo {
    pub name: String,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct AnkiDeckInfo {
    pub detected_fields: Vec<AnkiFieldInfo>,
}

pub struct ImportAnkiPackResult {
    pub total_created_count: usize,
    pub skipped_words: Vec<String>,
}

pub fn extract_anki_db_bytes(data: &[u8]) -> Result<Vec<u8>, OrigaError> {
    let cursor = Cursor::new(data);
    let mut archive = ZipArchive::new(cursor).map_err(|e| OrigaError::AnkiInvalidFile {
        reason: format!("Failed to read ZIP archive: {}", e),
    })?;
    for &db_file in ANKI_DB_FILES {
        if let Ok(mut entry) = archive.by_name(db_file) {
            let mut bytes = Vec::new();
            Read::read_to_end(&mut entry, &mut bytes).map_err(|e| OrigaError::AnkiInvalidFile {
                reason: format!("Failed to read '{}': {}", db_file, e),
            })?;
            if !bytes.is_empty() {
                return Ok(bytes);
            }
        }
    }
    Err(OrigaError::AnkiDatabaseNotFound {
        filename: "collection.anki21/anki21b/anki2".to_string(),
    })
}

pub fn read_anki_database(db_bytes: &[u8]) -> Result<AnkiDeckInfo, OrigaError> {
    let conn = open_db(db_bytes)?;
    let models_json = query_models(&conn)?;
    let models: Value =
        serde_json::from_str(&models_json).map_err(|e| OrigaError::AnkiInvalidFile {
            reason: format!("Failed to parse models JSON: {}", e),
        })?;
    let detected_fields = detect_fields(&models)?;
    Ok(AnkiDeckInfo { detected_fields })
}

pub fn parse_cards(
    models: &Value,
    notes_fields: &[String],
    word_tag: &str,
    translation_tag: Option<&str>,
) -> Result<Vec<AnkiCard>, OrigaError> {
    let (word_idx, trans_idx) = find_field_indices(models, word_tag, translation_tag)?;
    let mut cards = Vec::new();
    for flds in notes_fields {
        let (word, translation) = parse_fields(flds, word_idx, trans_idx);
        if !word.is_empty() {
            cards.push(AnkiCard { word, translation });
        }
    }
    Ok(cards)
}

pub fn extract_cards(
    data: &[u8],
    word_tag: &str,
    translation_tag: Option<&str>,
) -> Result<(Vec<AnkiCard>, Vec<AnkiFieldInfo>), OrigaError> {
    let db_bytes = extract_anki_db_bytes(data)?;
    let conn = open_db(&db_bytes)?;
    let models_json = query_models(&conn)?;
    let notes_fields = query_notes(&conn)?;
    let models: Value =
        serde_json::from_str(&models_json).map_err(|e| OrigaError::AnkiInvalidFile {
            reason: format!("Failed to parse models JSON: {}", e),
        })?;
    let detected_fields = detect_fields(&models)?;
    let cards = parse_cards(&models, &notes_fields, word_tag, translation_tag)?;
    Ok((cards, detected_fields))
}

pub struct ImportAnkiPackUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> ImportAnkiPackUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, cards: Vec<AnkiCard>) -> Result<ImportAnkiPackResult, OrigaError> {
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        let mut total_created = 0;
        let mut skipped = Vec::new();

        // Bulk bracket: an Anki pack can contain thousands of cards; the
        // index-backed uniqueness check + deferred stats recalc keep the
        // import linear (an early `return Err` below discards the unsaved
        // local user, so the unclosed bracket has no effect).
        user.begin_bulk_import();

        for anki_card in cards {
            let question = anki_card.word.clone();
            let result = VocabularyCard::from_text(&question, user.native_language());

            for vocab_card in result.cards {
                let card = Card::Vocabulary(vocab_card);
                match user.create_card(card) {
                    Ok(_) => total_created += 1,
                    Err(OrigaError::DuplicateCard { question: q }) => {
                        debug!(word = %q, "Duplicate card, skipping");
                        skipped.push(q);
                    },
                    Err(e) => return Err(e),
                }
            }
            for s in &result.skipped_no_translation {
                warn!(word = %s, "Translation not found, skipping");
                skipped.push(s.clone());
            }
        }

        user.end_bulk_import();

        self.repository.save_sync(&user).await?;

        info!(
            created = total_created,
            skipped = skipped.len(),
            "Anki import completed"
        );

        Ok(ImportAnkiPackResult {
            total_created_count: total_created,
            skipped_words: skipped,
        })
    }
}

fn open_db(db_bytes: &[u8]) -> Result<Connection, OrigaError> {
    let mut conn = Connection::open_in_memory().map_err(|e| OrigaError::AnkiInvalidFile {
        reason: format!("Failed to create in-memory database: {}", e),
    })?;
    let sz = db_bytes.len();
    conn.deserialize_read_exact("main", Cursor::new(db_bytes), sz, true)
        .map_err(|e| OrigaError::AnkiInvalidFile {
            reason: format!("Failed to load database from bytes: {}", e),
        })?;
    Ok(conn)
}

fn query_models(conn: &Connection) -> Result<String, OrigaError> {
    let mut stmt =
        conn.prepare("SELECT models FROM col")
            .map_err(|e| OrigaError::AnkiInvalidFile {
                reason: format!("Failed to query models: {}", e),
            })?;
    stmt.query_row([], |row| row.get(0))
        .map_err(|e| OrigaError::AnkiInvalidFile {
            reason: format!("Failed to read models: {}", e),
        })
}

fn query_notes(conn: &Connection) -> Result<Vec<String>, OrigaError> {
    let mut stmt =
        conn.prepare("SELECT flds FROM notes")
            .map_err(|e| OrigaError::AnkiInvalidFile {
                reason: format!("Failed to query notes: {}", e),
            })?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| OrigaError::AnkiInvalidFile {
            reason: format!("Failed to read notes: {}", e),
        })?;
    let mut fields = Vec::new();
    for row in rows {
        match row {
            Ok(flds) => fields.push(flds),
            Err(e) => warn!("Failed to read note field: {}", e),
        }
    }
    Ok(fields)
}

fn detect_fields(models: &Value) -> Result<Vec<AnkiFieldInfo>, OrigaError> {
    let mut seen = std::collections::HashSet::new();
    let mut fields = Vec::new();
    if let Some(map) = models.as_object() {
        for (_id, data) in map {
            if let Some(flds) = data["flds"].as_array() {
                for (i, f) in flds.iter().enumerate() {
                    if let Some(name) = f["name"].as_str() {
                        if seen.insert(name.to_string()) {
                            fields.push(AnkiFieldInfo {
                                name: name.to_string(),
                                index: i,
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(fields)
}

fn find_field_indices(
    models: &Value,
    word_tag: &str,
    translation_tag: Option<&str>,
) -> Result<(usize, Option<usize>), OrigaError> {
    if let Some(map) = models.as_object() {
        for (_id, data) in map {
            if let Some(flds) = data["flds"].as_array() {
                let mut word_idx = None;
                let mut trans_idx = None;

                for (i, f) in flds.iter().enumerate() {
                    if let Some(name) = f["name"].as_str() {
                        let lower = name.to_lowercase();
                        if lower == word_tag.to_lowercase() {
                            word_idx = Some(i);
                        }
                        if let Some(tag) = translation_tag {
                            if lower == tag.to_lowercase() {
                                trans_idx = Some(i);
                            }
                        }
                    }
                }

                if let Some(idx) = word_idx
                    && (translation_tag.is_none() || trans_idx.is_some())
                {
                    return Ok((idx, trans_idx));
                }
            }
        }
    }
    Err(OrigaError::AnkiFieldNotFound {
        field_name: word_tag.to_string(),
    })
}

fn parse_fields(flds: &str, word_idx: usize, trans_idx: Option<usize>) -> (String, Option<String>) {
    let fields: Vec<&str> = flds.split(FIELD_SEP).collect();
    let word = clean_html(fields.get(word_idx).unwrap_or(&""));
    let translation = trans_idx.and_then(|idx| {
        let cleaned = clean_html(fields.get(idx).unwrap_or(&""));
        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        }
    });
    (word, translation)
}

fn clean_html(raw: &str) -> String {
    let mut result = String::with_capacity(raw.len());
    let mut in_tag = false;
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '<' => {
                in_tag = true;
                result.push(' ');
                i += 1;
            },
            '>' if in_tag => {
                in_tag = false;
                i += 1;
            },
            _ if in_tag => {
                i += 1;
            },
            '&' => {
                if let Some((entity, len)) = try_parse_html_entity(&chars[i..]) {
                    result.push_str(&entity);
                    i += len;
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            },
            _ => {
                result.push(chars[i]);
                i += 1;
            },
        }
    }

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn try_parse_html_entity(chars: &[char]) -> Option<(String, usize)> {
    if chars.len() >= 6 && chars[0..6] == ['&', 'n', 'b', 's', 'p', ';'] {
        return Some((" ".to_string(), 6));
    }
    if chars.len() >= 5 && chars[0..5] == ['&', 'a', 'm', 'p', ';'] {
        return Some(("&".to_string(), 5));
    }
    if chars.len() >= 4 && chars[0..4] == ['&', 'l', 't', ';'] {
        return Some(("<".to_string(), 4));
    }
    if chars.len() >= 4 && chars[0..4] == ['&', 'g', 't', ';'] {
        return Some((">".to_string(), 4));
    }
    if chars.len() >= 6 && chars[0..6] == ['&', 'q', 'u', 'o', 't', ';'] {
        return Some(("\"".to_string(), 6));
    }
    if chars.len() >= 6 && chars[0..6] == ['&', 'a', 'p', 'o', 's', ';'] {
        return Some(("'".to_string(), 6));
    }
    if chars.len() >= 4 && chars[0] == '&' && chars[1] == '#' {
        let mut num_str = String::new();
        let mut j = 2;
        while j < chars.len() && chars[j].is_ascii_digit() {
            num_str.push(chars[j]);
            j += 1;
        }
        if j < chars.len() && chars[j] == ';' && !num_str.is_empty() {
            if let Ok(code_point) = num_str.parse::<u32>() {
                if let Some(ch) = char::from_u32(code_point) {
                    return Some((ch.to_string(), j + 1));
                }
            }
        }
    }
    None
}
