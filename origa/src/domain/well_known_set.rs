use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::value_objects::{JapaneseLevel, NativeLanguage};

pub type SetType = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeMeta {
    pub id: String,
    pub label_ru: String,
    pub label_en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypesMeta {
    pub types: Vec<TypeMeta>,
}

impl TypesMeta {
    pub fn get_label<'a>(&'a self, set_type: &'a SetType, lang: &NativeLanguage) -> &'a str {
        self.types
            .iter()
            .find(|t| &t.id == set_type)
            .map(|t| match lang {
                NativeLanguage::Russian => t.label_ru.as_str(),
                NativeLanguage::English => t.label_en.as_str(),
            })
            .unwrap_or(set_type)
    }
}

static TYPES_META_CACHE: OnceLock<TypesMeta> = OnceLock::new();

pub fn set_types_meta(meta: TypesMeta) {
    let _ = TYPES_META_CACHE.set(meta);
}

pub fn get_types_meta() -> Option<&'static TypesMeta> {
    TYPES_META_CACHE.get()
}

pub fn id_to_set_type(id: &str) -> SetType {
    if let Some(meta) = get_types_meta() {
        for t in &meta.types {
            if id.contains(&t.id) || id.starts_with(&t.id.to_lowercase()) {
                return t.id.clone();
            }
        }
    }

    if id.starts_with("jlpt_") {
        "Jlpt".to_string()
    } else if id.starts_with("migii_") {
        "Migii".to_string()
    } else if id.starts_with("spy_family") {
        "SpyFamily".to_string()
    } else if id.contains("_en_") {
        "DuolingoEn".to_string()
    } else if id.contains("_ru_") || id.starts_with("duolingo_") {
        "DuolingoRu".to_string()
    } else if id.starts_with("minna_n5_")
        || id.starts_with("minna_n4_")
        || id.starts_with("minna_n3_")
        || id.starts_with("minna_n2_")
    {
        "MinnaNoNihongo".to_string()
    } else if id.starts_with("irodori_") {
        "Irodori".to_string()
    } else {
        id.split('_').next().unwrap_or("Unknown").to_string()
    }
}

fn resolve_jlpt_path(id: &str) -> Option<String> {
    let level = id.strip_prefix("jlpt_")?;
    Some(format!("well_known_set/jlpt_{}.json", level))
}

fn resolve_migii_path(id: &str) -> Option<String> {
    let rest = id.strip_prefix("migii_")?;
    let level = rest.split('_').next().unwrap_or("");
    Some(format!("well_known_set/migii/{}/{}.json", level, id))
}

fn resolve_spy_family_path(id: &str) -> Option<String> {
    if id.strip_prefix("spy_family_").is_some() {
        return Some(format!("well_known_set/spy_family/{}.json", id));
    }
    None
}

fn resolve_duolingo_path(id: &str) -> Option<String> {
    let rest = id.strip_prefix("duolingo_")?;
    let level = rest.split('_').next().unwrap_or("");
    let parts: Vec<&str> = rest.split('_').collect();
    let remaining = parts.iter().skip(1).copied().collect::<Vec<_>>().join("_");
    let filename = if remaining.starts_with("duolingo_") {
        remaining
    } else if parts.len() >= 3 {
        parts[2..].join("_")
    } else {
        rest.to_string()
    };
    Some(format!(
        "well_known_set/duolingo/{}/{}.json",
        level, filename
    ))
}

fn resolve_minna_path(id: &str) -> Option<String> {
    if id.strip_prefix("minna_n5_").is_some() {
        return Some(format!("well_known_set/minna_n5/{}.json", id));
    }
    if id.strip_prefix("minna_n4_").is_some() {
        return Some(format!("well_known_set/minna_n4/{}.json", id));
    }
    if id.strip_prefix("minna_n2_").is_some() {
        return Some(format!("well_known_set/minna_n2/{}.json", id));
    }
    if id.strip_prefix("minna_n3_").is_some() {
        return Some(format!("well_known_set/minna_n3/{}.json", id));
    }
    None
}

fn resolve_irodori_path(id: &str) -> Option<String> {
    if id.strip_prefix("irodori_nyuumon_").is_some() {
        return Some(format!("well_known_set/irodori_nyuumon/{}.json", id));
    }
    if id.strip_prefix("irodori_shokyuu1_").is_some() {
        return Some(format!("well_known_set/irodori_shokyuu1/{}.json", id));
    }
    if id.strip_prefix("irodori_shokyuu2_").is_some() {
        return Some(format!("well_known_set/irodori_shokyuu2/{}.json", id));
    }
    None
}

pub fn resolve_set_path(id: &str) -> String {
    if id.contains("..") || id.contains('/') {
        return format!("well_known_set/{}.json", id);
    }

    resolve_jlpt_path(id)
        .or_else(|| resolve_migii_path(id))
        .or_else(|| resolve_spy_family_path(id))
        .or_else(|| resolve_duolingo_path(id))
        .or_else(|| resolve_minna_path(id))
        .or_else(|| resolve_irodori_path(id))
        .unwrap_or_else(|| format!("well_known_set/{}.json", id))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellKnownSetMeta {
    pub id: String,
    #[serde(rename = "set_type")]
    pub set_type: SetType,
    pub level: JapaneseLevel,
    pub title_ru: String,
    pub title_en: String,
    pub desc_ru: String,
    pub desc_en: String,
    #[serde(default)]
    pub word_count: usize,
}

impl WellKnownSetMeta {
    pub fn title(&self, lang: &NativeLanguage) -> &str {
        match lang {
            NativeLanguage::Russian => &self.title_ru,
            NativeLanguage::English => &self.title_en,
        }
    }

    pub fn description(&self, lang: &NativeLanguage) -> &str {
        match lang {
            NativeLanguage::Russian => &self.desc_ru,
            NativeLanguage::English => &self.desc_en,
        }
    }

    pub fn set_type_label(&self, lang: &NativeLanguage) -> String {
        get_types_meta()
            .map(|m| m.get_label(&self.set_type, lang).to_string())
            .unwrap_or_else(|| self.set_type.clone())
    }
}

#[derive(Clone)]
pub struct WellKnownSet {
    level: JapaneseLevel,
    words: Vec<String>,
}

impl WellKnownSet {
    pub fn new(level: JapaneseLevel, words: Vec<String>) -> Self {
        Self { level, words }
    }

    pub fn words(&self) -> &[String] {
        &self.words
    }

    pub fn level(&self) -> &JapaneseLevel {
        &self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("jlpt_n5", "Jlpt")]
    #[case("jlpt_n4", "Jlpt")]
    #[case("jlpt_n3", "Jlpt")]
    #[case("jlpt_n2", "Jlpt")]
    #[case("jlpt_n1", "Jlpt")]
    #[case("migii_n5_1", "Migii")]
    #[case("migii_n4_grammar", "Migii")]
    #[case("spy_family_s01_e01", "SpyFamily")]
    #[case("spy_family_s01_e12", "SpyFamily")]
    #[case("word_en_1", "DuolingoEn")]
    #[case("lesson_en_01", "DuolingoEn")]
    #[case("word_ru_1", "DuolingoRu")]
    #[case("lesson_ru_01", "DuolingoRu")]
    #[case("duolingo_n5_animals", "DuolingoRu")]
    #[case("duolingo_n4_verbs", "DuolingoRu")]
    #[case("minna_n5_01", "MinnaNoNihongo")]
    #[case("minna_n4_26", "MinnaNoNihongo")]
    #[case("minna_n3_01", "MinnaNoNihongo")]
    #[case("minna_n3_12", "MinnaNoNihongo")]
    #[case("minna_n2_13", "MinnaNoNihongo")]
    #[case("minna_n2_extra", "MinnaNoNihongo")]
    #[case("irodori_nyuumon_01", "Irodori")]
    #[case("irodori_nyuumon_18", "Irodori")]
    #[case("irodori_shokyuu1_01", "Irodori")]
    #[case("irodori_shokyuu1_10", "Irodori")]
    #[case("irodori_shokyuu2_01", "Irodori")]
    #[case("irodori_shokyuu2_18", "Irodori")]
    #[case("unknown_id", "unknown")]
    #[case("some_random_set", "some")]
    fn id_to_set_type_various_prefixes(#[case] id: &str, #[case] expected: &str) {
        assert_eq!(id_to_set_type(id), expected);
    }

    #[rstest]
    #[case("jlpt_n5", "well_known_set/jlpt_n5.json")]
    #[case("jlpt_n4", "well_known_set/jlpt_n4.json")]
    #[case("migii_n5_basic", "well_known_set/migii/n5/migii_n5_basic.json")]
    #[case("migii_n4_grammar", "well_known_set/migii/n4/migii_n4_grammar.json")]
    #[case("duolingo_n5_animals", "well_known_set/duolingo/n5/n5_animals.json")]
    #[case("duolingo_n4_verbs", "well_known_set/duolingo/n4/n4_verbs.json")]
    #[case(
        "duolingo_n5_duolingo_ru_n5_1",
        "well_known_set/duolingo/n5/duolingo_ru_n5_1.json"
    )]
    #[case(
        "duolingo_n4_duolingo_jp_n4_2",
        "well_known_set/duolingo/n4/duolingo_jp_n4_2.json"
    )]
    #[case("minna_n5_01", "well_known_set/minna_n5/minna_n5_01.json")]
    #[case("minna_n4_26", "well_known_set/minna_n4/minna_n4_26.json")]
    #[case("minna_n3_01", "well_known_set/minna_n3/minna_n3_01.json")]
    #[case("minna_n3_12", "well_known_set/minna_n3/minna_n3_12.json")]
    #[case("minna_n2_13", "well_known_set/minna_n2/minna_n2_13.json")]
    #[case("minna_n2_extra", "well_known_set/minna_n2/minna_n2_extra.json")]
    #[case(
        "irodori_nyuumon_01",
        "well_known_set/irodori_nyuumon/irodori_nyuumon_01.json"
    )]
    #[case(
        "irodori_nyuumon_18",
        "well_known_set/irodori_nyuumon/irodori_nyuumon_18.json"
    )]
    #[case(
        "irodori_shokyuu1_01",
        "well_known_set/irodori_shokyuu1/irodori_shokyuu1_01.json"
    )]
    #[case(
        "irodori_shokyuu1_10",
        "well_known_set/irodori_shokyuu1/irodori_shokyuu1_10.json"
    )]
    #[case(
        "irodori_shokyuu2_01",
        "well_known_set/irodori_shokyuu2/irodori_shokyuu2_01.json"
    )]
    #[case(
        "irodori_shokyuu2_18",
        "well_known_set/irodori_shokyuu2/irodori_shokyuu2_18.json"
    )]
    #[case(
        "spy_family_s01_e01",
        "well_known_set/spy_family/spy_family_s01_e01.json"
    )]
    #[case(
        "spy_family_s01_e12",
        "well_known_set/spy_family/spy_family_s01_e12.json"
    )]
    #[case("random_id", "well_known_set/random_id.json")]
    fn resolve_set_path_formats(#[case] id: &str, #[case] expected: &str) {
        assert_eq!(resolve_set_path(id), expected);
    }

    #[test]
    fn resolve_set_path_returns_fallback_for_spy_family_prefix_without_episode_suffix() {
        // Real spy_family ids always carry an episode suffix (spy_family_sNN_eNN), so a
        // bare prefix is not a valid id and intentionally falls through to the generic path.
        assert_eq!(
            resolve_set_path("spy_family"),
            "well_known_set/spy_family.json"
        );
    }

    #[test]
    fn resolve_set_path_handles_special_characters() {
        let id_with_dots = "test..dots";
        let path = resolve_set_path(id_with_dots);
        assert!(path.starts_with("well_known_set/"));
        assert!(path.ends_with(".json"));
    }

    #[test]
    fn types_meta_get_label_returns_correct_labels() {
        let meta = TypesMeta {
            types: vec![TypeMeta {
                id: "Test".to_string(),
                label_ru: "Тестовая метка".to_string(),
                label_en: "Test Label".to_string(),
            }],
        };

        assert_eq!(
            meta.get_label(&"Test".to_string(), &NativeLanguage::Russian),
            "Тестовая метка"
        );
        assert_eq!(
            meta.get_label(&"Test".to_string(), &NativeLanguage::English),
            "Test Label"
        );
        assert_eq!(
            meta.get_label(&"nonexistent".to_string(), &NativeLanguage::Russian),
            "nonexistent"
        );
    }

    #[test]
    fn well_known_set_meta_accessors() {
        let meta = WellKnownSetMeta {
            id: "test_set".to_string(),
            set_type: "Test".to_string(),
            level: JapaneseLevel::N5,
            title_ru: "Тестовый набор".to_string(),
            title_en: "Test Set".to_string(),
            desc_ru: "Тестовое описание".to_string(),
            desc_en: "A test set".to_string(),
            word_count: 0,
        };

        assert_eq!(meta.title(&NativeLanguage::English), "Test Set");
        assert_eq!(meta.title(&NativeLanguage::Russian), "Тестовый набор");
        assert_eq!(meta.description(&NativeLanguage::English), "A test set");
        assert_eq!(
            meta.description(&NativeLanguage::Russian),
            "Тестовое описание"
        );
        assert_eq!(meta.level, JapaneseLevel::N5);
    }

    #[test]
    fn well_known_set_accessors() {
        let set = WellKnownSet::new(
            JapaneseLevel::N5,
            vec!["word1".to_string(), "word2".to_string()],
        );

        assert_eq!(set.level(), &JapaneseLevel::N5);
        assert_eq!(set.words().len(), 2);
        assert_eq!(set.words(), &["word1".to_string(), "word2".to_string()]);
    }
}
