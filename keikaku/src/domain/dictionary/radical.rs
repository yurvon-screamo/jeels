use std::{collections::HashMap, sync::LazyLock};

use serde::{Deserialize, Serialize};

use crate::domain::{
    JeersError, dictionary::kanji::parse_jlpt_level, value_objects::JapaneseLevel,
};

const RADKFILE_DATA: &str = include_str!("./radicals.json");
pub static RADICAL_DB: LazyLock<RadicalDatabase> = LazyLock::new(RadicalDatabase::new);
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn jlpt(&self) -> &JapaneseLevel {
        &self.jlpt
    }

    pub fn kanji(&self) -> &[char] {
        &self.kanji
    }
}

impl Default for RadicalDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl RadicalDatabase {
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

#[derive(Serialize, Deserialize)]
struct RadicalStoredType {
    #[serde(rename = "strokeCount")]
    stroke_count: u32,
    kanji: Vec<String>,
    name: String,
    description: String,
    jlpt: String,
}

#[derive(Serialize, Deserialize)]
struct RadkfilStoredType {
    radicals: HashMap<String, RadicalStoredType>,
}
