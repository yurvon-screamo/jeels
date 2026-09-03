use std::{
    collections::{BTreeMap, HashMap},
    sync::OnceLock,
};

use serde::Deserialize;

use crate::domain::{NativeLanguage, OrigaError};

pub static VOCABULARY_DICTIONARY: OnceLock<VocabularyDatabase> = OnceLock::new();

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum TranslationValue {
    Structured { t: Vec<String>, d: String },
    Raw(String),
}

impl TranslationValue {
    fn into_translations_and_description(self) -> (Vec<String>, Option<String>) {
        match self {
            TranslationValue::Structured { t, d } => {
                let desc = if d.trim().is_empty() {
                    None
                } else {
                    Some(d.trim().to_string())
                };
                let translations = split_semicolon_joined_translations(t);
                (translations, desc)
            },
            TranslationValue::Raw(s) => {
                tracing::warn!(
                    "Legacy vocabulary format detected, consider migrating to structured format"
                );
                parse_legacy_translation(&s)
            },
        }
    }
}

fn split_semicolon_joined_translations(translations: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(translations.len());
    for entry in translations {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.contains(';') {
            for part in trimmed.split(';') {
                let part = part.trim();
                if !part.is_empty() {
                    out.push(part.to_string());
                }
            }
        } else {
            out.push(trimmed.to_string());
        }
    }
    out
}

fn parse_legacy_translation(text: &str) -> (Vec<String>, Option<String>) {
    let mut translations = Vec::new();
    let mut description_parts = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(t) = trimmed.strip_prefix("- ") {
            if !t.is_empty() {
                translations.push(t.to_string());
            }
        } else if let Some(d) = trimmed.strip_prefix("> ") {
            if !d.is_empty() {
                description_parts.push(d.to_string());
            }
        } else if !translations.is_empty() {
            description_parts.push(trimmed.to_string());
        } else {
            translations.push(trimmed.to_string());
        }
    }

    if translations.is_empty() && !text.trim().is_empty() {
        translations.push(text.trim().to_string());
    }

    let description = if description_parts.is_empty() {
        None
    } else {
        Some(description_parts.join(" "))
    };
    (translations, description)
}

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct VocabularyInfo {
    word: String,
    ru_translations: Vec<String>,
    ru_description: Option<String>,
    en_translations: Vec<String>,
    en_description: Option<String>,
}

impl VocabularyInfo {
    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn russian_translation(&self) -> String {
        self.ru_translations
            .iter()
            .map(|t| format!("- {}", t))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn english_translation(&self) -> String {
        self.en_translations
            .iter()
            .map(|t| format!("- {}", t))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn ru_translations(&self) -> &[String] {
        &self.ru_translations
    }

    pub fn en_translations(&self) -> &[String] {
        &self.en_translations
    }

    pub fn ru_description(&self) -> Option<&str> {
        self.ru_description.as_deref()
    }

    pub fn en_description(&self) -> Option<&str> {
        self.en_description.as_deref()
    }

    pub fn translations(&self, lang: &NativeLanguage) -> &[String] {
        match lang {
            NativeLanguage::Russian => &self.ru_translations,
            NativeLanguage::English => &self.en_translations,
        }
    }

    pub fn description(&self, lang: &NativeLanguage) -> Option<&str> {
        match lang {
            NativeLanguage::Russian => self.ru_description.as_deref(),
            NativeLanguage::English => self.en_description.as_deref(),
        }
    }
}

#[derive(Deserialize)]
struct VocabularyEntryStoredType {
    russian_translation: Option<String>,
    english_translation: Option<String>,
    ru: Option<TranslationValue>,
    en: Option<TranslationValue>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct VocabularyDatabase {
    vocabulary_map: HashMap<String, VocabularyInfo>,
}

/// Deterministic serialization form of [`VocabularyDatabase`] for CDN blobs.
///
/// The inner `HashMap` iterates in a per-process random order, so serializing
/// the database directly would produce a byte-different blob on every build
/// run and invalidate every client cache after each deploy. `BTreeMap`
/// serialization is ordered, keeping the blob hash stable for unchanged
/// sources.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct VocabularyBlob {
    entries: BTreeMap<String, VocabularyInfo>,
}

impl VocabularyDatabase {
    /// Deterministic CDN-blob form of this database.
    pub fn to_cdn_blob(&self) -> VocabularyBlob {
        VocabularyBlob {
            entries: self.vocabulary_map.clone().into_iter().collect(),
        }
    }
}

impl VocabularyBlob {
    pub fn into_database(self) -> VocabularyDatabase {
        VocabularyDatabase {
            vocabulary_map: self.entries.into_iter().collect(),
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct VocabularyChunkData {
    pub chunk_01: String,
    pub chunk_02: String,
    pub chunk_03: String,
    pub chunk_04: String,
    pub chunk_05: String,
    pub chunk_06: String,
    pub chunk_07: String,
    pub chunk_08: String,
    pub chunk_09: String,
    pub chunk_10: String,
    pub chunk_11: String,
}

pub fn init_vocabulary(data: VocabularyChunkData) -> Result<(), OrigaError> {
    let db = VocabularyDatabase::from_chunks(data)?;
    set_vocabulary_database(db)
}

/// Install an already-built database into the global slot. Used by the CDN
/// blob builder (via `from_chunks`) and by loader wrappers after rkyv
/// deserialization. Fails if another database is already installed.
pub fn set_vocabulary_database(db: VocabularyDatabase) -> Result<(), OrigaError> {
    VOCABULARY_DICTIONARY
        .set(db)
        .map_err(|_| OrigaError::VocabularyParseError {
            reason: "Failed to set vocabulary dictionary".to_string(),
        })
}

/// Build a database from chunk JSONs without installing it into the global
/// slot. Used by the CDN blob builder and by loader tests.
pub fn build_vocabulary_database_from_chunks(
    data: VocabularyChunkData,
) -> Result<VocabularyDatabase, OrigaError> {
    VocabularyDatabase::from_chunks(data)
}

/// Serialize a CDN-blob form of an already-built database (deterministic).
pub fn serialize_vocabulary_blob_to_rkyv(db: &VocabularyDatabase) -> Result<Vec<u8>, OrigaError> {
    rkyv::to_bytes::<rkyv::rancor::Error>(&db.to_cdn_blob())
        .map(|bytes| bytes.to_vec())
        .map_err(|e| OrigaError::VocabularyParseError {
            reason: format!("Failed to serialize vocabulary blob: {}", e),
        })
}

/// Deserialize a CDN-blob payload into a database (fast path, no JSON).
pub fn vocabulary_database_from_blob_rkyv(
    payload: &[u8],
) -> Result<VocabularyDatabase, OrigaError> {
    let blob: VocabularyBlob = rkyv::from_bytes::<VocabularyBlob, rkyv::rancor::Error>(payload)
        .map_err(|e| OrigaError::VocabularyParseError {
            reason: format!("Failed to deserialize vocabulary blob: {}", e),
        })?;
    Ok(blob.into_database())
}

/// Serialize the in-memory VocabularyDatabase to rkyv bytes for Cache API
/// storage. On subsequent loads, `init_vocabulary_from_rkyv` skips JSON
/// parsing entirely.
pub fn serialize_vocabulary_to_rkyv() -> Result<Vec<u8>, OrigaError> {
    let db = VOCABULARY_DICTIONARY
        .get()
        .ok_or(OrigaError::VocabularyParseError {
            reason: "Vocabulary dictionary not loaded".to_string(),
        })?;
    rkyv::to_bytes::<rkyv::rancor::Error>(db)
        .map_err(|e| OrigaError::VocabularyParseError {
            reason: format!("Failed to serialize vocabulary: {}", e),
        })
        .map(|bytes| bytes.to_vec())
}

/// Initialize vocabulary from cached rkyv bytes (fast path).
/// Skips JSON parsing of 11 chunks (~35 MB text).
pub fn init_vocabulary_from_rkyv(bytes: &[u8]) -> Result<(), OrigaError> {
    let db: VocabularyDatabase = rkyv::from_bytes::<VocabularyDatabase, rkyv::rancor::Error>(bytes)
        .map_err(|e| OrigaError::VocabularyParseError {
            reason: format!("Failed to deserialize vocabulary: {}", e),
        })?;
    VOCABULARY_DICTIONARY
        .set(db)
        .map_err(|_| OrigaError::VocabularyParseError {
            reason: "Failed to set vocabulary dictionary".to_string(),
        })
}

pub fn is_vocabulary_loaded() -> bool {
    VOCABULARY_DICTIONARY.get().is_some()
}

pub fn get_translation(word: &str, native_language: &NativeLanguage) -> Option<String> {
    VOCABULARY_DICTIONARY
        .get()
        .and_then(|db| db.get_translation(word, native_language))
}

pub fn get_translations(word: &str, native_language: &NativeLanguage) -> Option<Vec<String>> {
    VOCABULARY_DICTIONARY
        .get()
        .and_then(|db| db.get_translations(word, native_language))
}

pub fn get_description(word: &str, native_language: &NativeLanguage) -> Option<String> {
    VOCABULARY_DICTIONARY
        .get()
        .and_then(|db| db.get_description(word, native_language))
}

fn resolve_translations(
    entry: &VocabularyEntryStoredType,
    lang: TranslationLang,
) -> (Vec<String>, Option<String>) {
    let (structured, raw) = match lang {
        TranslationLang::Ru => (&entry.ru, &entry.russian_translation),
        TranslationLang::En => (&entry.en, &entry.english_translation),
    };

    if let Some(tv) = structured {
        return tv.clone().into_translations_and_description();
    }

    if let Some(raw_str) = raw {
        return parse_legacy_translation(raw_str);
    }

    (vec![], None)
}

enum TranslationLang {
    Ru,
    En,
}

impl VocabularyDatabase {
    fn from_chunks(data: VocabularyChunkData) -> Result<Self, OrigaError> {
        fn strip_bom(json: &str) -> &str {
            json.strip_prefix('\u{FEFF}').unwrap_or(json)
        }

        let parse_chunk = |json: &str, chunk_name: &str| {
            serde_json::from_str::<HashMap<String, VocabularyEntryStoredType>>(strip_bom(json))
                .map_err(|e| OrigaError::VocabularyParseError {
                    reason: format!("Failed to parse {}: {}", chunk_name, e),
                })
        };

        let vocabulary_data: HashMap<_, _> = parse_chunk(&data.chunk_01, "chunk_01")?
            .into_iter()
            .chain(parse_chunk(&data.chunk_02, "chunk_02")?)
            .chain(parse_chunk(&data.chunk_03, "chunk_03")?)
            .chain(parse_chunk(&data.chunk_04, "chunk_04")?)
            .chain(parse_chunk(&data.chunk_05, "chunk_05")?)
            .chain(parse_chunk(&data.chunk_06, "chunk_06")?)
            .chain(parse_chunk(&data.chunk_07, "chunk_07")?)
            .chain(parse_chunk(&data.chunk_08, "chunk_08")?)
            .chain(parse_chunk(&data.chunk_09, "chunk_09")?)
            .chain(parse_chunk(&data.chunk_10, "chunk_10")?)
            .chain(parse_chunk(&data.chunk_11, "chunk_11")?)
            .collect();

        let vocabulary_map = vocabulary_data
            .into_iter()
            .map(|(word, entry)| {
                let (ru_translations, ru_description) =
                    resolve_translations(&entry, TranslationLang::Ru);
                let (en_translations, en_description) =
                    resolve_translations(&entry, TranslationLang::En);

                (
                    word.clone(),
                    VocabularyInfo {
                        word,
                        ru_translations,
                        ru_description,
                        en_translations,
                        en_description,
                    },
                )
            })
            .collect::<HashMap<String, VocabularyInfo>>();

        Ok(Self { vocabulary_map })
    }

    pub fn get_translation(&self, word: &str, native_language: &NativeLanguage) -> Option<String> {
        self.vocabulary_map
            .get(word)
            .map(|info| match native_language {
                NativeLanguage::Russian => info.russian_translation(),
                NativeLanguage::English => info.english_translation(),
            })
    }

    pub fn get_translations(
        &self,
        word: &str,
        native_language: &NativeLanguage,
    ) -> Option<Vec<String>> {
        self.vocabulary_map
            .get(word)
            .map(|info| info.translations(native_language).to_vec())
    }

    pub fn get_description(&self, word: &str, native_language: &NativeLanguage) -> Option<String> {
        self.vocabulary_map
            .get(word)
            .and_then(|info| info.description(native_language).map(|s| s.to_string()))
    }

    pub fn get_vocabulary_info(&self, word: &str) -> Option<&VocabularyInfo> {
        self.vocabulary_map.get(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::NativeLanguage;

    fn empty_chunk_data_with(chunk_01: &str) -> VocabularyChunkData {
        let empty = "{}".to_string();
        VocabularyChunkData {
            chunk_01: chunk_01.to_string(),
            chunk_02: empty.clone(),
            chunk_03: empty.clone(),
            chunk_04: empty.clone(),
            chunk_05: empty.clone(),
            chunk_06: empty.clone(),
            chunk_07: empty.clone(),
            chunk_08: empty.clone(),
            chunk_09: empty.clone(),
            chunk_10: empty.clone(),
            chunk_11: empty,
        }
    }

    fn make_valid_chunk_json() -> String {
        r#"{
            "猫": {
                "level": "N5",
                "russian_translation": "- кошка\n- кот",
                "english_translation": "- cat"
            },
            "犬": {
                "level": "N5",
                "russian_translation": "собака",
                "english_translation": "dog"
            }
        }"#
        .to_string()
    }

    fn make_structured_chunk_json() -> String {
        r#"{
            "猫": {
                "level": "N5",
                "ru": { "t": ["кошка", "кот"], "d": "домашнее животное" },
                "en": { "t": ["cat"], "d": "" }
            },
            "犬": {
                "level": "N5",
                "ru": { "t": ["собака"], "d": "" },
                "en": { "t": ["dog"], "d": "domestic animal" }
            }
        }"#
        .to_string()
    }

    fn make_mixed_chunk_json() -> String {
        r#"{
            "猫": {
                "level": "N5",
                "russian_translation": "- кошка\n- кот",
                "english_translation": "- cat"
            },
            "犬": {
                "level": "N5",
                "ru": { "t": ["собака"], "d": "" },
                "en": { "t": ["dog"], "d": "" }
            }
        }"#
        .to_string()
    }

    #[test]
    fn from_chunks_valid_json_loads_entries() {
        let data = empty_chunk_data_with(&make_valid_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();
        assert!(db.get_vocabulary_info("猫").is_some());
        assert!(db.get_vocabulary_info("犬").is_some());
        assert!(db.get_vocabulary_info("魚").is_none());
    }

    #[test]
    fn from_chunks_structured_format_loads_entries() {
        let data = empty_chunk_data_with(&make_structured_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();
        assert!(db.get_vocabulary_info("猫").is_some());
        assert!(db.get_vocabulary_info("犬").is_some());
    }

    #[test]
    fn from_chunks_mixed_formats_work() {
        let data = empty_chunk_data_with(&make_mixed_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        let cat_ru = db.get_translation("猫", &NativeLanguage::Russian).unwrap();
        assert!(cat_ru.contains("кошка"));

        let dog_ru = db.get_translation("犬", &NativeLanguage::Russian).unwrap();
        assert!(dog_ru.contains("собака"));
    }

    #[test]
    fn from_chunks_strips_bom_prefix() {
        let json_with_bom = format!("\u{FEFF}{}", make_valid_chunk_json());
        let data = empty_chunk_data_with(&json_with_bom);
        let db = VocabularyDatabase::from_chunks(data).unwrap();
        assert!(db.get_vocabulary_info("猫").is_some());
    }

    #[test]
    fn from_chunks_empty_all_chunks_succeeds() {
        let empty = "{}".to_string();
        let data = VocabularyChunkData {
            chunk_01: empty.clone(),
            chunk_02: empty.clone(),
            chunk_03: empty.clone(),
            chunk_04: empty.clone(),
            chunk_05: empty.clone(),
            chunk_06: empty.clone(),
            chunk_07: empty.clone(),
            chunk_08: empty.clone(),
            chunk_09: empty.clone(),
            chunk_10: empty.clone(),
            chunk_11: empty,
        };
        let db = VocabularyDatabase::from_chunks(data).unwrap();
        assert!(db.get_vocabulary_info("anything").is_none());
    }

    #[test]
    fn get_translation_found_returns_correct_language_legacy() {
        let data = empty_chunk_data_with(&make_valid_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        let ru = db.get_translation("猫", &NativeLanguage::Russian).unwrap();
        let en = db.get_translation("猫", &NativeLanguage::English).unwrap();

        assert!(ru.contains("кошка"));
        assert!(ru.contains("кот"));
        assert!(en.contains("cat"));
    }

    #[test]
    fn get_translation_found_returns_correct_language_structured() {
        let data = empty_chunk_data_with(&make_structured_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        let ru = db.get_translation("猫", &NativeLanguage::Russian).unwrap();
        let en = db.get_translation("猫", &NativeLanguage::English).unwrap();

        assert!(ru.contains("кошка"));
        assert!(ru.contains("кот"));
        assert!(en.contains("cat"));
    }

    #[test]
    fn get_translation_not_found_returns_none() {
        let data = empty_chunk_data_with(&make_valid_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();
        let result = db.get_translation("魚", &NativeLanguage::Russian);
        assert!(result.is_none());
    }

    #[test]
    fn get_translations_returns_vec() {
        let data = empty_chunk_data_with(&make_structured_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        let ru = db.get_translations("猫", &NativeLanguage::Russian).unwrap();
        assert_eq!(ru, vec!["кошка", "кот"]);

        let en = db.get_translations("猫", &NativeLanguage::English).unwrap();
        assert_eq!(en, vec!["cat"]);
    }

    #[test]
    fn get_translations_returns_vec_legacy_format() {
        let data = empty_chunk_data_with(&make_valid_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        let ru = db.get_translations("猫", &NativeLanguage::Russian).unwrap();
        assert_eq!(ru, vec!["кошка", "кот"]);
    }

    #[test]
    fn get_description_returns_some() {
        let data = empty_chunk_data_with(&make_structured_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        let ru_desc = db.get_description("猫", &NativeLanguage::Russian);
        assert_eq!(ru_desc, Some("домашнее животное".to_string()));

        let en_desc = db.get_description("犬", &NativeLanguage::English);
        assert_eq!(en_desc, Some("domestic animal".to_string()));
    }

    #[test]
    fn get_description_returns_none_for_empty() {
        let data = empty_chunk_data_with(&make_structured_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        let en_desc = db.get_description("猫", &NativeLanguage::English);
        assert!(en_desc.is_none());

        let ru_desc = db.get_description("犬", &NativeLanguage::Russian);
        assert!(ru_desc.is_none());
    }

    #[test]
    fn get_description_returns_none_for_legacy_format() {
        let data = empty_chunk_data_with(&make_valid_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        let desc = db.get_description("猫", &NativeLanguage::Russian);
        assert!(desc.is_none());
    }

    #[test]
    fn from_chunks_invalid_json_returns_error() {
        let data = empty_chunk_data_with("not valid json");
        let result = VocabularyDatabase::from_chunks(data);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(OrigaError::VocabularyParseError { .. })
        ));
    }

    #[test]
    fn parse_legacy_translation_dash_prefixed() {
        let (translations, desc) = parse_legacy_translation("- кошка\n- кот");
        assert_eq!(translations, vec!["кошка", "кот"]);
        assert!(desc.is_none());
    }

    #[test]
    fn parse_legacy_translation_plain_text() {
        let (translations, desc) = parse_legacy_translation("собака");
        assert_eq!(translations, vec!["собака"]);
        assert!(desc.is_none());
    }

    #[test]
    fn parse_legacy_translation_with_description() {
        let (translations, desc) = parse_legacy_translation("- кошка\n> домашнее животное");
        assert_eq!(translations, vec!["кошка"]);
        assert_eq!(desc, Some("домашнее животное".to_string()));
    }

    #[test]
    fn parse_legacy_translation_empty_string() {
        let (translations, desc) = parse_legacy_translation("");
        assert!(translations.is_empty());
        assert!(desc.is_none());
    }

    #[test]
    fn structured_format_splits_semicolon_joined_translations() {
        let data = empty_chunk_data_with(
            r#"{
                "意思": {
                    "level": "N5",
                    "ru": { "t": ["намерение; Воля; Цель", "смысл; Значение; Суть"], "d": "" },
                    "en": { "t": ["intention; Will; Purpose"], "d": "" }
                }
            }"#,
        );
        let db = VocabularyDatabase::from_chunks(data).unwrap();
        let ru = db
            .get_translations("意思", &NativeLanguage::Russian)
            .unwrap();
        assert_eq!(
            ru,
            vec!["намерение", "Воля", "Цель", "смысл", "Значение", "Суть"]
        );
        let en = db
            .get_translations("意思", &NativeLanguage::English)
            .unwrap();
        assert_eq!(en, vec!["intention", "Will", "Purpose"]);
    }

    #[test]
    fn split_semicolon_joined_translations_handles_clean_input() {
        let out = split_semicolon_joined_translations(vec!["кошка".to_string(), "кот".to_string()]);
        assert_eq!(out, vec!["кошка", "кот"]);
    }

    #[test]
    fn split_semicolon_joined_translations_drops_empty_parts() {
        let out = split_semicolon_joined_translations(vec!["кошка;;кот;".to_string()]);
        assert_eq!(out, vec!["кошка", "кот"]);
    }

    #[test]
    fn split_semicolon_joined_translations_drops_empty_entries() {
        let out = split_semicolon_joined_translations(vec!["кошка".to_string(), "   ".to_string()]);
        assert_eq!(out, vec!["кошка"]);
    }

    #[test]
    fn rkyv_vocabulary_round_trip() {
        let data = empty_chunk_data_with(&make_structured_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();

        // Serialize directly (bypass OnceLock — serialize_vocabulary_to_rkyv
        // requires a populated VOCABULARY_DICTIONARY which OnceLock prevents
        // in tests).
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&db).expect("serialize should succeed");
        assert!(!bytes.is_empty());

        // Deserialize back.
        let restored: VocabularyDatabase =
            rkyv::from_bytes::<VocabularyDatabase, rkyv::rancor::Error>(&bytes).unwrap();

        let ru = restored
            .get_translations("猫", &NativeLanguage::Russian)
            .unwrap();
        assert_eq!(ru, vec!["кошка", "кот"]);
        let en = restored
            .get_translations("猫", &NativeLanguage::English)
            .unwrap();
        assert_eq!(en, vec!["cat"]);
        let desc = restored.get_description("猫", &NativeLanguage::Russian);
        assert_eq!(desc, Some("домашнее животное".to_string()));
    }

    #[test]
    fn cdn_blob_round_trip_preserves_translations() {
        // Arrange
        let data = empty_chunk_data_with(&make_structured_chunk_json());
        let db = build_vocabulary_database_from_chunks(data).unwrap();

        // Act
        let payload = serialize_vocabulary_blob_to_rkyv(&db).unwrap();
        let restored = vocabulary_database_from_blob_rkyv(&payload).unwrap();

        // Assert
        let ru = restored
            .get_translations("猫", &NativeLanguage::Russian)
            .unwrap();
        assert_eq!(ru, vec!["кошка", "кот"]);
        let en = restored
            .get_translations("猫", &NativeLanguage::English)
            .unwrap();
        assert_eq!(en, vec!["cat"]);
    }

    #[test]
    fn cdn_blob_serialization_is_deterministic_across_builds() {
        // Arrange: two independently built databases from the same chunks
        let json = make_structured_chunk_json();
        let first = build_vocabulary_database_from_chunks(empty_chunk_data_with(&json)).unwrap();
        let second = build_vocabulary_database_from_chunks(empty_chunk_data_with(&json)).unwrap();

        // Act
        let first_payload = serialize_vocabulary_blob_to_rkyv(&first).unwrap();
        let second_payload = serialize_vocabulary_blob_to_rkyv(&second).unwrap();

        // Assert: byte-identical blobs keep the manifest hash stable across
        // deploys with unchanged sources (no mass client re-download).
        assert_eq!(first_payload, second_payload);
    }

    #[test]
    fn rkyv_vocabulary_round_trip_preserves_legacy_format() {
        let data = empty_chunk_data_with(&make_valid_chunk_json());
        let db = VocabularyDatabase::from_chunks(data).unwrap();
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&db).expect("serialize should succeed");

        let restored: VocabularyDatabase =
            rkyv::from_bytes::<VocabularyDatabase, rkyv::rancor::Error>(&bytes).unwrap();
        let ru = restored
            .get_translations("猫", &NativeLanguage::Russian)
            .unwrap();
        assert_eq!(ru, vec!["кошка", "кот"]);
    }
}
