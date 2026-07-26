use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::domain::OrigaError;

pub static PITCH_AUDIO_INDEX: OnceLock<PitchAudioIndex> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct PitchAudioEntry {
    file: String,
    pitch: Option<u8>,
}

impl PitchAudioEntry {
    pub fn file(&self) -> &str {
        &self.file
    }

    pub fn pitch(&self) -> Option<u8> {
        self.pitch
    }

    pub fn cdn_path(&self) -> String {
        format!("pitch/audio/{}", self.file)
    }
}

pub struct PitchAudioIndex {
    entries: HashMap<String, PitchAudioEntry>,
    version: u32,
}

#[derive(Deserialize)]
struct IndexFile {
    #[serde(rename = "v")]
    version: u32,
    #[serde(rename = "total")]
    _total: u32,
    entries: HashMap<String, IndexEntryRaw>,
}

#[derive(Deserialize)]
struct IndexEntryRaw {
    #[serde(rename = "f")]
    file: String,
    #[serde(rename = "p")]
    pitch: Option<u8>,
}

impl PitchAudioIndex {
    fn from_json(json: &str) -> Result<Self, OrigaError> {
        let file: IndexFile =
            serde_json::from_str(json).map_err(|e| OrigaError::PitchAudioParseError {
                reason: format!("Failed to parse pitch audio index: {}", e),
            })?;

        let entries: HashMap<String, PitchAudioEntry> = file
            .entries
            .into_iter()
            .map(|(word, raw)| {
                (
                    word,
                    PitchAudioEntry {
                        file: raw.file,
                        pitch: raw.pitch,
                    },
                )
            })
            .collect();

        Ok(Self {
            entries,
            version: file.version,
        })
    }

    fn get_entry(&self, word: &str) -> Option<&PitchAudioEntry> {
        self.entries.get(word)
    }

    /// Lookup by word+reading with fallback chain: "word|reading" → "reading" → "word".
    pub fn find_audio_for_reading(&self, word: &str, reading: &str) -> Option<&PitchAudioEntry> {
        let composite = format!("{}|{}", word, reading);
        self.get_entry(&composite)
            .or_else(|| self.get_entry(reading))
            .or_else(|| self.get_entry(word))
    }
}

pub fn init_pitch_audio_index(json: &str) -> Result<(), OrigaError> {
    let index = PitchAudioIndex::from_json(json)?;
    PITCH_AUDIO_INDEX
        .set(index)
        .map_err(|_| OrigaError::PitchAudioParseError {
            reason: "Pitch audio index already initialized".to_string(),
        })
}

pub fn is_pitch_audio_loaded() -> bool {
    PITCH_AUDIO_INDEX.get().is_some()
}

/// Simple word lookup. Prefer [`get_audio_for_reading`] which uses tokenizer
/// reading for correct pitch matching.
pub fn get_audio_for_word(word: &str) -> Option<&'static PitchAudioEntry> {
    PITCH_AUDIO_INDEX.get().and_then(|idx| idx.get_entry(word))
}

/// Lookup pitch audio by word and reading (from tokenizer).
/// Tries: "word|reading" → "reading" → "word".
pub fn get_audio_for_reading(word: &str, reading: &str) -> Option<&'static PitchAudioEntry> {
    PITCH_AUDIO_INDEX
        .get()
        .and_then(|idx| idx.find_audio_for_reading(word, reading))
}

pub fn pitch_audio_version() -> u32 {
    PITCH_AUDIO_INDEX.get().map(|idx| idx.version).unwrap_or(0)
}

pub fn get_audio_entry_count() -> usize {
    PITCH_AUDIO_INDEX
        .get()
        .map(|idx| idx.entries.len())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index_json() -> &'static str {
        r#"{"v":2,"total":3,"entries":{"猫":{"f":"a1b2c3d4.opus","p":1},"食べる":{"f":"e5f6a7b8.opus","p":0},"あ":{"f":"c9d0e1f2.opus","p":null}}}"#
    }

    fn index_v3_json() -> &'static str {
        r#"{"v":3,"total":5,"entries":{"役|やく":{"f":"yaku.opus","p":1},"役|えき":{"f":"eki.opus","p":0},"やく":{"f":"yaku_kana.opus","p":1},"えき":{"f":"eki_kana.opus","p":0},"役":{"f":"fallback.opus","p":0}}}"#
    }

    #[test]
    fn from_json_valid() {
        let index = PitchAudioIndex::from_json(index_json()).expect("valid JSON should parse");
        assert_eq!(index.version, 2);
        assert_eq!(index.entries.len(), 3);
    }

    #[test]
    fn from_json_invalid() {
        let result = PitchAudioIndex::from_json("not json");
        assert!(result.is_err());
    }

    #[test]
    fn get_entry_found() {
        let index = PitchAudioIndex::from_json(index_json()).expect("valid JSON");
        let entry = index.get_entry("猫").expect("entry should exist");
        assert_eq!(entry.file(), "a1b2c3d4.opus");
        assert_eq!(entry.pitch(), Some(1));
    }

    #[test]
    fn get_entry_not_found() {
        let index = PitchAudioIndex::from_json(index_json()).expect("valid JSON");
        assert!(index.get_entry("犬").is_none());
    }

    #[test]
    fn entry_with_null_pitch() {
        let index = PitchAudioIndex::from_json(index_json()).expect("valid JSON");
        let entry = index.get_entry("あ").expect("entry should exist");
        assert_eq!(entry.pitch(), None);
    }

    #[test]
    fn cdn_path_format() {
        let index = PitchAudioIndex::from_json(index_json()).expect("valid JSON");
        let entry = index.get_entry("猫").expect("entry should exist");
        assert_eq!(entry.cdn_path(), "pitch/audio/a1b2c3d4.opus");
    }

    #[test]
    fn get_audio_for_word_before_init() {
        assert!(get_audio_for_word("猫").is_none());
    }

    #[test]
    fn pitch_audio_version_before_init() {
        assert_eq!(pitch_audio_version(), 0);
    }

    #[test]
    fn get_audio_for_reading_composite_key() {
        let index = PitchAudioIndex::from_json(index_v3_json()).expect("valid JSON");
        let entry = index
            .get_entry("役|やく")
            .expect("composite key should match");
        assert_eq!(entry.file(), "yaku.opus");
        assert_eq!(entry.pitch(), Some(1));
    }

    #[test]
    fn get_audio_for_reading_fallback_to_kana() {
        let index = PitchAudioIndex::from_json(index_v3_json()).expect("valid JSON");
        let entry = index.get_entry("えき").expect("kana key should match");
        assert_eq!(entry.file(), "eki_kana.opus");
    }

    #[test]
    fn get_audio_for_reading_fallback_to_kanji() {
        let index = PitchAudioIndex::from_json(index_v3_json()).expect("valid JSON");
        let entry = index.get_entry("役").expect("kanji fallback should match");
        assert_eq!(entry.file(), "fallback.opus");
    }

    #[test]
    fn get_audio_for_reading_no_match() {
        let index = PitchAudioIndex::from_json(index_v3_json()).expect("valid JSON");
        assert!(index.get_entry("NotExist|xyz").is_none());
    }

    #[test]
    fn get_audio_for_reading_priority_chain() {
        let index = PitchAudioIndex::from_json(index_v3_json()).expect("valid JSON");
        // "役|やく" should return composite entry, not kanji fallback
        let composite = index.get_entry("役|やく").expect("should exist");
        assert_eq!(composite.file(), "yaku.opus");
        // plain "役" should return kanji fallback
        let kanji = index.get_entry("役").expect("should exist");
        assert_eq!(kanji.file(), "fallback.opus");
    }

    #[test]
    fn get_audio_for_reading_fallback_chain_integration() {
        let index = PitchAudioIndex::from_json(index_v3_json()).expect("valid JSON");

        let entry = index
            .find_audio_for_reading("役", "やく")
            .expect("composite should match");
        assert_eq!(entry.file(), "yaku.opus");

        let entry = index
            .find_audio_for_reading("NotExist", "えき")
            .expect("kana fallback");
        assert_eq!(entry.file(), "eki_kana.opus");

        let entry = index
            .find_audio_for_reading("役", "NotExist")
            .expect("kanji fallback");
        assert_eq!(entry.file(), "fallback.opus");
    }
}
