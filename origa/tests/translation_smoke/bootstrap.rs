//! Process-wide dictionary bootstrap shared by every test in this binary.
//!
//! Mirrors the Once-based race-safe pattern used by `grammar_regression_checks`
//! and the in-crate `integration_tests`: parallel test threads share the
//! `GRAMMAR_RULES` / `VOCABULARY_DICTIONARY` / `TOKENIZER` `OnceLock`s, so the
//! first load must run exactly once per binary via `Once::call_once`.

use std::io::Read;
use std::path::PathBuf;
use std::sync::Once;

use flate2::read::DeflateDecoder;
use origa::dictionary::grammar::{GrammarData, init_grammar, is_grammar_loaded};
use origa::dictionary::vocabulary::{VocabularyChunkData, init_vocabulary, is_vocabulary_loaded};
use origa::domain::{DictionaryData, init_dictionary, is_dictionary_loaded};

static BOOTSTRAP: Once = Once::new();

pub fn cdn_dir() -> PathBuf {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by cargo");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("workspace root is parent of the origa crate manifest")
        .join("cdn")
}

pub fn cdn_path(components: &[&str]) -> PathBuf {
    let mut path = cdn_dir();
    for component in components {
        path.push(component);
    }
    path
}

/// Loads tokenizer / vocabulary / grammar dictionaries exactly once.
///
/// Returns ``true`` when all three stores end up loaded, ``false`` when any of
/// the gitignored ``cdn/`` artifacts is absent so callers can decide whether
/// to skip the test gracefully. A malformed store still panics — corrupt data
/// is a real error, not a missing-environment condition.
pub fn ensure_all_dictionaries() -> bool {
    if is_dictionary_loaded() && is_vocabulary_loaded() && is_grammar_loaded() {
        return true;
    }

    let mut loaded = true;
    BOOTSTRAP.call_once(|| {
        loaded = try_load_tokenizer_dictionary()
            && try_load_vocabulary_dictionary()
            && try_load_grammar_dictionary();
    });
    loaded && is_dictionary_loaded() && is_vocabulary_loaded() && is_grammar_loaded()
}

fn try_load_tokenizer_dictionary() -> bool {
    if is_dictionary_loaded() {
        return true;
    }

    let dict_dir = cdn_path(&["dictionaries"]);
    let exists = |name: &str| dict_dir.join(name).exists();
    let required = [
        "char_def.bin",
        "matrix.mtx",
        "dict.trie",
        "dict.valsidx",
        "dict.vals",
        "unk.bin",
        "dict.wordsidx",
        "dict.words",
        "metadata.json",
    ];
    if !required.iter().all(|name| exists(name)) {
        return false;
    }

    let read_file = |name: &str| {
        std::fs::read(dict_dir.join(name)).unwrap_or_else(|e| panic!("{name} read: {e}"))
    };
    let decompress = |data: Vec<u8>| -> Vec<u8> {
        let mut decoder = DeflateDecoder::new(&data[..]);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .unwrap_or_else(|e| panic!("dictionary decompress: {e}"));
        decompressed
    };

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

    init_dictionary(data).expect("init_dictionary must succeed on valid data");
    true
}

fn try_load_vocabulary_dictionary() -> bool {
    if is_vocabulary_loaded() {
        return true;
    }

    let vocab_dir = cdn_path(&["dictionary"]);
    let mut chunks = Vec::with_capacity(11);
    for n in 1..=11u8 {
        let name = format!("chunk_{n:02}.json");
        match std::fs::read_to_string(vocab_dir.join(name)) {
            Ok(body) => chunks.push(body),
            Err(_) => return false,
        }
    }

    let mut iter = chunks.into_iter();
    let vocab_data = VocabularyChunkData {
        chunk_01: iter.next().expect("11 chunks"),
        chunk_02: iter.next().expect("11 chunks"),
        chunk_03: iter.next().expect("11 chunks"),
        chunk_04: iter.next().expect("11 chunks"),
        chunk_05: iter.next().expect("11 chunks"),
        chunk_06: iter.next().expect("11 chunks"),
        chunk_07: iter.next().expect("11 chunks"),
        chunk_08: iter.next().expect("11 chunks"),
        chunk_09: iter.next().expect("11 chunks"),
        chunk_10: iter.next().expect("11 chunks"),
        chunk_11: iter.next().expect("11 chunks"),
    };

    // Tolerate the OnceLock init race: a concurrent test thread may have
    // initialized the vocabulary first with identical data. Only a genuinely
    // unset dictionary indicates a real failure.
    if let Err(err) = init_vocabulary(vocab_data) {
        if !is_vocabulary_loaded() {
            panic!("vocabulary dictionary failed to load: {err:?}");
        }
    }
    true
}

fn try_load_grammar_dictionary() -> bool {
    if is_grammar_loaded() {
        return true;
    }

    let grammar_path = cdn_path(&["grammar", "grammar.json"]);
    let grammar_json = match std::fs::read_to_string(&grammar_path) {
        Ok(body) => body,
        Err(_) => return false,
    };

    if let Err(err) = init_grammar(GrammarData { grammar_json }) {
        if !is_grammar_loaded() {
            panic!("grammar dictionary failed to load: {err:?}");
        }
    }
    true
}
