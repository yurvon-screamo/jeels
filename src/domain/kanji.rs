use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Serialize};

use crate::domain::{JeersError, value_objects::JapaneseLevel};

const RADKFILE_DATA: &str = include_str!("./radicals.json");
const KANJI_DATA: &str = include_str!("./kanji.json");

pub static RADKFILE: LazyLock<Radkfile> = LazyLock::new(Radkfile::new);
pub static KANJI_DB: LazyLock<KanjiDatabase> = LazyLock::new(KanjiDatabase::new);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RadicalInfo {
    pub radical: char,
    pub stroke_count: u32,
    pub code: Option<String>,
    pub name: String,
    pub description: String,
    pub jlpt: JapaneseLevel,
    pub kanji: Vec<char>,
}

pub struct Radkfile {
    known_radicals: Vec<char>,
    radical_map: HashMap<char, RadicalInfo>,
}

#[derive(Serialize, Deserialize)]
struct RadicalStoredType {
    #[serde(rename = "strokeCount")]
    stroke_count: u32,
    code: Option<String>,
    kanji: Vec<String>,
    name: String,
    description: String,
    jlpt: String,
}

#[derive(Serialize, Deserialize)]
struct RadkfilStoredType {
    radicals: HashMap<String, RadicalStoredType>,
}

impl Default for Radkfile {
    fn default() -> Self {
        Self::new()
    }
}

impl Radkfile {
    pub fn new() -> Self {
        let radkfile: RadkfilStoredType = serde_json::from_str(RADKFILE_DATA).unwrap();
        let radical_map = radkfile
            .radicals
            .into_iter()
            .map(|(k, v)| {
                let radical_char = k.chars().next().unwrap();
                let jlpt = parse_jlpt_level(&v.jlpt);
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
                        code: v.code,
                        name: v.name,
                        description: v.description,
                        jlpt,
                        kanji,
                    },
                )
            })
            .collect::<HashMap<char, RadicalInfo>>();

        let known_radicals: Vec<_> = radical_map.keys().copied().collect();

        Self {
            known_radicals,
            radical_map,
        }
    }

    pub fn get_radical_kanji(&self, radical: &char) -> Result<Vec<char>, JeersError> {
        Ok(self
            .radical_map
            .get(radical)
            .ok_or(JeersError::KradfileError {
                reason: format!("Radical {} not found in radkfile", radical),
            })?
            .kanji
            .clone())
    }

    pub fn get_radical_info(&self, radical: &char) -> Result<&RadicalInfo, JeersError> {
        self.radical_map
            .get(radical)
            .ok_or(JeersError::KradfileError {
                reason: format!("Radical {} not found in radkfile", radical),
            })
    }

    pub fn known_radicals(&self) -> &[char] {
        &self.known_radicals
    }
}

#[derive(Debug, Clone)]
pub struct KanjiInfo {
    pub kanji: char,
    pub jlpt: JapaneseLevel,
    pub used_in: u32,
    pub description: String,
    pub radicals: Vec<char>,
}

#[derive(Serialize, Deserialize)]
struct KanjiStoredType {
    kanji: String,
    jlpt: String,
    used_in: u32,
    description: String,
    radicals: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct KanjiDatabaseStoredType {
    kanji: Vec<KanjiStoredType>,
}

pub struct KanjiDatabase {
    kanji_map: HashMap<char, KanjiInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KanjiCard {
    pub kanji: char,
    pub jlpt: JapaneseLevel,
    pub used_in: u32,
    pub description: String,
    pub radicals_info: Vec<RadicalInfo>,
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
                    kanji_char,
                    KanjiInfo {
                        kanji: kanji_char,
                        jlpt,
                        used_in: k.used_in,
                        description: k.description,
                        radicals,
                    },
                )
            })
            .collect::<HashMap<char, KanjiInfo>>();

        Self { kanji_map }
    }

    pub fn get_kanji_info(&self, kanji: &char) -> Result<&KanjiInfo, JeersError> {
        self.kanji_map.get(kanji).ok_or(JeersError::KradfileError {
            reason: format!("Kanji {} not found in kanji database", kanji),
        })
    }

    pub fn get_kanji_card(&self, kanji: &char) -> Result<KanjiCard, JeersError> {
        let kanji_info = self.get_kanji_info(kanji)?.clone();
        let radicals_info: Vec<_> = kanji_info
            .radicals
            .iter()
            .filter_map(|r| RADKFILE.get_radical_info(r).ok().cloned())
            .collect();

        Ok(KanjiCard {
            kanji: kanji_info.kanji,
            jlpt: kanji_info.jlpt,
            used_in: kanji_info.used_in,
            description: kanji_info.description,
            radicals_info,
        })
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
