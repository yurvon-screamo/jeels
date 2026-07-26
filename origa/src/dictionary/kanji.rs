use std::{collections::HashMap, sync::OnceLock};

use serde::{Deserialize, Serialize, de};
use tracing::debug;

use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError};

fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct StringOrVec;

    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or an array of strings")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            Ok(vec![value.to_string()])
        }

        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut values = Vec::new();
            while let Some(value) = seq.next_element::<String>()? {
                values.push(value);
            }
            Ok(values)
        }
    }

    deserializer.deserialize_any(StringOrVec)
}

pub static KANJI_DICTIONARY: OnceLock<KanjiDatabase> = OnceLock::new();

#[derive(Clone, Serialize, Deserialize)]
pub struct KanjiData {
    pub kanji_json: String,
}

pub fn init_kanji(data: KanjiData) -> Result<(), OrigaError> {
    if is_kanji_loaded() {
        return Ok(());
    }

    let db = KanjiDatabase::from_json(&data.kanji_json)?;
    KANJI_DICTIONARY.set(db).ok();
    Ok(())
}

pub fn is_kanji_loaded() -> bool {
    KANJI_DICTIONARY.get().is_some()
}

pub fn get_kanji_info(kanji: &str) -> Result<&'static KanjiInfo, OrigaError> {
    let db = KANJI_DICTIONARY.get();
    if db.is_none() {
        debug!(kanji = %kanji, "Kanji dictionary not loaded");
    }
    db.ok_or(OrigaError::KradfileError {
        reason: "Kanji dictionary not loaded".to_string(),
    })?
    .get_kanji_info(kanji)
}

pub fn get_kanji_list(level: &JapaneseLevel) -> Vec<&'static KanjiInfo> {
    KANJI_DICTIONARY
        .get()
        .map(|db| db.get_kanji_list(level))
        .unwrap_or_default()
}

pub fn get_all_kanji() -> Vec<char> {
    KANJI_DICTIONARY
        .get()
        .map(|db| db.all_kanji())
        .unwrap_or_default()
}

#[derive(Clone)]
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
    description_ru: Vec<String>,
    description_en: Vec<String>,
    radicals: Vec<char>,
    popular_words: Vec<String>,
    on_readings: Vec<String>,
    kun_readings: Vec<String>,
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

    pub fn description(&self, lang: &NativeLanguage) -> String {
        let descs = match lang {
            NativeLanguage::Russian => &self.description_ru,
            NativeLanguage::English => {
                if self.description_en.is_empty() {
                    &self.description_ru
                } else {
                    &self.description_en
                }
            },
        };
        descs.join(", ")
    }

    pub fn descriptions(&self, lang: &NativeLanguage) -> &[String] {
        match lang {
            NativeLanguage::Russian => &self.description_ru,
            NativeLanguage::English => {
                if self.description_en.is_empty() {
                    &self.description_ru
                } else {
                    &self.description_en
                }
            },
        }
    }

    pub fn radicals_chars(&self) -> &[char] {
        &self.radicals
    }

    pub fn popular_words(&self) -> &[String] {
        &self.popular_words
    }

    pub fn on_readings(&self) -> &[String] {
        &self.on_readings
    }

    pub fn kun_readings(&self) -> &[String] {
        &self.kun_readings
    }

    pub fn popular_words_with_translations(
        &self,
        native_language: &NativeLanguage,
    ) -> Vec<PopularWord> {
        use crate::dictionary::vocabulary::VOCABULARY_DICTIONARY;

        let fallback = match native_language {
            NativeLanguage::Russian => "Перевод не найден",
            NativeLanguage::English => "Translation not found",
        };

        self.popular_words
            .iter()
            .map(|word| {
                let translation = VOCABULARY_DICTIONARY
                    .get()
                    .and_then(|db| db.get_translation(word, native_language))
                    .unwrap_or_else(|| fallback.to_string());
                PopularWord::new(word.clone(), translation)
            })
            .collect()
    }
}

impl KanjiDatabase {
    fn from_json(json: &str) -> Result<Self, OrigaError> {
        let kanji_db: KanjiDatabaseStoredType =
            serde_json::from_str(json).map_err(|e| OrigaError::KradfileError {
                reason: format!("Failed to parse kanji.json: {}", e),
            })?;

        let kanji_map = kanji_db
            .kanji
            .into_iter()
            .map(|k| {
                let jlpt = JapaneseLevel::from_str_or_default(&k.jlpt);
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
                        description_ru: k.description_ru,
                        description_en: k.description_en,
                        radicals,
                        popular_words: k.popular_words,
                        on_readings: k.on_readings,
                        kun_readings: k.kun_readings,
                    },
                )
            })
            .collect::<HashMap<String, KanjiInfo>>();

        Ok(Self { kanji_map })
    }

    pub fn get_kanji_info(&self, kanji: &str) -> Result<&KanjiInfo, OrigaError> {
        let info = self.kanji_map.get(kanji);
        if info.is_none() {
            debug!(kanji = %kanji, "Kanji not found in dictionary");
        }
        info.ok_or(OrigaError::KradfileError {
            reason: format!("Kanji {} not found in kanji database", kanji),
        })
    }

    pub fn get_kanji_list(&self, level: &JapaneseLevel) -> Vec<&KanjiInfo> {
        self.kanji_map
            .values()
            .filter(|x| x.jlpt() == level)
            .collect()
    }

    pub fn all_kanji(&self) -> Vec<char> {
        self.kanji_map.values().map(|info| info.kanji).collect()
    }
}

#[derive(Serialize, Deserialize)]
struct KanjiStoredType {
    kanji: String,
    jlpt: String,
    used_in: u32,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    description_ru: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    description_en: Vec<String>,
    radicals: Vec<String>,
    popular_words: Vec<String>,
    #[serde(default)]
    on_readings: Vec<String>,
    #[serde(default)]
    kun_readings: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct KanjiDatabaseStoredType {
    kanji: Vec<KanjiStoredType>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::path::Path;

    fn create_valid_kanji_json() -> String {
        r#"{
            "kanji": [
                {
                    "kanji": "日",
                    "jlpt": "N5",
                    "used_in": 100,
                    "description_ru": ["день", "солнце"],
                    "description_en": ["day", "sun"],
                    "radicals": ["一", "口"],
                    "popular_words": ["日本", "日曜日"],
                    "on_readings": ["NICHI", "JITSU"],
                    "kun_readings": ["ひ", "-び"]
                },
                {
                    "kanji": "本",
                    "jlpt": "N5",
                    "used_in": 80,
                    "description_ru": ["книга", "основа"],
                    "description_en": ["book", "origin"],
                    "radicals": ["木", "一"],
                    "popular_words": ["本", "日本"],
                    "on_readings": ["HON"],
                    "kun_readings": ["もと"]
                }
            ]
        }"#
        .to_string()
    }

    fn create_invalid_json() -> String {
        "{ invalid json }".to_string()
    }

    fn load_real_kanji_json() -> String {
        let kanji_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../cdn/dictionary/kanji.json");

        if kanji_path.exists() {
            std::fs::read_to_string(&kanji_path).expect("Failed to read kanji.json")
        } else {
            create_valid_kanji_json()
        }
    }

    fn ensure_test_dictionary_loaded() {
        if !is_kanji_loaded() {
            let kanji_path =
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../cdn/dictionary/kanji.json");

            if kanji_path.exists() {
                let kanji_json =
                    std::fs::read_to_string(&kanji_path).expect("Failed to read kanji.json");
                let data = KanjiData { kanji_json };
                init_kanji(data).expect("Failed to initialize kanji dictionary");
            } else {
                let data = KanjiData {
                    kanji_json: create_valid_kanji_json(),
                };
                init_kanji(data).expect("Failed to initialize test dictionary");
            }
        }
    }

    #[test]
    fn init_kanji_valid_json_success() {
        ensure_test_dictionary_loaded();
        assert!(is_kanji_loaded());
    }

    #[test]
    fn init_kanji_invalid_json_error() {
        let result = KanjiDatabase::from_json(&create_invalid_json());
        assert!(result.is_err());
        assert!(matches!(result, Err(OrigaError::KradfileError { .. })));
    }

    #[rstest]
    #[case("日", '日', JapaneseLevel::N5)]
    #[case("人", '人', JapaneseLevel::N5)]
    #[case("本", '本', JapaneseLevel::N5)]
    fn get_kanji_info_found(
        #[case] kanji_str: &str,
        #[case] expected_char: char,
        #[case] expected_level: JapaneseLevel,
    ) {
        ensure_test_dictionary_loaded();
        let result = get_kanji_info(kanji_str);
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.kanji(), expected_char);
        assert_eq!(info.jlpt(), &expected_level);
    }

    #[test]
    fn get_kanji_info_not_found_error() {
        ensure_test_dictionary_loaded();
        let result = get_kanji_info("𠮷");
        assert!(result.is_err());
        assert!(matches!(result, Err(OrigaError::KradfileError { .. })));
    }

    #[rstest]
    #[case(JapaneseLevel::N5)]
    #[case(JapaneseLevel::N4)]
    #[case(JapaneseLevel::N3)]
    #[case(JapaneseLevel::N2)]
    #[case(JapaneseLevel::N1)]
    fn get_kanji_list_all_levels(#[case] level: JapaneseLevel) {
        ensure_test_dictionary_loaded();
        let result = get_kanji_list(&level);
        assert!(!result.is_empty(), "Level {:?} should have kanji", level);
    }

    #[test]
    fn kanji_info_accessors() {
        ensure_test_dictionary_loaded();
        let info = get_kanji_info("日").unwrap();

        assert_eq!(info.kanji(), '日');
        assert_eq!(info.jlpt(), &JapaneseLevel::N5);
    }

    #[test]
    fn popular_word_new() {
        let word = PopularWord::new("日本".to_string(), "Japan".to_string());
        assert_eq!(word.word(), "日本");
        assert_eq!(word.translation(), "Japan");
    }

    #[test]
    fn popular_words_with_translations() {
        ensure_test_dictionary_loaded();
        let info = get_kanji_info("日").unwrap();
        let words = info.popular_words_with_translations(&NativeLanguage::Russian);
        assert!(!words.is_empty());
        assert!(!words.is_empty(), "Should have at least 1 popular word");
    }

    #[test]
    fn integration_real_dictionary() {
        ensure_test_dictionary_loaded();
        let n5_list = get_kanji_list(&JapaneseLevel::N5);
        assert!(!n5_list.is_empty());

        for kanji_str in ["日", "人", "一", "大", "年"] {
            let result = get_kanji_info(kanji_str);
            assert!(result.is_ok(), "Should find kanji: {}", kanji_str);
            let info = result.unwrap();
            assert!(!info.description(&NativeLanguage::Russian).is_empty());
            assert!(!info.popular_words().is_empty());
        }
    }

    #[test]
    fn kanji_database_from_json_with_empty_kanji_array() {
        let json = r#"{"kanji": []}"#;
        let db = KanjiDatabase::from_json(json).unwrap();
        assert!(db.get_kanji_list(&JapaneseLevel::N5).is_empty());
    }

    #[test]
    fn kanji_database_from_json_missing_optional_fields() {
        let json = r#"{
            "kanji": [{
                "kanji": "測",
                "jlpt": "N2",
                "used_in": 100,
                "description_ru": ["измерение"],
                "description_en": ["measurement"],
                "radicals": [],
                "popular_words": []
            }]
        }"#;
        let db = KanjiDatabase::from_json(json).unwrap();
        let info = db.get_kanji_info("測").unwrap();
        assert_eq!(info.radicals_chars().len(), 0);
        assert!(info.popular_words().is_empty());
    }

    #[test]
    fn kanji_database_from_json_multibyte_kanji() {
        let json = r#"{
            "kanji": [{
                "kanji": "一二三四五六七八九十",
                "jlpt": "N5",
                "used_in": 100,
                "description_ru": ["числа"],
                "description_en": ["numbers"],
                "radicals": [],
                "popular_words": [],
                "on_readings": [],
                "kun_readings": []
            }]
        }"#;
        let db = KanjiDatabase::from_json(json).unwrap();
        let info = db.get_kanji_info("一").unwrap();
        assert_eq!(info.kanji(), '一');
    }

    #[test]
    fn get_kanji_info_empty_string() {
        let json = load_real_kanji_json();
        let db = KanjiDatabase::from_json(&json).unwrap();
        let result = db.get_kanji_info("");
        assert!(result.is_err());
    }

    #[test]
    fn kanji_info_radicals_expanded_from_multichar_strings() {
        let json = r#"{
            "kanji": [{
                "kanji": "木",
                "jlpt": "N5",
                "used_in": 100,
                "description_ru": ["дерево"],
                "description_en": ["tree"],
                "radicals": ["木", "一"],
                "popular_words": [],
                "on_readings": ["ボク", "モク"],
                "kun_readings": ["き", "こ"]
            }]
        }"#;
        let db = KanjiDatabase::from_json(json).unwrap();
        let info = db.get_kanji_info("木").unwrap();
        assert!(info.radicals_chars().contains(&'木'));
        assert!(info.radicals_chars().contains(&'一'));
    }

    #[test]
    fn kanji_info_description_is_russian() {
        let json = load_real_kanji_json();
        let db = KanjiDatabase::from_json(&json).unwrap();
        let info = db.get_kanji_info("日").unwrap();
        assert!(!info.description(&NativeLanguage::Russian).is_empty());
    }

    #[test]
    fn popular_words_with_translations_fallback() {
        let json = load_real_kanji_json();
        let db = KanjiDatabase::from_json(&json).unwrap();
        let info = db.get_kanji_info("日").unwrap();
        let words = info.popular_words_with_translations(&NativeLanguage::English);
        assert!(!words.is_empty());
    }

    #[rstest]
    #[case("人", 2000)]
    #[case("一", 0)]
    #[case("日", 1000)]
    fn kanji_used_in_frequency(#[case] kanji: &str, #[case] min_used_in: u32) {
        let json = load_real_kanji_json();
        let db = KanjiDatabase::from_json(&json).unwrap();
        let info = db.get_kanji_info(kanji).unwrap();
        assert!(
            info.used_in() >= min_used_in,
            "Kanji {} should have high usage frequency",
            kanji
        );
    }

    #[test]
    fn kanji_database_clone() {
        let json = load_real_kanji_json();
        let db1 = KanjiDatabase::from_json(&json).unwrap();
        let db2 = db1.clone();
        let info1 = db1.get_kanji_info("日").unwrap();
        let info2 = db2.get_kanji_info("日").unwrap();
        assert_eq!(info1.kanji(), info2.kanji());
    }

    #[test]
    fn kanji_database_from_json_parsing_errors() {
        let invalid_json_variants = vec![
            "not json at all",
            r#"{"kanji": invalid}"#,
            r#"{"kanji": [{ "kanji": 123 }]}"#,
            "",
        ];

        for json in invalid_json_variants {
            let result = KanjiDatabase::from_json(json);
            assert!(result.is_err(), "Should fail to parse: {:?}", json);
        }
    }

    #[test]
    fn get_kanji_info_debug_logging() {
        let json = r#"{"kanji": []}"#;
        let db = KanjiDatabase::from_json(json).unwrap();

        let result = db.get_kanji_info("不存在");
        assert!(result.is_err());
    }

    #[test]
    fn get_kanji_list_filters_by_level() {
        let json = load_real_kanji_json();
        let db = KanjiDatabase::from_json(&json).unwrap();

        let n5_count = db.get_kanji_list(&JapaneseLevel::N5).len();
        let n1_count = db.get_kanji_list(&JapaneseLevel::N1).len();

        assert!(n5_count > 0, "N5 list should not be empty");
        assert_ne!(n5_count, n1_count, "N5 and N1 should have different counts");
    }

    #[test]
    fn kanji_info_all_readings_accessors() {
        let json = r#"{
            "kanji": [{
                "kanji": "会",
                "jlpt": "N4",
                "used_in": 500,
                "description_ru": ["встреча"],
                "description_en": ["meeting"],
                "radicals": ["人", "云"],
                "popular_words": ["会社"],
                "on_readings": ["カイ", "エ"],
                "kun_readings": ["あ.う"]
            }]
        }"#;
        let db = KanjiDatabase::from_json(json).unwrap();
        let info = db.get_kanji_info("会").unwrap();

        assert!(!info.on_readings().is_empty());
        assert!(!info.kun_readings().is_empty());

        assert!(info.on_readings().iter().any(|r| r == "カイ"));
        assert!(info.kun_readings().iter().any(|r| r == "あ.う"));
    }

    #[test]
    fn module_public_functions_isolation() {
        let json = load_real_kanji_json();
        let data = KanjiData { kanji_json: json };

        let result = init_kanji(data);
        assert!(result.is_ok());

        assert!(is_kanji_loaded());
    }

    #[test]
    fn module_get_kanji_info_public_function() {
        let json = load_real_kanji_json();
        let data = KanjiData { kanji_json: json };
        let _ = init_kanji(data);

        let result = get_kanji_info("日");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().kanji(), '日');
    }

    #[test]
    fn module_get_kanji_list_public_function() {
        let json = load_real_kanji_json();
        let data = KanjiData { kanji_json: json };
        let _ = init_kanji(data);

        let result = get_kanji_list(&JapaneseLevel::N5);
        assert!(!result.is_empty());
    }

    #[test]
    fn get_kanji_info_exact_match_required() {
        let json = r#"{
            "kanji": [{
                "kanji": "日",
                "jlpt": "N5",
                "used_in": 100,
                "description_ru": ["день"],
                "description_en": ["day"],
                "radicals": ["日"],
                "popular_words": [],
                "on_readings": ["ニチ"],
                "kun_readings": ["ひ"]
            }]
        }"#;
        let db = KanjiDatabase::from_json(json).unwrap();

        let single_char = db.get_kanji_info("日");
        assert!(single_char.is_ok());

        let multi_char = db.get_kanji_info("日本");
        assert!(multi_char.is_err());
    }

    #[test]
    fn popular_word_clone() {
        let word = PopularWord::new("test".to_string(), "тест".to_string());
        let cloned = word.clone();
        assert_eq!(word.word(), cloned.word());
        assert_eq!(word.translation(), cloned.translation());
    }

    #[test]
    fn backward_compat_string_description_deserializes_to_vec() {
        let json = r#"{
            "kanji": [{
                "kanji": "日",
                "jlpt": "N5",
                "used_in": 100,
                "description_ru": "день",
                "description_en": "day",
                "radicals": ["日"],
                "popular_words": [],
                "on_readings": ["ニチ"],
                "kun_readings": ["ひ"]
            }]
        }"#;
        let db = KanjiDatabase::from_json(json).unwrap();
        let info = db.get_kanji_info("日").unwrap();
        assert_eq!(
            info.description(&NativeLanguage::Russian),
            "день".to_string()
        );
        assert_eq!(
            info.description(&NativeLanguage::English),
            "day".to_string()
        );
        assert_eq!(
            info.descriptions(&NativeLanguage::Russian),
            &["день".to_string()]
        );
    }

    #[test]
    fn description_joins_multiple_values() {
        let json = r#"{
            "kanji": [{
                "kanji": "可",
                "jlpt": "N4",
                "used_in": 100,
                "description_ru": ["хороший", "возможный"],
                "description_en": ["good", "possible"],
                "radicals": [],
                "popular_words": [],
                "on_readings": [],
                "kun_readings": []
            }]
        }"#;
        let db = KanjiDatabase::from_json(json).unwrap();
        let info = db.get_kanji_info("可").unwrap();
        assert_eq!(
            info.description(&NativeLanguage::Russian),
            "хороший, возможный"
        );
        assert_eq!(info.description(&NativeLanguage::English), "good, possible");
        assert_eq!(info.descriptions(&NativeLanguage::Russian).len(), 2);
    }
}
