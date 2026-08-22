use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError};

pub static RADICAL_DICTIONARY: OnceLock<RadicalDatabase> = OnceLock::new();

#[derive(Clone, Serialize, Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct RadicalData {
    pub radicals_json: String,
}

pub fn init_radicals(data: RadicalData) -> Result<(), OrigaError> {
    if is_radicals_loaded() {
        return Ok(());
    }

    let db = RadicalDatabase::from_json(&data.radicals_json)?;
    RADICAL_DICTIONARY.set(db).ok();
    Ok(())
}

/// Serialize RadicalData to rkyv bytes
pub fn serialize_radicals_to_rkyv(data: &RadicalData) -> Result<Vec<u8>, OrigaError> {
    let bytes =
        rkyv::to_bytes::<rkyv::rancor::Error>(data).map_err(|e| OrigaError::KradfileError {
            reason: format!("Failed to serialize radicals: {}", e),
        })?;
    Ok(bytes.to_vec())
}

/// Initialize radicals from rkyv bytes
pub fn init_radicals_from_rkyv(bytes: &[u8]) -> Result<(), OrigaError> {
    let archived =
        rkyv::access::<ArchivedRadicalData, rkyv::rancor::Error>(bytes).map_err(|e| {
            OrigaError::KradfileError {
                reason: format!("Failed to validate radicals data: {:?}", e),
            }
        })?;

    let data = RadicalData {
        radicals_json: archived.radicals_json.as_str().to_string(),
    };

    init_radicals(data)
}

pub fn is_radicals_loaded() -> bool {
    RADICAL_DICTIONARY.get().is_some()
}

pub fn get_radical_info(radical: char) -> Result<&'static RadicalInfo, OrigaError> {
    RADICAL_DICTIONARY
        .get()
        .ok_or_else(|| {
            debug!(radical = %radical, "Radical dictionary not loaded");
            OrigaError::KradfileError {
                reason: "Radical dictionary not loaded".to_string(),
            }
        })?
        .get_radical_info(&radical)
}

pub fn get_radical_list() -> Vec<RadicalInfo> {
    RADICAL_DICTIONARY
        .get()
        .map(|db| db.radical_list())
        .unwrap_or_default()
}

pub struct RadicalDatabase {
    known_radicals: Vec<char>,
    radical_map: HashMap<char, RadicalInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadicalInfo {
    radical: char,
    stroke_count: u32,
    name: String,
    description: String,
    /// English projection of `name`; empty until the CDN data ships it.
    name_en: Option<String>,
    /// English projection of `description`; empty until the CDN data ships it.
    description_en: Option<String>,
    jlpt: JapaneseLevel,
    kanji: Vec<char>,
}

impl RadicalInfo {
    pub fn radical(&self) -> char {
        self.radical
    }

    pub fn stroke_count(&self) -> u32 {
        self.stroke_count
    }

    /// Localized radical name. An empty string means "no translation for this
    /// language yet" (legacy CDN data without the `_en` fields) — callers hide
    /// empty copy and degrade to the bare radical symbol instead of showing
    /// Russian to an English user.
    pub fn name(&self, lang: &NativeLanguage) -> &str {
        match lang {
            NativeLanguage::English => self.name_en.as_deref().unwrap_or(""),
            NativeLanguage::Russian => &self.name,
        }
    }

    /// Localized radical description; see [`RadicalInfo::name`] for the
    /// empty-string fallback contract.
    pub fn description(&self, lang: &NativeLanguage) -> &str {
        match lang {
            NativeLanguage::English => self.description_en.as_deref().unwrap_or(""),
            NativeLanguage::Russian => &self.description,
        }
    }

    pub fn jlpt(&self) -> &JapaneseLevel {
        &self.jlpt
    }

    pub fn kanji(&self) -> &[char] {
        &self.kanji
    }
}

impl RadicalDatabase {
    pub fn from_json(json: &str) -> Result<Self, OrigaError> {
        let radkfile: RadkfilStoredType =
            serde_json::from_str(json).map_err(|e| OrigaError::KradfileError {
                reason: format!("Failed to parse radicals.json: {}", e),
            })?;
        let radical_map = radkfile
            .radicals
            .into_iter()
            .map(|(k, v)| {
                let radical_char = k.chars().next().unwrap();
                let jlpt = JapaneseLevel::from_str_or_default(&v.jlpt);
                let kanji = v
                    .kanji
                    .into_iter()
                    .flat_map(|c| c.chars().collect::<Vec<_>>())
                    .collect::<Vec<char>>();

                (
                    radical_char,
                    RadicalInfo {
                        radical: radical_char,
                        stroke_count: v.stroke_count,
                        name: v.name,
                        description: v.description,
                        name_en: v.name_en,
                        description_en: v.description_en,
                        jlpt,
                        kanji,
                    },
                )
            })
            .collect::<HashMap<char, RadicalInfo>>();

        let known_radicals: Vec<_> = radical_map.keys().copied().collect();

        Ok(Self {
            known_radicals,
            radical_map,
        })
    }

    pub fn get_radical_info(&self, radical: &char) -> Result<&RadicalInfo, OrigaError> {
        self.radical_map.get(radical).ok_or_else(|| {
            debug!(radical = %radical, "Radical not found in dictionary");
            OrigaError::KradfileError {
                reason: format!("Radical {} not found in radkfile", radical),
            }
        })
    }

    pub fn known_radicals(&self) -> &[char] {
        &self.known_radicals
    }

    pub fn radical_list(&self) -> Vec<RadicalInfo> {
        self.radical_map.values().cloned().collect()
    }
}

#[derive(Serialize, Deserialize)]
struct RadicalStoredType {
    #[serde(rename = "strokeCount")]
    stroke_count: u32,
    kanji: Vec<String>,
    name: String,
    description: String,
    name_en: Option<String>,
    description_en: Option<String>,
    jlpt: String,
}

#[derive(Serialize, Deserialize)]
struct RadkfilStoredType {
    radicals: HashMap<String, RadicalStoredType>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // RadicalDatabase::from_json tests
    #[test]
    fn radical_database_from_json_valid() {
        let json = r#"{
            "radicals": {
                "日": {
                    "strokeCount": 4,
                    "kanji": ["明", "睛"],
                    "name": "sun, day",
                    "description": "Sun or day radical",
                    "jlpt": "N5"
                }
            }
        }"#;

        let db = RadicalDatabase::from_json(json).unwrap();
        let info = db.get_radical_info(&'日').unwrap();
        assert_eq!(info.stroke_count(), 4);
        assert_eq!(info.name(&NativeLanguage::Russian), "sun, day");
        assert!(info.kanji().contains(&'明'));
    }

    #[test]
    fn radical_database_from_json_multibyte_key() {
        let json = r#"{"radicals": {"山": {"strokeCount": 3, "kanji": [], "name": "mountain", "description": "Mountain", "jlpt": "N5"}}}"#;
        let db = RadicalDatabase::from_json(json).unwrap();
        assert!(db.get_radical_info(&'山').is_ok());
    }

    #[test]
    fn radical_database_from_json_empty() {
        let json = r#"{"radicals": {}}"#;
        let db = RadicalDatabase::from_json(json).unwrap();
        assert!(db.radical_list().is_empty());
    }

    #[test]
    fn radical_database_from_json_invalid_json() {
        let json = "not valid json";
        let result = RadicalDatabase::from_json(json);
        assert!(result.is_err());
    }

    // RadicalDatabase::radical_list tests
    #[test]
    fn radical_database_radical_list() {
        let json = r#"{
            "radicals": {
                "一": {"strokeCount": 1, "kanji": [], "name": "one", "description": "One", "jlpt": "N5"},
                "二": {"strokeCount": 2, "kanji": [], "name": "two", "description": "Two", "jlpt": "N5"}
            }
        }"#;

        let db = RadicalDatabase::from_json(json).unwrap();
        let list = db.radical_list();

        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|r| r.radical() == '一'));
        assert!(list.iter().any(|r| r.radical() == '二'));
    }

    // RadicalDatabase::known_radicals tests
    #[test]
    fn radical_database_known_radicals() {
        let json = r#"{
            "radicals": {
                "日": {"strokeCount": 4, "kanji": [], "name": "sun", "description": "Sun", "jlpt": "N5"},
                "月": {"strokeCount": 4, "kanji": [], "name": "moon", "description": "Moon", "jlpt": "N5"}
            }
        }"#;

        let db = RadicalDatabase::from_json(json).unwrap();
        let radicals = db.known_radicals();

        assert_eq!(radicals.len(), 2);
        assert!(radicals.contains(&'日'));
        assert!(radicals.contains(&'月'));
    }

    // RadicalDatabase::get_radical_info tests
    #[test]
    fn radical_database_get_radical_info_not_found() {
        let json = r#"{"radicals": {"日": {"strokeCount": 4, "kanji": [], "name": "sun", "description": "Sun", "jlpt": "N5"}}}"#;
        let db = RadicalDatabase::from_json(json).unwrap();

        assert!(db.get_radical_info(&'不').is_err());
    }

    // RadicalDatabase::get_radical_info all fields tests
    #[test]
    fn radical_database_get_radical_info_all_fields() {
        let json = r#"{
            "radicals": {
                "木": {
                    "strokeCount": 4,
                    "kanji": ["林", "森", "村"],
                    "name": "tree, wood",
                    "description": "Tree radical",
                    "jlpt": "N4"
                }
            }
        }"#;

        let db = RadicalDatabase::from_json(json).unwrap();
        let info = db.get_radical_info(&'木').unwrap();

        assert_eq!(info.radical(), '木');
        assert_eq!(info.stroke_count(), 4);
        assert_eq!(info.name(&NativeLanguage::Russian), "tree, wood");
        assert_eq!(info.description(&NativeLanguage::Russian), "Tree radical");
        assert_eq!(info.jlpt(), &JapaneseLevel::N4);
        assert_eq!(info.kanji().len(), 3);
        assert!(info.kanji().contains(&'林'));
        assert!(info.kanji().contains(&'森'));
        assert!(info.kanji().contains(&'村'));
    }

    // Tests for public API functions
    #[test]
    fn init_radicals_valid() {
        // Test init_radicals with RadicalDatabase instead of global static to avoid OnceLock issues
        let data = RadicalData {
            radicals_json: r#"{
                "radicals": {
                    "日": {"strokeCount": 4, "kanji": [], "name": "sun", "description": "Sun", "jlpt": "N5"}
                }
            }"#.to_string(),
        };

        // Create a database directly to test the JSON parsing without setting global state
        let db = RadicalDatabase::from_json(&data.radicals_json).unwrap();
        assert_eq!(db.radical_list().len(), 1);
        assert_eq!(db.radical_list()[0].radical(), '日');
    }

    #[test]
    fn init_radicals_invalid_json() {
        let data = RadicalData {
            radicals_json: "not valid json".to_string(),
        };

        // Test that invalid JSON is properly detected
        let result = RadicalDatabase::from_json(&data.radicals_json);
        assert!(result.is_err());
    }

    #[test]
    fn get_radical_info_after_init() {
        // Test using RadicalDatabase directly to avoid OnceLock issues
        let json = r#"{
            "radicals": {
                "水": {"strokeCount": 4, "kanji": ["海", "川"], "name": "water", "description": "Water", "jlpt": "N5"}
            }
        }"#;

        let db = RadicalDatabase::from_json(json).unwrap();
        let info = db.get_radical_info(&'水').unwrap();

        assert_eq!(info.radical(), '水');
        assert_eq!(info.stroke_count(), 4);
        assert_eq!(info.name(&NativeLanguage::Russian), "water");
        assert!(info.kanji().contains(&'海'));
    }

    #[test]
    fn get_radical_info_not_loaded() {
        // Test with a fresh RadicalDatabase instead of global static to avoid OnceLock issues
        let json = r#"{"radicals": {"日": {"strokeCount": 4, "kanji": [], "name": "sun", "description": "Sun", "jlpt": "N5"}}}"#;
        let db = RadicalDatabase::from_json(json).unwrap();
        let result = db.get_radical_info(&'中');

        assert!(result.is_err());
        if let Err(OrigaError::KradfileError { reason }) = result {
            assert!(reason.contains("中"));
        }
    }

    #[test]
    fn get_radical_list_not_loaded_returns_empty() {
        // Test that get_radical_list returns empty when dictionary is not loaded
        // Since we can't reset the global dictionary, we test the behavior with RadicalDatabase instead
        let json = r#"{"radicals": {}}"#;
        let db = RadicalDatabase::from_json(json).unwrap();
        assert!(db.radical_list().is_empty());
    }

    #[test]
    fn get_radical_list() {
        // Test with a fresh RadicalDatabase instead of global static to avoid OnceLock issues
        let json = r#"{
            "radicals": {
                "日": {"strokeCount": 4, "kanji": [], "name": "sun", "description": "Sun", "jlpt": "N5"},
                "月": {"strokeCount": 4, "kanji": [], "name": "moon", "description": "Moon", "jlpt": "N5"},
                "金": {"strokeCount": 8, "kanji": [], "name": "gold", "description": "Gold", "jlpt": "N4"}
            }
        }"#;

        let db = RadicalDatabase::from_json(json).unwrap();
        let list = db.radical_list();

        assert_eq!(list.len(), 3);
        assert!(list.iter().any(|r| r.radical() == '日'));
        assert!(list.iter().any(|r| r.radical() == '月'));
        assert!(list.iter().any(|r| r.radical() == '金'));
    }

    // English name/description projection (language-aware accessors)
    #[test]
    fn radical_info_name_returns_english_for_english_language() {
        let json = r#"{
            "radicals": {
                "日": {
                    "strokeCount": 4,
                    "kanji": [],
                    "name": "Солнце",
                    "name_en": "Sun",
                    "description": "Радикал «Солнце»",
                    "description_en": "The sun radical",
                    "jlpt": "N5"
                }
            }
        }"#;

        let db = RadicalDatabase::from_json(json).unwrap();
        let info = db.get_radical_info(&'日').unwrap();

        assert_eq!(info.name(&NativeLanguage::English), "Sun");
        assert_eq!(
            info.description(&NativeLanguage::English),
            "The sun radical"
        );
    }

    #[test]
    fn radical_info_name_returns_russian_for_russian_language() {
        let json = r#"{
            "radicals": {
                "日": {
                    "strokeCount": 4,
                    "kanji": [],
                    "name": "Солнце",
                    "name_en": "Sun",
                    "description": "Радикал «Солнце»",
                    "description_en": "The sun radical",
                    "jlpt": "N5"
                }
            }
        }"#;

        let db = RadicalDatabase::from_json(json).unwrap();
        let info = db.get_radical_info(&'日').unwrap();

        assert_eq!(info.name(&NativeLanguage::Russian), "Солнце");
        assert_eq!(
            info.description(&NativeLanguage::Russian),
            "Радикал «Солнце»"
        );
    }

    #[test]
    fn radical_info_without_english_fields_returns_empty_for_english() {
        // Legacy radicals.json (pre-EN data) must keep deserializing; the
        // English projection degrades to an empty string so the UI can hide
        // untranslated copy instead of showing Russian to an English user.
        let json = r#"{
            "radicals": {
                "日": {"strokeCount": 4, "kanji": [], "name": "Солнце", "description": "Радикал «Солнце»", "jlpt": "N5"}
            }
        }"#;

        let db = RadicalDatabase::from_json(json).unwrap();
        let info = db.get_radical_info(&'日').unwrap();

        assert_eq!(info.name(&NativeLanguage::English), "");
        assert_eq!(info.description(&NativeLanguage::English), "");
        assert_eq!(info.name(&NativeLanguage::Russian), "Солнце");
    }

    // Parametrized tests with rstest
    #[rstest]
    #[case('日', 4, "sun")]
    #[case('月', 4, "moon")]
    #[case('山', 3, "mountain")]
    fn radical_database_stroke_count_and_name(
        #[case] radical_char: char,
        #[case] expected_stroke_count: u32,
        #[case] expected_name: &str,
    ) {
        let json = format!(
            r#"{{
                "radicals": {{
                    "{}": {{"strokeCount": {}, "kanji": [], "name": "{}", "description": "Test", "jlpt": "N5"}}
                }}
            }}"#,
            radical_char, expected_stroke_count, expected_name
        );

        let db = RadicalDatabase::from_json(&json).unwrap();
        let info = db.get_radical_info(&radical_char).unwrap();

        assert_eq!(info.stroke_count(), expected_stroke_count);
        assert_eq!(info.name(&NativeLanguage::Russian), expected_name);
    }

    #[rstest]
    #[case("N5", JapaneseLevel::N5)]
    #[case("N4", JapaneseLevel::N4)]
    #[case("N3", JapaneseLevel::N3)]
    #[case("N2", JapaneseLevel::N2)]
    #[case("N1", JapaneseLevel::N1)]
    fn radical_database_jlpt_parsing(
        #[case] jlpt_str: &str,
        #[case] expected_level: JapaneseLevel,
    ) {
        let json = format!(
            r#"{{
                "radicals": {{
                    "一": {{"strokeCount": 1, "kanji": [], "name": "one", "description": "One", "jlpt": "{}"}}
                }}
            }}"#,
            jlpt_str
        );

        let db = RadicalDatabase::from_json(&json).unwrap();
        let info = db.get_radical_info(&'一').unwrap();

        assert_eq!(info.jlpt(), &expected_level);
    }

    #[rstest]
    #[case("invalid", JapaneseLevel::N1)] // defaults to N1
    fn radical_database_jlpt_invalid_defaults(
        #[case] jlpt_str: &str,
        #[case] expected_level: JapaneseLevel,
    ) {
        let json = format!(
            r#"{{
                "radicals": {{
                    "一": {{"strokeCount": 1, "kanji": [], "name": "one", "description": "One", "jlpt": "{}"}}
                }}
            }}"#,
            jlpt_str
        );

        let db = RadicalDatabase::from_json(&json).unwrap();
        let info = db.get_radical_info(&'一').unwrap();

        assert_eq!(info.jlpt(), &expected_level);
    }

    // Test kanji parsing from strings
    #[test]
    fn radical_database_kanji_from_string() {
        let json = r#"{
            "radicals": {
                "火": {
                    "strokeCount": 4,
                    "kanji": ["燃", "焼", "灯"],
                    "name": "fire",
                    "description": "Fire radical",
                    "jlpt": "N5"
                }
            }
        }"#;

        let db = RadicalDatabase::from_json(json).unwrap();
        let info = db.get_radical_info(&'火').unwrap();

        assert_eq!(info.kanji().len(), 3);
        assert!(info.kanji().contains(&'燃'));
        assert!(info.kanji().contains(&'焼'));
        assert!(info.kanji().contains(&'灯'));
    }

    // Test multiple radicals in kanji list (edge case)
    #[test]
    fn radical_database_multichar_kanji() {
        let json = r#"{
            "radicals": {
                "土": {
                    "strokeCount": 3,
                    "kanji": ["埴"],
                    "name": "earth",
                    "description": "Earth radical",
                    "jlpt": "N5"
                }
            }
        }"#;

        let db = RadicalDatabase::from_json(json).unwrap();
        let info = db.get_radical_info(&'土').unwrap();

        assert_eq!(info.kanji().len(), 1);
        assert!(info.kanji().contains(&'埴'));
    }

    // RadicalDatabase equality and cloning tests
    #[test]
    fn radical_info_clone() {
        let json = r#"{
            "radicals": {
                "日": {"strokeCount": 4, "kanji": [], "name": "sun", "description": "Sun", "jlpt": "N5"}
            }
        }"#;

        let db = RadicalDatabase::from_json(json).unwrap();
        let list = db.radical_list();
        let cloned = list[0].clone();

        assert_eq!(cloned.radical(), '日');
        assert_eq!(cloned.stroke_count(), 4);
    }

    // Test error messages
    #[test]
    fn radical_database_error_message() {
        let json = r#"{"radicals": {"日": {"strokeCount": 4, "kanji": [], "name": "sun", "description": "Sun", "jlpt": "N5"}}}"#;
        let db = RadicalDatabase::from_json(json).unwrap();

        let result = db.get_radical_info(&'不');
        assert!(result.is_err());

        if let Err(OrigaError::KradfileError { reason }) = result {
            assert!(reason.contains("不"));
        }
    }
}
