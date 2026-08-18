use std::collections::HashSet;

use crate::dictionary::furigana_dict::{ReadingSpan, is_furigana_dict_loaded};
use crate::domain::furigana_annotator::AnnotatedSpan;
use crate::domain::{OrigaError, japanese::JapaneseChar, tokenizer::tokenize_text};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FuriganaSegment {
    text: String,
    reading: Option<String>,
    is_known: bool,
}

impl FuriganaSegment {
    pub fn new(text: String, reading: Option<String>, is_known: bool) -> Self {
        Self {
            text,
            reading,
            is_known,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn reading(&self) -> Option<&str> {
        self.reading.as_deref()
    }

    pub fn has_reading(&self) -> bool {
        self.reading.is_some()
    }

    pub fn is_known(&self) -> bool {
        self.is_known
    }
}

pub fn furiganize_segments(
    text: &str,
    known_kanji: &HashSet<char>,
) -> Result<Vec<FuriganaSegment>, OrigaError> {
    if is_furigana_dict_loaded() {
        let spans = crate::domain::furigana_annotator::annotate_text(text)?;
        return Ok(spans_to_segments(spans, known_kanji));
    }

    let tokens = tokenize_text(text).map_err(|e| OrigaError::FuriganaError {
        reason: e.to_string(),
    })?;
    Ok(tokens
        .into_iter()
        .map(|token| token_to_furigana_segment(token, known_kanji))
        .collect())
}

fn token_to_furigana_segment(
    token: crate::domain::tokenizer::TokenInfo,
    known_kanji: &HashSet<char>,
) -> FuriganaSegment {
    let surface = token.orthographic_surface_form().to_string();
    let contains_kanji = surface.chars().any(|c| c.is_kanji());

    if contains_kanji {
        let all_kanji_known = surface
            .chars()
            .filter(|c| c.is_kanji())
            .all(|c| known_kanji.contains(&c));

        let reading = token.phonological_surface_form();
        let reading = if reading.is_empty() {
            None
        } else {
            Some(reading.to_string())
        };
        FuriganaSegment::new(surface, reading, all_kanji_known)
    } else {
        FuriganaSegment::new(surface, None, false)
    }
}

fn spans_to_segments(
    spans: Vec<AnnotatedSpan>,
    known_kanji: &HashSet<char>,
) -> Vec<FuriganaSegment> {
    spans
        .into_iter()
        .flat_map(|span| {
            let has_kanji = span.text.chars().any(|c| c.is_kanji());
            if span.reading_spans.is_empty() || !has_kanji {
                let reading = if has_kanji { span.reading } else { None };
                let is_known = reading.is_some()
                    && span
                        .text
                        .chars()
                        .filter(|c| c.is_kanji())
                        .all(|c| known_kanji.contains(&c));
                vec![FuriganaSegment::new(span.text, reading, is_known)]
            } else {
                apply_reading_spans(&span.text, &span.reading_spans, known_kanji)
            }
        })
        .collect()
}

fn apply_reading_spans(
    text: &str,
    spans: &[ReadingSpan],
    known_kanji: &HashSet<char>,
) -> Vec<FuriganaSegment> {
    let chars: Vec<char> = text.chars().collect();
    let mut segments = Vec::new();
    let mut last_end: usize = 0;

    for span in spans {
        let start = span.start_index;
        let end = span.end_index;

        if start > last_end {
            let gap: String = chars[last_end..start].iter().collect();
            if !gap.is_empty() {
                segments.push(FuriganaSegment::new(gap, None, false));
            }
        }

        let base_text: String = chars[start..=end].iter().collect();
        let is_known = base_text
            .chars()
            .filter(|c| c.is_kanji())
            .all(|c| known_kanji.contains(&c));
        segments.push(FuriganaSegment::new(
            base_text,
            Some(span.text.clone()),
            is_known,
        ));

        last_end = end + 1;
    }

    if last_end < chars.len() {
        let tail: String = chars[last_end..].iter().collect();
        if !tail.is_empty() {
            segments.push(FuriganaSegment::new(tail, None, false));
        }
    }

    segments
}

fn html_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            _ => result.push(c),
        }
    }
    result
}

pub fn furiganize_text_html(segments: &[FuriganaSegment]) -> String {
    segments
        .iter()
        .map(|seg| match &seg.reading {
            Some(reading) => {
                let class = if seg.is_known { "furigana-hidden furigana-ruby" } else { "furigana-ruby" };
                format!(
                    "<ruby class=\"{}\">{}<rp>(</rp><rt class=\"furigana-rt\">{}</rt><rp>)</rp></ruby>",
                    class, html_escape(&seg.text), html_escape(reading)
                )
            }
            None => html_escape(&seg.text),
        })
        .collect()
}

pub fn furiganize_text(text: &str, known_kanji: &HashSet<char>) -> Result<String, OrigaError> {
    let segments = furiganize_segments(text, known_kanji)?;
    Ok(furiganize_text_html(&segments))
}

#[cfg(test)]
mod tests {
    use std::{env, fs, io::Read, path::PathBuf};

    use flate2::read::DeflateDecoder;

    use super::*;
    use crate::domain::{DictionaryData, init_dictionary, is_dictionary_loaded};

    fn decompress(data: Vec<u8>) -> Vec<u8> {
        let mut decoder = DeflateDecoder::new(&data[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        decompressed
    }

    fn ensure_dictionary() {
        if is_dictionary_loaded() {
            return;
        }

        let base = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .parent()
            .unwrap()
            .join("cdn")
            .join("dictionaries");
        // Prefer the versioned SudachiDict directory (see deploy_cdn.py);
        // fall back to a stale build-script output.
        let dict_dir = if let Ok(out_dir) = env::var("OUT_DIR") {
            let out_dict = PathBuf::from(out_dir).join("lindera-unidic");
            let versioned = base.join("sudachidict-20260723");
            if versioned.join("dict.trie").exists() {
                versioned
            } else if out_dict.exists() {
                out_dict
            } else {
                base
            }
        } else {
            let versioned = base.join("sudachidict-20260723");
            if versioned.join("dict.trie").exists() {
                versioned
            } else {
                base
            }
        };

        let read_file = |name: &str| fs::read(dict_dir.join(name)).unwrap();

        let data = DictionaryData {
            char_def: decompress(read_file("char_def.bin")),
            matrix: decompress(read_file("matrix.mtx")),
            dict_trie: decompress(read_file("dict.trie")),
            dict_vals_idx: decompress(read_file("dict.valsidx")),
            dict_vals: decompress(read_file("dict.vals")),
            unk: decompress(read_file("unk.bin")),
            words_idx: decompress(read_file("dict.wordsidx")),
            words: decompress(read_file("dict.words")),
            metadata: read_file("metadata.json"),
        };

        init_dictionary(data).unwrap();
    }

    #[test]
    fn should_create_segment_with_reading() {
        let segment = FuriganaSegment::new("食べ".to_string(), Some("タベ".to_string()), false);
        assert_eq!(segment.text(), "食べ");
        assert_eq!(segment.reading(), Some("タベ"));
        assert!(segment.has_reading());
        assert!(!segment.is_known());
    }

    #[test]
    fn should_create_segment_without_reading() {
        let segment = FuriganaSegment::new("たべ".to_string(), None, false);
        assert_eq!(segment.text(), "たべ");
        assert_eq!(segment.reading(), None);
        assert!(!segment.has_reading());
    }

    #[test]
    fn should_create_segment_with_known_kanji() {
        let segment = FuriganaSegment::new("食".to_string(), Some("ショク".to_string()), true);
        assert!(segment.is_known());
    }

    #[test]
    fn should_furiganize_kanji_word_with_reading() {
        ensure_dictionary();
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = furiganize_segments("食べ物", &known_kanji).unwrap();
        assert!(!segments.is_empty());
        assert!(segments.iter().any(|s| s.has_reading()));
    }

    #[test]
    fn should_furiganize_hiragana_without_reading() {
        ensure_dictionary();
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = furiganize_segments("たべもの", &known_kanji).unwrap();
        assert!(!segments.is_empty());
        assert!(segments.iter().all(|s| !s.has_reading()));
    }

    // Standalone kanji that Lindera classifies as Unknown (no UniDic lemma on
    // its own — e.g. 璃 in character names) carries no derivable kana reading,
    // so the furigana segment must surface `reading == None` rather than
    // `Some("")`. An empty-string reading would render `<rt></rt>` and lie via
    // `has_reading() == true`.
    #[test]
    fn should_furiganize_unknown_kanji_without_empty_reading() {
        ensure_dictionary();
        let known_kanji: HashSet<char> = HashSet::new();
        // 杏璃 is a single token in SudachiDict now; 蕗 alone is not a lemma,
        // so a name ending in it takes the unknown-kanji path.
        let segments = furiganize_segments("杏蕩", &known_kanji).unwrap();
        let ri = segments
            .iter()
            .find(|s| s.text().contains("蕩"))
            .expect("unknown-kanji segment should exist");
        assert_eq!(ri.reading(), None);
        assert!(
            !ri.has_reading(),
            "unknown kanji must not carry an empty-string reading, got {:?}",
            ri.reading()
        );
    }

    #[test]
    fn should_furiganize_mixed_text() {
        ensure_dictionary();
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = furiganize_segments("食べます", &known_kanji).unwrap();
        assert!(!segments.is_empty());
    }

    #[test]
    fn should_furiganize_non_japanese_text() {
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = furiganize_segments("hello", &known_kanji).unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text(), "hello");
        assert!(!segments[0].has_reading());
    }

    #[test]
    fn should_furiganize_mixed_japanese_and_ascii() {
        ensure_dictionary();
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = furiganize_segments("hello食べ物world", &known_kanji).unwrap();
        assert!(!segments.is_empty());
        assert!(
            segments
                .iter()
                .any(|s| s.text() == "hello" && !s.has_reading())
        );
        assert!(
            segments
                .iter()
                .any(|s| s.text() == "world" && !s.has_reading())
        );
    }

    #[test]
    fn should_generate_html_for_segment_with_reading() {
        let segments = vec![FuriganaSegment::new(
            "食".to_string(),
            Some("ショク".to_string()),
            false,
        )];
        let html = furiganize_text_html(&segments);
        assert_eq!(
            html,
            "<ruby class=\"furigana-ruby\">食<rp>(</rp><rt class=\"furigana-rt\">ショク</rt><rp>)</rp></ruby>"
        );
    }

    #[test]
    fn should_generate_html_for_segment_without_reading() {
        let segments = vec![FuriganaSegment::new("たべ".to_string(), None, false)];
        let html = furiganize_text_html(&segments);
        assert_eq!(html, "たべ");
    }

    #[test]
    fn should_generate_html_for_mixed_segments() {
        let segments = vec![
            FuriganaSegment::new("食".to_string(), Some("ショク".to_string()), false),
            FuriganaSegment::new("べ".to_string(), None, false),
        ];
        let html = furiganize_text_html(&segments);
        assert_eq!(
            html,
            "<ruby class=\"furigana-ruby\">食<rp>(</rp><rt class=\"furigana-rt\">ショク</rt><rp>)</rp></ruby>べ"
        );
    }

    #[test]
    fn should_generate_html_with_hidden_furigana_for_known_kanji() {
        let segments = vec![FuriganaSegment::new(
            "食".to_string(),
            Some("ショク".to_string()),
            true,
        )];
        let html = furiganize_text_html(&segments);
        assert_eq!(
            html,
            "<ruby class=\"furigana-hidden furigana-ruby\">食<rp>(</rp><rt class=\"furigana-rt\">ショク</rt><rp>)</rp></ruby>"
        );
    }

    #[test]
    fn should_furiganize_text_backwards_compatible() {
        ensure_dictionary();
        let known_kanji: HashSet<char> = HashSet::new();
        let result = furiganize_text("食べ物", &known_kanji).unwrap();
        assert!(result.contains("<ruby"));
        assert!(result.contains("<rt class=\"furigana-rt\">"));
    }

    #[test]
    fn should_show_furigana_when_only_partial_kanji_known() {
        ensure_dictionary();
        let mut known_kanji: HashSet<char> = HashSet::new();
        known_kanji.insert('食');

        let segments = furiganize_segments("食べ物", &known_kanji).unwrap();

        let kanji_segments: Vec<_> = segments
            .iter()
            .filter(|s| s.text().chars().any(|c| c.is_kanji()))
            .collect();
        assert!(!kanji_segments.is_empty());
        assert!(kanji_segments.iter().any(|s| s.has_reading()));
        assert!(kanji_segments.iter().any(|s| !s.is_known()));
    }

    #[test]
    fn should_show_furigana_when_no_kanji_known() {
        ensure_dictionary();
        let known_kanji: HashSet<char> = HashSet::new();

        let segments = furiganize_segments("食べ物", &known_kanji).unwrap();

        let kanji_segments: Vec<_> = segments
            .iter()
            .filter(|s| s.text().chars().any(|c| c.is_kanji()))
            .collect();
        assert!(!kanji_segments.is_empty());
        assert!(kanji_segments.iter().all(|s| s.has_reading()));
        assert!(kanji_segments.iter().all(|s| !s.is_known()));
    }

    #[test]
    fn should_hide_furigana_when_all_kanji_known() {
        ensure_dictionary();
        let mut known_kanji: HashSet<char> = HashSet::new();
        known_kanji.insert('食');
        known_kanji.insert('物');

        let segments = furiganize_segments("食べ物", &known_kanji).unwrap();

        let kanji_segments: Vec<_> = segments
            .iter()
            .filter(|s| s.text().chars().any(|c| c.is_kanji()))
            .collect();
        assert!(!kanji_segments.is_empty());
        assert!(kanji_segments.iter().all(|s| s.has_reading()));
        assert!(kanji_segments.iter().all(|s| s.is_known()));
    }

    #[test]
    fn apply_reading_spans_single_kanji_per_span() {
        let spans = vec![
            ReadingSpan {
                start_index: 0,
                end_index: 0,
                text: "たべ".to_string(),
            },
            ReadingSpan {
                start_index: 2,
                end_index: 2,
                text: "もの".to_string(),
            },
        ];
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = apply_reading_spans("食べ物", &spans, &known_kanji);

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].text(), "食");
        assert_eq!(segments[0].reading(), Some("たべ"));
        assert_eq!(segments[1].text(), "べ");
        assert_eq!(segments[1].reading(), None);
        assert_eq!(segments[2].text(), "物");
        assert_eq!(segments[2].reading(), Some("もの"));
    }

    #[test]
    fn apply_reading_spans_multi_char_span() {
        let spans = vec![ReadingSpan {
            start_index: 0,
            end_index: 1,
            text: "おとな".to_string(),
        }];
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = apply_reading_spans("大人", &spans, &known_kanji);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text(), "大人");
        assert_eq!(segments[0].reading(), Some("おとな"));
    }

    #[test]
    fn apply_reading_spans_known_kanji_marks_segment() {
        let spans = vec![
            ReadingSpan {
                start_index: 0,
                end_index: 0,
                text: "たべ".to_string(),
            },
            ReadingSpan {
                start_index: 2,
                end_index: 2,
                text: "もの".to_string(),
            },
        ];
        let mut known_kanji: HashSet<char> = HashSet::new();
        known_kanji.insert('食');
        let segments = apply_reading_spans("食べ物", &spans, &known_kanji);

        assert!(segments[0].is_known());
        assert!(!segments[2].is_known());
    }

    #[test]
    fn apply_reading_spans_tail_after_last_span() {
        let spans = vec![ReadingSpan {
            start_index: 0,
            end_index: 0,
            text: "ほう".to_string(),
        }];
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = apply_reading_spans("方程式", &spans, &known_kanji);

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text(), "方");
        assert_eq!(segments[0].reading(), Some("ほう"));
        assert_eq!(segments[1].text(), "程式");
        assert_eq!(segments[1].reading(), None);
    }

    #[test]
    fn spans_to_segments_without_reading_spans() {
        let annotated = vec![AnnotatedSpan {
            text: "食べ物".to_string(),
            reading: Some("たべもの".to_string()),
            reading_spans: vec![],
        }];
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = spans_to_segments(annotated, &known_kanji);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text(), "食べ物");
        assert_eq!(segments[0].reading(), Some("たべもの"));
    }

    #[test]
    fn spans_to_segments_with_reading_spans() {
        let annotated = vec![AnnotatedSpan {
            text: "食べ物".to_string(),
            reading: Some("たべもの".to_string()),
            reading_spans: vec![
                ReadingSpan {
                    start_index: 0,
                    end_index: 0,
                    text: "たべ".to_string(),
                },
                ReadingSpan {
                    start_index: 2,
                    end_index: 2,
                    text: "もの".to_string(),
                },
            ],
        }];
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = spans_to_segments(annotated, &known_kanji);

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].text(), "食");
        assert_eq!(segments[0].reading(), Some("たべ"));
        assert_eq!(segments[1].text(), "べ");
        assert_eq!(segments[2].text(), "物");
        assert_eq!(segments[2].reading(), Some("もの"));
    }

    fn setup_dictionaries_for_integration() {
        ensure_dictionary();
        if !is_furigana_dict_loaded() {
            let content = "\
食べる|たべる|0:た
食べ物|たべもの|0:たべ;2:もの
大人|おとな|0-1:おとな
指|ゆび|0:ゆび
";
            let _ = crate::dictionary::furigana_dict::init_furigana_dict(content);
        }
    }

    #[test]
    fn furiganize_segments_uses_annotator_when_dict_loaded() {
        setup_dictionaries_for_integration();
        let known_kanji: HashSet<char> = HashSet::new();
        let segments = furiganize_segments("食べ物", &known_kanji).unwrap();

        let ta = segments
            .iter()
            .find(|s| s.text() == "食" && s.reading() == Some("タベ"));
        assert!(
            ta.is_some(),
            "expected segment '食' with reading 'タベ', got: {segments:?}"
        );

        let mo = segments
            .iter()
            .find(|s| s.text() == "物" && s.reading() == Some("モノ"));
        assert!(
            mo.is_some(),
            "expected segment '物' with reading 'モノ', got: {segments:?}"
        );
    }

    #[test]
    fn furiganize_segments_annotator_known_kanji() {
        setup_dictionaries_for_integration();
        let mut known_kanji: HashSet<char> = HashSet::new();
        known_kanji.insert('食');
        let segments = furiganize_segments("食べ物", &known_kanji).unwrap();

        let ta = segments.iter().find(|s| s.text() == "食").unwrap();
        assert!(ta.is_known());
        let mo = segments.iter().find(|s| s.text() == "物").unwrap();
        assert!(!mo.is_known());
    }
}
