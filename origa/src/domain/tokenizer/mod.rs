mod part_of_speech;
pub mod translation;

use std::sync::OnceLock;

pub use part_of_speech::PartOfSpeech;
pub use translation::{TokenTranslation, lookup_tokens_translations};

use crate::domain::{JapaneseChar, OrigaError, hiragana_to_katakana};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TokenInfo {
    orthographic_base_form: String,
    phonological_base_form: String,
    orthographic_surface_form: String,
    phonological_surface_form: String,
    part_of_speech: PartOfSpeech,
}

impl TokenInfo {
    pub fn orthographic_base_form(&self) -> &str {
        &self.orthographic_base_form
    }

    pub fn phonological_base_form(&self) -> &str {
        &self.phonological_base_form
    }

    pub fn orthographic_surface_form(&self) -> &str {
        &self.orthographic_surface_form
    }

    pub fn phonological_surface_form(&self) -> &str {
        &self.phonological_surface_form
    }

    pub fn part_of_speech(&self) -> &PartOfSpeech {
        &self.part_of_speech
    }
}

#[cfg(test)]
impl TokenInfo {
    pub fn new_test(base: &str, pos: PartOfSpeech) -> Self {
        Self::new_test_with_reading(base, base, pos)
    }

    pub fn new_test_with_reading(base: &str, reading: &str, pos: PartOfSpeech) -> Self {
        Self {
            orthographic_base_form: base.to_string(),
            phonological_base_form: reading.to_string(),
            orthographic_surface_form: base.to_string(),
            phonological_surface_form: reading.to_string(),
            part_of_speech: pos,
        }
    }
}

pub struct DictionaryData {
    pub char_def: Vec<u8>,
    pub matrix: Vec<u8>,
    pub dict_trie: Vec<u8>,
    pub dict_vals_idx: Vec<u8>,
    pub dict_vals: Vec<u8>,
    pub unk: Vec<u8>,
    pub words_idx: Vec<u8>,
    pub words: Vec<u8>,
    pub metadata: Vec<u8>,
}

static SEGMENTER: OnceLock<lindera::segmenter::Segmenter> = OnceLock::new();

/// Read-only access to the segmenter (debug tooling / probes).
pub fn segmenter_pub() -> Option<&'static lindera::segmenter::Segmenter> {
    SEGMENTER.get()
}

pub fn is_dictionary_loaded() -> bool {
    SEGMENTER.get().is_some()
}

pub fn init_dictionary(data: DictionaryData) -> Result<(), OrigaError> {
    let DictionaryData {
        char_def,
        matrix,
        dict_trie,
        dict_vals_idx,
        dict_vals,
        unk,
        words_idx,
        words,
        metadata,
    } = data;

    let metadata =
        lindera_dictionary::dictionary::metadata::Metadata::load(&metadata).map_err(|e| {
            OrigaError::TokenizerError {
                reason: format!("Failed to load metadata: {e}"),
            }
        })?;

    let prefix_dictionary =
        lindera_dictionary::dictionary::prefix_dictionary::PrefixDictionary::load(
            dict_trie,
            dict_vals_idx,
            dict_vals,
            words_idx,
            words,
        )
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to load prefix dictionary: {e}"),
        })?;

    let connection_cost_matrix =
        lindera_dictionary::dictionary::connection_cost_matrix::ConnectionCostMatrix::load(matrix);

    let character_definition =
        lindera_dictionary::dictionary::character_definition::CharacterDefinition::load(&char_def)
            .map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to load character definition: {e}"),
            })?;

    let unknown_dictionary =
        lindera_dictionary::dictionary::unknown_dictionary::UnknownDictionary::load(&unk).map_err(
            |e| OrigaError::TokenizerError {
                reason: format!("Failed to load unknown dictionary: {e}"),
            },
        )?;

    let connection_cost_matrix =
        connection_cost_matrix.map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to load connection cost matrix: {e}"),
        })?;

    let dictionary = lindera_dictionary::dictionary::Dictionary {
        prefix_dictionary: std::sync::Arc::new(prefix_dictionary),
        connection_cost_matrix: std::sync::Arc::new(connection_cost_matrix),
        character_definition: std::sync::Arc::new(character_definition),
        unknown_dictionary: std::sync::Arc::new(unknown_dictionary),
        metadata: std::sync::Arc::new(metadata),
    };

    init_tokenizer_from_dictionary(dictionary)
}

fn init_tokenizer_from_dictionary(
    dictionary: lindera_dictionary::dictionary::Dictionary,
) -> Result<(), OrigaError> {
    // No user dictionary: the extra vocabulary (連体詞 compounds, STT katakana)
    // is baked into the system lexicon at dictionary-build time — see
    // scripts/build_sudachidict.py. A runtime user dict kept a second POS
    // schema in play, which silently broke Token::get for its words.
    let segmenter =
        lindera::segmenter::Segmenter::new(lindera::mode::Mode::Normal, dictionary, None);

    let _ = SEGMENTER.get_or_init(|| segmenter);
    Ok(())
}

pub fn tokenize_text(text: &str) -> Result<Vec<TokenInfo>, OrigaError> {
    let mut result = Vec::new();
    let mut current_segment = String::new();
    let mut is_current_japanese = false;

    for ch in text.chars() {
        let is_japanese = ch.is_japanese();

        if current_segment.is_empty() {
            is_current_japanese = is_japanese;
            current_segment.push(ch);
        } else if is_japanese == is_current_japanese {
            current_segment.push(ch);
        } else {
            flush_segment(&current_segment, is_current_japanese, &mut result)?;
            current_segment = ch.to_string();
            is_current_japanese = is_japanese;
        }
    }

    if !current_segment.is_empty() {
        flush_segment(&current_segment, is_current_japanese, &mut result)?;
    }

    // Post-processing: split ProperNoun tokens that end in productive suffixes
    // (Lindera over-merges noun + suffix into single ProperNoun, losing
    // individual word meanings and translations).
    split_compound_proper_nouns(&mut result);

    Ok(result)
}

fn flush_segment(
    segment: &str,
    is_japanese: bool,
    result: &mut Vec<TokenInfo>,
) -> Result<(), OrigaError> {
    if is_japanese {
        let segmenter = SEGMENTER.get().ok_or(OrigaError::TokenizerError {
            reason: "Dictionary not loaded. Call init_dictionary() first.".to_string(),
        })?;
        let mut tokens =
            segmenter
                .segment(segment.into())
                .map_err(|e| OrigaError::TokenizerError {
                    reason: e.to_string(),
                })?;
        for token in tokens.iter_mut() {
            result.push(token_to_token_info(token));
        }
    } else {
        let is_whitespace = segment.chars().all(|c| c.is_whitespace());
        result.push(non_japanese_token(segment, is_whitespace));
    }
    Ok(())
}

/// Convert a lindera `Token` (SudachiDict schema) into our `TokenInfo`.
///
/// Field mapping — SudachiDict exposes a different schema than UniDic:
/// - `orthographic_base_form` ← `normalized_form` (dictionary/lemma form,
///   e.g. たべます→食べる); falls back to the surface when the field is
///   missing or `*` (unknown words, user-dict tokens).
/// - `phonological_base_form` / `phonological_surface_form` ← `reading`
///   (katakana). Unknown kanji tokens have no reading — keep the
///   kanji-free fallback for them.
/// - `orthographic_surface_form` ← the token surface itself.
fn token_to_token_info(token: &mut lindera::token::Token) -> TokenInfo {
    let surface = token.surface.to_string();

    let normalized_raw = token.get("normalized_form").unwrap_or_default().to_string();
    let orthographic_base_form = if normalized_raw == "*" || normalized_raw.is_empty() {
        surface.clone()
    } else {
        normalized_raw
    };

    let orthographic_surface_form = surface.clone();

    let reading_raw = token.get("reading").unwrap_or_default().to_string();
    let phonological_base_form = if reading_raw != "*" && !reading_raw.is_empty() {
        reading_raw.clone()
    } else {
        reading_fallback_for_surface(&surface)
    };
    let phonological_surface_form = phonological_base_form.clone();

    let pos_sub1 = token
        .get("part_of_speech_subcategory_1")
        .unwrap_or_default()
        .to_string();

    let mut part_of_speech: PartOfSpeech = token
        .get("part_of_speech")
        .unwrap_or_default()
        .parse()
        .unwrap_or(PartOfSpeech::Unspecified);

    if part_of_speech == PartOfSpeech::Noun && pos_sub1 == "固有名詞" {
        part_of_speech = PartOfSpeech::ProperNoun;
    }

    if surface == "*" || surface == "×" || surface == "•" {
        part_of_speech = PartOfSpeech::Symbol;
    }

    // SudachiDict sometimes merges punctuation runs (！？。) into a single
    // 名詞 token. A surface without any kana or kanji cannot be a vocabulary
    // word — force it to Symbol so downstream card analysis drops it.
    if part_of_speech.is_vocabulary_word()
        && !surface
            .chars()
            .any(|c| c.is_hiragana() || c.is_katakana() || c.is_kanji())
    {
        part_of_speech = PartOfSpeech::Symbol;
    }

    TokenInfo {
        orthographic_base_form,
        phonological_base_form,
        orthographic_surface_form,
        phonological_surface_form,
        part_of_speech,
    }
}

/// Reading fallback for tokens where Lindera provides neither
/// ``phonological_base_form`` nor ``reading`` (Unknown-word tokens, e.g.
/// standalone kanji that are not UniDic lemmas on their own — common in
/// character names like 杏璃 or rare kanji like 蕩).
///
/// A phonological form must be kana. For a kana-only surface the surface
/// itself is a legitimate reading (onomatopoeia, interjections). For a surface
/// that carries kanji no kana reading is derivable, so an empty string is
/// returned instead of leaking the kanji surface as the "reading" — that would
/// render furigana as kanji-over-kanji and feed non-kana strings to downstream
/// consumers (furigana annotator, grammar_label hiragana fallback).
fn reading_fallback_for_surface(surface: &str) -> String {
    if surface.chars().any(|c| c.is_kanji()) {
        String::new()
    } else {
        surface.to_string()
    }
}

fn non_japanese_token(segment: &str, is_whitespace: bool) -> TokenInfo {
    let pos = if is_whitespace {
        PartOfSpeech::Whitespace
    } else {
        PartOfSpeech::Symbol
    };
    TokenInfo {
        orthographic_base_form: segment.to_string(),
        phonological_base_form: segment.to_string(),
        orthographic_surface_form: segment.to_string(),
        phonological_surface_form: segment.to_string(),
        part_of_speech: pos,
    }
}

/// Suffixes that Lindera sometimes over-merges with preceding nouns into a
/// single ProperNoun token. When found at the end of a ProperNoun, the token
/// is split so the suffix gets its own token. Each entry pairs the suffix
/// surface with the POS the split suffix token should carry: なし is a noun
/// suffix (absence), while くらい/ぐらい are particles (degree/extent).
struct SplittableSuffix {
    surface: &'static str,
    pos: PartOfSpeech,
}

const SPLITTABLE_SUFFIXES: &[SplittableSuffix] = &[
    SplittableSuffix {
        surface: "なし",
        pos: PartOfSpeech::Noun,
    },
    SplittableSuffix {
        surface: "くらい",
        pos: PartOfSpeech::Particle,
    },
    SplittableSuffix {
        surface: "ぐらい",
        pos: PartOfSpeech::Particle,
    },
];

/// Post-processing step: split ProperNoun tokens ending in known productive
/// suffixes (e.g. 水なし → 水 + なし). Lindera sometimes merges noun + suffix
/// into a single ProperNoun, losing the individual word meanings.
///
/// Only splits ProperNoun tokens (not regular Nouns) to minimise false
/// positives. The prefix must contain at least one kanji to avoid splitting
/// katakana names or hiragana fragments.
fn split_compound_proper_nouns(tokens: &mut Vec<TokenInfo>) {
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i].part_of_speech() != &PartOfSpeech::ProperNoun {
            i += 1;
            continue;
        }

        let surface = tokens[i].orthographic_surface_form();
        match find_splittable_suffix(surface) {
            Some((prefix, suffix, suffix_pos)) => {
                let split = split_at_suffix(&tokens[i], &prefix, suffix, suffix_pos);
                tokens.splice(i..=i, split);
                i += 2;
            },
            None => i += 1,
        }
    }
}

/// Splits a merged ProperNoun token into prefix + suffix Noun tokens, deriving
/// each token's reading from the merged token's phonological form. Lindera
/// emits the merged token's reading as katakana; the suffix's hiragana surface
/// (e.g. なし) is converted to katakana (ナシ) and stripped from the merged
/// reading to obtain the prefix reading. This handles rendaku and differing
/// surface/reading lengths automatically. If the merged reading does not end
/// in the suffix's katakana form (unexpected), falls back to using surface
/// forms as readings.
fn split_at_suffix(
    token: &TokenInfo,
    prefix: &str,
    suffix: &str,
    suffix_pos: PartOfSpeech,
) -> [TokenInfo; 2] {
    let merged_reading = token.phonological_surface_form();
    let (prefix_reading, suffix_reading) = split_reading(merged_reading, suffix);

    [
        TokenInfo {
            orthographic_base_form: prefix.to_string(),
            phonological_base_form: prefix_reading.clone(),
            orthographic_surface_form: prefix.to_string(),
            phonological_surface_form: prefix_reading,
            part_of_speech: PartOfSpeech::Noun,
        },
        TokenInfo {
            orthographic_base_form: suffix.to_string(),
            phonological_base_form: suffix_reading.clone(),
            orthographic_surface_form: suffix.to_string(),
            phonological_surface_form: suffix_reading,
            part_of_speech: suffix_pos,
        },
    ]
}

fn split_reading(merged_reading: &str, suffix: &str) -> (String, String) {
    let kata_suffix = hiragana_to_katakana(suffix);
    if let Some(prefix_reading) = merged_reading.strip_suffix(&kata_suffix) {
        return (prefix_reading.to_string(), kata_suffix);
    }
    // Fallback: merged reading does not end in the suffix's katakana form
    // (unexpected — Lindera typically emits katakana readings). Surface suffix
    // is the safest guess for the suffix reading.
    (merged_reading.to_string(), suffix.to_string())
}

fn find_splittable_suffix(surface: &str) -> Option<(String, &'static str, PartOfSpeech)> {
    for suffix in SPLITTABLE_SUFFIXES {
        if let Some(prefix_str) = surface.strip_suffix(suffix.surface) {
            // Prefix must be non-empty AND contain at least one kanji,
            // otherwise we'd split katakana names or hiragana fragments.
            if !prefix_str.is_empty() && prefix_str.chars().any(|c| c.is_kanji()) {
                return Some((prefix_str.to_string(), suffix.surface, suffix.pos.clone()));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ensure_dictionary() {
        if !is_dictionary_loaded() {
            let data = create_test_dictionary_data();
            let _ = init_dictionary(data);
        }
    }

    fn create_test_dictionary_data() -> DictionaryData {
        use flate2::read::DeflateDecoder;
        use std::fs;
        use std::io::Read;

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let dict_dir = std::path::PathBuf::from(manifest_dir)
            .parent()
            .unwrap()
            .join("cdn")
            .join("dictionaries");

        let decompress = |data: Vec<u8>| -> Vec<u8> {
            let mut decoder = DeflateDecoder::new(&data[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).unwrap();
            decompressed
        };

        let read_file = |name: &str| fs::read(dict_dir.join(name)).unwrap();

        DictionaryData {
            char_def: decompress(read_file("char_def.bin")),
            matrix: decompress(read_file("matrix.mtx")),
            dict_trie: decompress(read_file("dict.trie")),
            dict_vals_idx: decompress(read_file("dict.valsidx")),
            dict_vals: decompress(read_file("dict.vals")),
            unk: decompress(read_file("unk.bin")),
            words_idx: decompress(read_file("dict.wordsidx")),
            words: decompress(read_file("dict.words")),
            metadata: read_file("metadata.json"),
        }
    }

    #[test]
    fn should_return_base_form_for_verb() {
        ensure_dictionary();
        let tokens = tokenize_text("食べます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_base_form, "食べる");
        // SudachiDict exposes a single katakana reading (surface pronunciation);
        // the separate base-reading field UniDic had does not exist.
        assert_eq!(tokens[0].phonological_base_form, "タベ");
    }

    #[test]
    fn should_return_base_form_for_noun() {
        ensure_dictionary();
        let tokens = tokenize_text("食べ物").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_base_form, "食べ物");
        assert_eq!(tokens[0].phonological_base_form, "タベモノ");
    }

    #[test]
    fn should_return_base_form_for_adjective() {
        ensure_dictionary();
        let tokens = tokenize_text("美味しい").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_base_form, "美味しい");
        assert_eq!(tokens[0].phonological_base_form, "オイシイ");
    }

    #[test]
    fn should_return_base_form_for_hiragana() {
        ensure_dictionary();
        let tokens = tokenize_text("たべます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_base_form, "食べる");
        assert_eq!(tokens[0].phonological_base_form, "タベ");
    }

    #[test]
    fn should_return_base_form_for_hiragana_2() {
        ensure_dictionary();
        let tokens = tokenize_text("こんや").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_base_form, "今夜");
        assert_eq!(tokens[0].phonological_base_form, "コンヤ");
    }

    #[test]
    fn should_return_surface_form_for_verb() {
        ensure_dictionary();
        let tokens = tokenize_text("食べます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_surface_form, "食べ");
        assert_eq!(tokens[0].phonological_surface_form, "タベ");
    }

    #[test]
    fn should_return_surface_form_for_noun() {
        ensure_dictionary();
        let tokens = tokenize_text("食べ物").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_surface_form, "食べ物");
        assert_eq!(tokens[0].phonological_surface_form, "タベモノ");
    }

    #[test]
    fn should_return_surface_form_for_adjective() {
        ensure_dictionary();
        let tokens = tokenize_text("美味しい").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_surface_form, "美味しい");
        assert_eq!(tokens[0].phonological_surface_form, "オイシイ");
    }

    #[test]
    fn should_return_surface_form_for_hiragana() {
        ensure_dictionary();
        let tokens = tokenize_text("たべます").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].orthographic_surface_form, "たべ");
        assert_eq!(tokens[0].phonological_surface_form, "タベ");
    }

    #[test]
    fn should_clean_katakana_base_form() {
        ensure_dictionary();
        let tokens = tokenize_text("アニメ").unwrap();
        assert_eq!(tokens.len(), 1);
        // UniDic typically returns "アニメ-animation" for lexeme
        // Our modification should have stripped "-animation"
        assert_eq!(tokens[0].orthographic_base_form, "アニメ");
    }

    #[test]
    fn should_ignore_junk_symbols_as_vocabulary() {
        ensure_dictionary();
        // Symbols like * and x should be forced to Symbol POS
        let tokens = tokenize_text("アーニャ*ダミアン×ヨル").unwrap();
        for token in tokens {
            let base = token.orthographic_base_form();
            if base == "*" || base == "×" {
                assert!(
                    !token.part_of_speech().is_vocabulary_word(),
                    "Symbol '{}' (POS {:?}) should not be a vocabulary word",
                    base,
                    token.part_of_speech()
                );
                assert_eq!(token.part_of_speech(), &PartOfSpeech::Symbol);
            }
        }
    }

    #[test]
    fn should_preserve_kanji_for_proper_nouns() {
        ensure_dictionary();
        let tokens = tokenize_text("名古屋 横浜").unwrap();
        assert_eq!(tokens[0].orthographic_base_form, "名古屋");
        assert_eq!(tokens[1].part_of_speech(), &PartOfSpeech::Whitespace);
        assert_eq!(tokens[2].orthographic_base_form, "横浜");
    }

    #[test]
    fn should_detect_proper_noun() {
        ensure_dictionary();
        let tokens = tokenize_text("名古屋").unwrap();
        assert_eq!(tokens[0].part_of_speech(), &PartOfSpeech::ProperNoun);
        assert!(tokens[0].part_of_speech().is_vocabulary_word());
    }

    #[test]
    fn should_detect_common_noun_as_noun_not_proper() {
        ensure_dictionary();
        let tokens = tokenize_text("食べ物").unwrap();
        assert_eq!(tokens[0].part_of_speech(), &PartOfSpeech::Noun);
    }

    #[test]
    fn should_preserve_digits_in_tokenization() {
        ensure_dictionary();
        let tokens = tokenize_text("第3課").unwrap();
        let digit_token = tokens.iter().find(|t| t.orthographic_surface_form() == "3");
        assert!(
            digit_token.is_some(),
            "Expected digit token '3', got: {:?}",
            tokens
                .iter()
                .map(|t| t.orthographic_surface_form())
                .collect::<Vec<_>>()
        );
        assert_eq!(digit_token.unwrap().part_of_speech(), &PartOfSpeech::Symbol);
    }

    #[test]
    fn should_preserve_non_japanese_as_symbol() {
        ensure_dictionary();
        let tokens = tokenize_text("hello食べ物world").unwrap();
        let hello_token = tokens
            .iter()
            .find(|t| t.orthographic_surface_form() == "hello");
        assert!(hello_token.is_some());
        assert_eq!(hello_token.unwrap().part_of_speech(), &PartOfSpeech::Symbol);
        let world_token = tokens
            .iter()
            .find(|t| t.orthographic_surface_form() == "world");
        assert!(world_token.is_some());
        assert_eq!(world_token.unwrap().part_of_speech(), &PartOfSpeech::Symbol);
    }

    #[test]
    fn should_handle_only_non_japanese() {
        ensure_dictionary();
        let tokens = tokenize_text("123").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].orthographic_surface_form(), "123");
        assert_eq!(tokens[0].part_of_speech(), &PartOfSpeech::Symbol);
    }

    #[test]
    fn should_handle_empty_text() {
        ensure_dictionary();
        let tokens = tokenize_text("").unwrap();
        assert!(tokens.is_empty());
    }

    #[test]
    fn should_handle_whitespace_as_whitespace_pos() {
        ensure_dictionary();
        let tokens = tokenize_text("   ").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].part_of_speech(), &PartOfSpeech::Whitespace);
    }

    #[test]
    fn should_not_split_souiu_into_sou_and_iu() {
        ensure_dictionary();
        let tokens = tokenize_text("そういう").unwrap();
        assert!(
            tokens
                .iter()
                .any(|t| t.orthographic_surface_form() == "そういう"),
            "「そういう」should be a single token, got: {:?}",
            tokens
                .iter()
                .map(|t| t.orthographic_surface_form())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn should_not_split_kouiu_into_kou_and_iu() {
        ensure_dictionary();
        let tokens = tokenize_text("こういう").unwrap();
        assert!(
            tokens
                .iter()
                .any(|t| t.orthographic_surface_form() == "こういう"),
            "「こういう」should be a single token, got: {:?}",
            tokens
                .iter()
                .map(|t| t.orthographic_surface_form())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn should_not_split_douiu_into_dou_and_iu() {
        ensure_dictionary();
        let tokens = tokenize_text("どういう").unwrap();
        assert!(
            tokens
                .iter()
                .any(|t| t.orthographic_surface_form() == "どういう"),
            "「どういう」should be a single token, got: {:?}",
            tokens
                .iter()
                .map(|t| t.orthographic_surface_form())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn should_not_translate_iu_as_govoriit_in_souiu_context() {
        ensure_dictionary();
        let tokens = tokenize_text("そういう").unwrap();
        let iu_token = tokens
            .iter()
            .find(|t| t.orthographic_surface_form() == "いう");
        if let Some(t) = iu_token {
            assert_ne!(
                t.orthographic_base_form(),
                "言う",
                "「いう」in 「そういう」context should NOT have base_form 言う"
            );
        }
        let has_souiu = tokens
            .iter()
            .any(|t| t.orthographic_surface_form() == "そういう");
        assert!(has_souiu, "「そういう」should be a single token");
    }

    #[test]
    fn should_not_split_to_iu_into_to_and_iu() {
        ensure_dictionary();
        let tokens = tokenize_text("という").unwrap();
        assert!(
            tokens
                .iter()
                .any(|t| t.orthographic_surface_form() == "という"),
            "「という」should be a single token, got: {:?}",
            tokens
                .iter()
                .map(|t| t.orthographic_surface_form())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn should_split_mizunashi_proper_noun() {
        let mut tokens = vec![
            TokenInfo::new_test_with_reading("水なし", "ミズナシ", PartOfSpeech::ProperNoun),
            TokenInfo::new_test("で", PartOfSpeech::Particle),
        ];
        split_compound_proper_nouns(&mut tokens);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].orthographic_surface_form(), "水");
        assert_eq!(tokens[0].phonological_surface_form(), "ミズ");
        assert_eq!(tokens[0].part_of_speech(), &PartOfSpeech::Noun);
        assert_eq!(tokens[1].orthographic_surface_form(), "なし");
        assert_eq!(tokens[1].phonological_surface_form(), "ナシ");
        assert_eq!(tokens[1].part_of_speech(), &PartOfSpeech::Noun);
        // Trailing token must survive the splice unchanged.
        assert_eq!(tokens[2].orthographic_surface_form(), "で");
        assert_eq!(tokens[2].part_of_speech(), &PartOfSpeech::Particle);
    }

    // 努力くらい is over-merged by Lindera into a single ProperNoun (ドリョクライ),
    // swallowing the くらい particle. The splitter must split it into 努力
    // (Noun) + くらい (Particle) so the particle carries its own token. Unlike
    // なし, the くらい suffix is a particle, so its split token's POS is derived
    // from the SplittableSuffix mapping rather than hardcoded Noun.
    #[test]
    fn should_split_doryokukurai_proper_noun() {
        let mut tokens = vec![
            TokenInfo::new_test_with_reading(
                "努力くらい",
                "ドリョククライ",
                PartOfSpeech::ProperNoun,
            ),
            TokenInfo::new_test("は", PartOfSpeech::Particle),
        ];
        split_compound_proper_nouns(&mut tokens);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].orthographic_surface_form(), "努力");
        assert_eq!(tokens[0].phonological_surface_form(), "ドリョク");
        assert_eq!(tokens[0].part_of_speech(), &PartOfSpeech::Noun);
        assert_eq!(tokens[1].orthographic_surface_form(), "くらい");
        assert_eq!(tokens[1].phonological_surface_form(), "クライ");
        assert_eq!(tokens[1].part_of_speech(), &PartOfSpeech::Particle);
        // Trailing token must survive the splice unchanged.
        assert_eq!(tokens[2].orthographic_surface_form(), "は");
        assert_eq!(tokens[2].part_of_speech(), &PartOfSpeech::Particle);
    }

    #[test]
    fn should_not_split_regular_noun_ending_in_nashi() {
        let mut tokens = vec![TokenInfo::new_test("某なし", PartOfSpeech::Noun)];
        split_compound_proper_nouns(&mut tokens);
        assert_eq!(tokens.len(), 1);
    }

    #[test]
    fn should_not_split_proper_noun_without_kanji_prefix() {
        let mut tokens = vec![TokenInfo::new_test("アなし", PartOfSpeech::ProperNoun)];
        split_compound_proper_nouns(&mut tokens);
        assert_eq!(tokens.len(), 1);
    }

    #[test]
    fn should_not_split_proper_noun_without_known_suffix() {
        let mut tokens = vec![TokenInfo::new_test("田中", PartOfSpeech::ProperNoun)];
        split_compound_proper_nouns(&mut tokens);
        assert_eq!(tokens.len(), 1);
    }

    #[test]
    fn should_not_use_kanji_surface_as_phonological_reading_for_unknown_kanji() {
        ensure_dictionary();

        // SudachiDict keeps the character name 杏璃 whole. Force an unknown
        // kanji path through a made-up name to keep the contract covered.
        let tokens = tokenize_text("杏璃蕗").unwrap();
        for t in &tokens {
            assert!(
                !t.phonological_base_form().chars().any(|c| c.is_kanji()),
                "phonological_base_form must not contain kanji, got '{}'",
                t.phonological_base_form()
            );
            assert!(
                !t.phonological_surface_form().chars().any(|c| c.is_kanji()),
                "phonological_surface_form must not contain kanji, got '{}'",
                t.phonological_surface_form()
            );
        }
    }

    // RED repro: common word 自動車 ("car") is split by the bundled UniDic
    // subset into 自動 (Noun) + 車 (Suffix) / katakana ジドウ (Noun) + シャ
    // (Suffix). The Suffix tail is not a vocabulary word, so downstream card
    // analysis keeps only the leading token — the real word never reaches the
    // user. Dictionary bundle lacks the 自動車 lemma entirely (no occurrence in
    // dict.words), so Viterbi cuts at known-word boundaries.
    #[test]
    fn should_keep_jidousha_as_single_vocabulary_token() {
        ensure_dictionary();

        for surface in ["自動車", "飛行機", "図書館", "自転車", "令和", "お父さん"]
        {
            let tokens = tokenize_text(surface).unwrap();
            let vocab: Vec<&TokenInfo> = tokens
                .iter()
                .filter(|t| t.part_of_speech().is_vocabulary_word())
                .collect();
            let vocab_surfaces: Vec<&str> = vocab
                .iter()
                .map(|t| t.orthographic_surface_form())
                .collect();
            assert_eq!(
                vocab.len(),
                1,
                "「{surface}」 must survive as a single vocabulary token, got vocab {vocab_surfaces:?} from {:?}",
                tokens
                    .iter()
                    .map(|t| {
                        format!(
                            "{}({:?})",
                            t.orthographic_surface_form(),
                            t.part_of_speech()
                        )
                    })
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                vocab[0].orthographic_surface_form(),
                surface,
                "the vocabulary token must cover the whole word"
            );
        }
    }
}
