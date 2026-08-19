//! Rebuild well-known sets from original sources.
//!
//! Sources:
//! - JLPT N1–N5: `<sources>/JLPT_vocab_ALL.json` — {word: [{reading, level}]}
//! - Minna N2/N3: `<sources>/minnan2.json`, `<sources>/minnan3.json5` —
//!   [{lesson, words}]
//! - Minna N4/N5: extracted from git history by the caller into
//!   `<sources>/minna_n4/*.json`, `<sources>/minna_n5/*.json` (original
//!   set-file format with kana-first words).
//!
//! For every source word the canonical form is resolved as:
//! 1. `base = tokenize(word)` — the SudachiDict base lemma (first vocabulary
//!    token), e.g. あなた → 貴方, この → 此の;
//! 2. if `base` has a translation entry → use `base`;
//! 3. else if the source word itself has a translation entry → keep it;
//! 4. else keep the source word and report it as missing.
//!
//! Output rewrites `cdn/well_known_set/...` set files in place (words only;
//! content/level metadata preserved) and prints a per-set report.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use origa::domain::tokenize_text;
use utils::dictionary::load_dictionary;

fn load_translation_keys(cdn: &Path) -> HashSet<String> {
    let mut keys = HashSet::new();
    let dir = cdn.join("dictionary");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        panic!("vocabulary chunks not found: {}", dir.display());
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let Ok(raw) = std::fs::read_to_string(&p) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
            continue;
        };
        if let Some(obj) = v.as_object() {
            for k in obj.keys() {
                keys.insert(k.clone());
            }
        }
    }
    keys
}

fn has_translation(word: &str, dict_keys: &HashSet<String>) -> bool {
    dict_keys.contains(word)
}

/// Base form when the source word tokenizes into a single vocabulary word.
///
/// Multi-token results (きょうし → こう|し, かいしゃいん → 会社|名) mean the
/// tokenizer does not know the word as a unit — taking the first token's
/// lemma then produces garbage (供する, 会社名). In that case we fall back to
/// the source form itself.
fn base_form(word: &str) -> Option<String> {
    let tokens = tokenize_text(word).ok()?;
    if tokens.len() != 1 {
        return None;
    }
    let t = &tokens[0];
    if !t.part_of_speech().is_vocabulary_word() {
        return None;
    }
    Some(t.orthographic_base_form().to_string())
}

fn canonical(word: &str, dict_keys: &HashSet<String>, missing: &mut Vec<String>) -> String {
    if let Some(base) = base_form(word)
        && base != word
        && has_translation(&base, dict_keys)
    {
        return base;
    }
    if has_translation(word, dict_keys) {
        return word.to_string();
    }
    missing.push(word.to_string());
    word.to_string()
}

/// Source lists carry pedagogical noise the original pipeline used to drop:
/// 〜-suffixed grammar markers (〜ごと), bracketed optional forms ([お] 国),
/// romaji items (ATM), full dialogue phrases. None of these are card words.
fn is_noise(word: &str) -> bool {
    let w = word.trim();
    if w.is_empty() {
        return true;
    }
    let has_jp = w.chars().any(|c| {
        let cp = c as u32;
        (0x3040..=0x30FF).contains(&cp) || (0x4E00..=0x9FFF).contains(&cp)
    });
    if !has_jp {
        return true;
    }
    if w.chars().any(|c| "[]()×〜～".contains(c)) {
        return true;
    }
    // dialogue phrases / multi-word items
    if w.chars().any(|c| c == ' ' || c == '　') {
        return true;
    }
    // long sentence-like items
    if w.chars().count() > 15 {
        return true;
    }
    // conjugated / polite verb forms and lesson phrases are grammar, not
    // card words — the base verb belongs to its own lesson entry anyway
    // (紹介します → 紹介). Also covers 〜て/〜た/〜ない tails on verbs.
    if w.chars().any(|c| "。？!！、".contains(c)) {
        return true;
    }
    for tail in [
        "ます",
        "ました",
        "ません",
        "ましょう",
        "てください",
        "ないです",
        "ましたか",
    ] {
        if w.ends_with(tail) && w.chars().count() > tail.chars().count() {
            return true;
        }
    }
    false
}

fn dedup_keep_order(words: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    words
        .into_iter()
        .filter(|w| seen.insert(w.clone()))
        .collect()
}

fn load_json(path: &Path) -> serde_json::Value {
    let raw = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    // tolerate JSON5 comments in .json5 sources
    let cleaned: String = raw
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    serde_json::from_str(&cleaned).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn rewrite_set_file(path: &Path, new_words: &[String]) {
    let mut v = load_json(path);
    v["words"] = serde_json::Value::Array(
        new_words
            .iter()
            .map(|w| serde_json::Value::String(w.clone()))
            .collect(),
    );
    std::fs::write(path, serde_json::to_string_pretty(&v).unwrap()).unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let sources = PathBuf::from(
        args.get(1)
            .expect("usage: rebuild_sets_from_sources <sources-dir> <cdn-dir>"),
    );
    let cdn = PathBuf::from(
        args.get(2)
            .expect("usage: rebuild_sets_from_sources <sources-dir> <cdn-dir>"),
    );

    load_dictionary().expect("dictionary");
    let dict_keys = load_translation_keys(&cdn);
    println!("словарь переводов: {} ключей", dict_keys.len());
    let wk = cdn.join("well_known_set");

    let mut total_missing: Vec<String> = Vec::new();
    let mut total_sets = 0usize;
    let mut total_words = 0usize;

    // ---- JLPT ----
    let jlpt = load_json(&sources.join("JLPT_vocab_ALL.json"));
    if let Some(map) = jlpt.as_object() {
        let mut by_level: std::collections::BTreeMap<u8, Vec<String>> = Default::default();
        let mut missing: Vec<String> = Vec::new();
        for (word, entries) in map {
            for e in entries.as_array().into_iter().flatten() {
                let lvl = e["level"].as_u64().unwrap_or(0) as u8;
                if (1..=5).contains(&lvl) && !is_noise(word) {
                    by_level.entry(lvl).or_default().push(canonical(
                        word,
                        &dict_keys,
                        &mut missing,
                    ));
                }
            }
        }
        for (lvl, words) in by_level {
            let words = dedup_keep_order(words);
            let path = wk.join(format!("jlpt_n{lvl}.json"));
            total_sets += 1;
            total_words += words.len();
            println!("jlpt_n{lvl}: {} слов", words.len());
            rewrite_set_file(&path, &words);
        }
        total_missing.extend(missing);
    }

    // ---- Minna N2/N3 (lesson lists) ----
    for (file, level) in [("minnan2.json", "N2"), ("minnan3.json5", "N3")] {
        let src = load_json(&sources.join(file));
        if let Some(lessons) = src.as_array() {
            for lesson in lessons {
                let no = match lesson["lesson"].as_u64() {
                    Some(n) => format!("{n:02}"),
                    None => lesson["lesson"].as_str().unwrap_or("extra").to_string(),
                };
                let mut missing: Vec<String> = Vec::new();
                let words: Vec<String> = lesson["words"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|w| w.as_str())
                    .filter(|w| !w.is_empty() && !is_noise(w))
                    .map(|w| canonical(w, &dict_keys, &mut missing))
                    .collect();
                let words = dedup_keep_order(words);
                let lvl_lower = level.to_lowercase();
                let path = wk.join(format!("minna_{lvl_lower}/minna_{lvl_lower}_{no}.json"));
                total_sets += 1;
                total_words += words.len();
                println!("minna_{}_{no}: {} слов", lvl_lower, words.len());
                rewrite_set_file(&path, &words);
                total_missing.extend(missing);
            }
        }
    }

    // ---- Minna N4/N5 (extracted set files from git) ----
    for dir in ["minna_n5", "minna_n4"] {
        let src_dir = sources.join(dir);
        let Ok(entries) = std::fs::read_dir(&src_dir) else {
            println!("(пропуск {dir}: нет {}/{dir})", sources.display());
            continue;
        };
        let mut files: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
            .collect();
        files.sort();
        for f in files {
            let v = load_json(&f);
            let mut missing: Vec<String> = Vec::new();
            let words: Vec<String> = v["words"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|w| w.as_str())
                .filter(|w| !w.is_empty() && !is_noise(w))
                .map(|w| canonical(w, &dict_keys, &mut missing))
                .collect();
            let words = dedup_keep_order(words);
            let name = f.file_name().unwrap().to_str().unwrap().to_string();
            let path = wk.join(dir).join(&name);
            total_sets += 1;
            total_words += words.len();
            println!("{dir}/{name}: {} слов", words.len());
            rewrite_set_file(&path, &words);
            total_missing.extend(missing);
        }
    }

    println!(
        "\nИТОГО: сетов {total_sets}, слов {total_words}, без перевода: {}",
        total_missing.len()
    );
    let uniq: HashSet<String> = total_missing.into_iter().collect();
    let mut sorted: Vec<String> = uniq.into_iter().collect();
    sorted.sort();
    let report = sources.join("missing_after_rebuild.txt");
    std::fs::write(&report, sorted.join("\n")).unwrap();
    println!("отчёт: {}", report.display());
}
