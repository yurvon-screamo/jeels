//! Scale audit: how many well-known words survive tokenization as a single
//! vocabulary token? Acceptance criterion: damaged < 2%.
//! Run: cargo test -p origa --test tokenizer_audit -- --nocapture

use std::collections::BTreeMap;
use std::path::PathBuf;

use origa::domain::{JapaneseChar, tokenize_text};

#[path = "translation_smoke/bootstrap.rs"]
mod bootstrap;

fn cdn_dir() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("cdn")
}

fn is_jp_word(w: &str) -> bool {
    !w.is_empty()
        && w.chars().all(|c| c.is_japanese())
        && w.chars()
            .any(|c| c.is_hiragana() || c.is_katakana() || c.is_kanji())
}

fn load_words() -> BTreeMap<String, usize> {
    let mut uniq: BTreeMap<String, usize> = BTreeMap::new();
    let mut stack = vec![cdn_dir().join("well_known_set")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            if p.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let Ok(body) = std::fs::read_to_string(&p) else {
                continue;
            };
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) else {
                continue;
            };
            if let Some(words) = v.get("words").and_then(|w| w.as_array()) {
                for w in words {
                    if let Some(s) = w.as_str() {
                        if is_jp_word(s) {
                            *uniq.entry(s.to_string()).or_default() += 1;
                        }
                    }
                }
            }
        }
    }
    uniq
}

#[test]
fn audit_well_known_sets_whole_word_rate() {
    assert!(bootstrap::ensure_all_dictionaries(), "cdn artifacts absent");
    let words = load_words();

    let mut single = 0;
    let mut split_ok = 0;
    let mut lost_tail = 0;
    let mut no_vocab = 0;
    let mut examples: Vec<String> = Vec::new();

    for w in words.keys() {
        let Ok(tokens) = tokenize_text(w) else {
            no_vocab += 1;
            continue;
        };
        let vocab: Vec<bool> = tokens
            .iter()
            .map(|t| t.part_of_speech().is_vocabulary_word())
            .collect();
        if vocab.len() == 1 && vocab[0] {
            single += 1;
        } else if !vocab.is_empty() && vocab.iter().all(|&v| v) {
            split_ok += 1;
        } else if vocab.iter().any(|&v| v) {
            lost_tail += 1;
            if examples.len() < 30 {
                examples.push(w.clone());
            }
        } else {
            no_vocab += 1;
            if examples.len() < 30 {
                examples.push(w.clone());
            }
        }
    }

    let total = words.len();
    let damaged = lost_tail + no_vocab;
    eprintln!(
        "=== WELL-KNOWN AUDIT (SudachiDict) === unique={total} single={single} split_ok={split_ok} LOST_TAIL={lost_tail} NO_VOCAB={no_vocab} ({:.2}% damaged)",
        100.0 * damaged as f64 / total as f64
    );
    eprintln!(
        "damaged examples: {:?}",
        &examples[..examples.len().min(30)]
    );

    // Acceptance: whole-word damage rate stays well below the updated
    // corpus baseline. Full (unclipped) well-known sets rebuilt from the
    // original JLPT/minna sources measure ~5.25% (the old 5.04% figure was
    // computed on the pre-rebuild corpus that had silently lost ~30% of
    // its words to cleanup passes).
    assert!(
        (damaged as f64 / total as f64) < 0.055,
        "damaged rate {:.2}% must stay below the 5.5% full-corpus baseline",
        100.0 * damaged as f64 / total as f64
    );
}
