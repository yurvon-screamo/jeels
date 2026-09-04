use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::domain::OrigaError;

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ReadingSpan {
    pub start_index: usize,
    pub end_index: usize,
    pub text: String,
}

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct FuriganaEntry {
    pub text: String,
    pub reading: String,
    pub reading_spans: Vec<ReadingSpan>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct FuriganaDictionary {
    entries: BTreeMap<String, Vec<FuriganaEntry>>,
}

static FURIGANA_DICT: OnceLock<FuriganaDictionary> = OnceLock::new();

pub fn is_furigana_dict_loaded() -> bool {
    FURIGANA_DICT.get().is_some()
}

/// Build the dictionary from JmdictFurigana text without installing it into
/// the global slot. Used by the CDN blob builder and by loader tests.
pub fn build_furigana_dict_from_text(content: &str) -> Result<FuriganaDictionary, OrigaError> {
    FuriganaDictionary::from_text(content)
}

/// Install an already-built dictionary (e.g. deserialized from a CDN blob)
/// into the global slot. Fails if another dictionary is already installed.
pub fn set_furigana_dict(dict: FuriganaDictionary) -> Result<(), OrigaError> {
    FURIGANA_DICT
        .set(dict)
        .map_err(|_| OrigaError::FuriganaError {
            reason: "Furigana dictionary already loaded".to_string(),
        })
}

/// Serialize a built dictionary to rkyv bytes (payload of a CDN blob).
pub fn serialize_furigana_dict_to_rkyv(dict: &FuriganaDictionary) -> Result<Vec<u8>, OrigaError> {
    rkyv::to_bytes::<rkyv::rancor::Error>(dict)
        .map(|bytes| bytes.to_vec())
        .map_err(|e| OrigaError::FuriganaError {
            reason: format!("failed to serialize furigana dictionary: {e}"),
        })
}

/// Deserialize a dictionary from rkyv payload bytes (CDN blob payload).
pub fn furigana_dict_from_rkyv(payload: &[u8]) -> Result<FuriganaDictionary, OrigaError> {
    rkyv::from_bytes::<FuriganaDictionary, rkyv::rancor::Error>(payload).map_err(|e| {
        OrigaError::FuriganaError {
            reason: format!("failed to deserialize furigana dictionary: {e}"),
        }
    })
}

pub fn init_furigana_dict(content: &str) -> Result<(), OrigaError> {
    let dict = FuriganaDictionary::from_text(content)?;
    FURIGANA_DICT
        .set(dict)
        .map_err(|_| OrigaError::FuriganaError {
            reason: "Furigana dictionary already loaded".to_string(),
        })
}

pub fn get_furigana_dict() -> Option<&'static FuriganaDictionary> {
    FURIGANA_DICT.get()
}

impl FuriganaDictionary {
    pub fn from_text(content: &str) -> Result<Self, OrigaError> {
        let mut entries: BTreeMap<String, Vec<FuriganaEntry>> = BTreeMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Some(entry) = parse_line(trimmed) {
                entries.entry(entry.text.clone()).or_default().push(entry);
            }
        }

        Ok(Self { entries })
    }

    pub fn lookup_word(&self, word: &str) -> Vec<&FuriganaEntry> {
        self.entries
            .get(word)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn lookup_prefixed(&self, prefix: &str) -> Vec<&FuriganaEntry> {
        self.entries
            .range(prefix.to_string()..)
            .map_while(|(text, entries)| {
                if text.starts_with(prefix) {
                    Some(entries.iter())
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }
}

fn parse_line(line: &str) -> Option<FuriganaEntry> {
    let parts: Vec<&str> = line.splitn(3, '|').collect();
    if parts.len() != 3 {
        return None;
    }

    let text = parts[0].to_string();
    let reading = parts[1].to_string();
    let reading_spans = parse_reading_spans(parts[2])?;

    Some(FuriganaEntry {
        text,
        reading,
        reading_spans,
    })
}

fn parse_reading_spans(input: &str) -> Option<Vec<ReadingSpan>> {
    if input.is_empty() {
        return Some(vec![]);
    }

    input
        .split(';')
        .map(|span| {
            let (range, text) = span.split_once(':')?;
            let (start, end) = if let Some((s, e)) = range.split_once('-') {
                (s.parse().ok()?, e.parse().ok()?)
            } else {
                let s = range.parse().ok()?;
                (s, s)
            };
            Some(ReadingSpan {
                start_index: start,
                end_index: end,
                text: text.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_span() {
        let entry = parse_line("指|ゆび|0:ゆび").unwrap();
        assert_eq!(entry.text, "指");
        assert_eq!(entry.reading, "ゆび");
        assert_eq!(entry.reading_spans.len(), 1);
        assert_eq!(entry.reading_spans[0].start_index, 0);
        assert_eq!(entry.reading_spans[0].end_index, 0);
        assert_eq!(entry.reading_spans[0].text, "ゆび");
    }

    #[test]
    fn parse_range_span() {
        let entry = parse_line("大人|おとな|0-1:おとな").unwrap();
        assert_eq!(entry.text, "大人");
        assert_eq!(entry.reading, "おとな");
        assert_eq!(entry.reading_spans.len(), 1);
        assert_eq!(entry.reading_spans[0].start_index, 0);
        assert_eq!(entry.reading_spans[0].end_index, 1);
        assert_eq!(entry.reading_spans[0].text, "おとな");
    }

    #[test]
    fn parse_multiple_spans() {
        let entry = parse_line("間に合う|まにあう|0:ま;2:あ").unwrap();
        assert_eq!(entry.text, "間に合う");
        assert_eq!(entry.reading, "まにあう");
        assert_eq!(entry.reading_spans.len(), 2);
        assert_eq!(entry.reading_spans[0].start_index, 0);
        assert_eq!(entry.reading_spans[0].end_index, 0);
        assert_eq!(entry.reading_spans[0].text, "ま");
        assert_eq!(entry.reading_spans[1].start_index, 2);
        assert_eq!(entry.reading_spans[1].end_index, 2);
        assert_eq!(entry.reading_spans[1].text, "あ");
    }

    #[test]
    fn parse_complex_entry() {
        let entry = parse_line("方程式|ほうていしき|0:ほう;1:てい;2:しき").unwrap();
        assert_eq!(entry.text, "方程式");
        assert_eq!(entry.reading_spans.len(), 3);
        assert_eq!(entry.reading_spans[0].text, "ほう");
        assert_eq!(entry.reading_spans[1].text, "てい");
        assert_eq!(entry.reading_spans[2].text, "しき");
    }

    #[test]
    fn parse_partial_reading() {
        let entry = parse_line("食べる|たべる|0:た").unwrap();
        assert_eq!(entry.reading_spans.len(), 1);
        assert_eq!(entry.reading_spans[0].text, "た");
    }

    #[test]
    fn empty_input_produces_empty_dict() {
        let dict = FuriganaDictionary::from_text("").unwrap();
        assert!(dict.lookup_word("指").is_empty());
    }

    #[test]
    fn invalid_lines_are_skipped() {
        let content = "invalid_line_without_pipes\n指|ゆび|0:ゆび\nalso_invalid";
        let dict = FuriganaDictionary::from_text(content).unwrap();
        let results = dict.lookup_word("指");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].reading, "ゆび");
    }

    #[test]
    fn lookup_word_returns_all_matches() {
        let content = "大人|おとな|0-1:おとな\n大人|だいじん|0-1:だいじん";
        let dict = FuriganaDictionary::from_text(content).unwrap();
        let results = dict.lookup_word("大人");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn lookup_word_returns_empty_for_unknown() {
        let content = "指|ゆび|0:ゆび";
        let dict = FuriganaDictionary::from_text(content).unwrap();
        assert!(dict.lookup_word("手").is_empty());
    }

    #[test]
    fn lookup_prefixed_returns_matching_entries() {
        let content = "食べる|たべる|0:た\n食べ物|たべもの|0-1:たべもの\n飲む|のむ|0:のむ";
        let dict = FuriganaDictionary::from_text(content).unwrap();
        let results = dict.lookup_prefixed("食べ");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn lookup_prefixed_returns_empty_for_no_match() {
        let content = "食べる|たべる|0:た";
        let dict = FuriganaDictionary::from_text(content).unwrap();
        assert!(dict.lookup_prefixed("飲").is_empty());
    }

    #[test]
    fn rkyv_payload_round_trip_preserves_lookups() {
        // Arrange
        let content = "大人|おとな|0-1:おとな\n大人|だいじん|0-1:だいじん\n指|ゆび|0:ゆび";
        let dict = build_furigana_dict_from_text(content).unwrap();

        // Act
        let payload = serialize_furigana_dict_to_rkyv(&dict).unwrap();
        let restored = furigana_dict_from_rkyv(&payload).unwrap();

        // Assert
        assert_eq!(restored.lookup_word("大人").len(), 2);
        assert_eq!(restored.lookup_word("指").len(), 1);
        assert_eq!(restored.lookup_prefixed("大").len(), 2);
    }

    #[test]
    fn integration_lookup_across_multiple_entries() {
        let content = "\
指|ゆび|0:ゆび
間に合う|まにあう|0:ま;2:あ
大人|おとな|0-1:おとな
方程式|ほうていしき|0:ほう;1:てい;2:しき
食べる|たべる|0:た";
        let dict = FuriganaDictionary::from_text(content).unwrap();

        let yubi = dict.lookup_word("指");
        assert_eq!(yubi.len(), 1);
        assert_eq!(yubi[0].reading_spans[0].text, "ゆび");

        let ok = dict.lookup_word("間に合う");
        assert_eq!(ok.len(), 1);
        assert_eq!(ok[0].reading_spans.len(), 2);

        let prefixed = dict.lookup_prefixed("大");
        assert_eq!(prefixed.len(), 1);
        assert_eq!(prefixed[0].text, "大人");
    }
}
