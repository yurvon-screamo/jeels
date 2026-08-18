use crate::dictionary::load_dictionary;
use origa::domain::{OrigaError, tokenize_text};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Re-tokenizes every phrase in phrase_index.json with the CURRENT tokenizer.
///
/// The index carries only token arrays (`t`) — the phrase text (`x`) lives in
/// the data bundles — so this loads every data_bundle_*.json first to build an
/// id -> text map, then rewrites `t` in place. The `i`, `c`, `g` fields are
/// preserved byte-for-byte; `h` (content hash) and `v` are left untouched —
/// bump them at the deploy step if needed.
pub fn run_retokenize_phrases(index_path: PathBuf, data_dir: PathBuf) -> Result<(), OrigaError> {
    load_dictionary()?;

    // 1) id -> text from data bundles
    let mut id_to_text: HashMap<String, String> = HashMap::new();
    let mut bundles = vec![];
    for entry in fs::read_dir(&data_dir).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read {}: {e}", data_dir.display()),
    })? {
        let p = entry
            .map_err(|e| OrigaError::TokenizerError {
                reason: format!("readdir: {e}"),
            })?
            .path();
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with("data_bundle_") && name.ends_with(".json") {
            bundles.push(p.clone());
        }
    }
    bundles.sort();
    for b in &bundles {
        let content = fs::read_to_string(b).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to read {}: {e}", b.display()),
        })?;
        let v: Value = serde_json::from_str(&content).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to parse {}: {e}", b.display()),
        })?;
        // bundle = { "p0000": [ {id,text,...}, ... ], ... }
        for (_chunk, arr) in v.as_object().into_iter().flatten() {
            for item in arr.as_array().into_iter().flatten() {
                if let (Some(id), Some(text)) = (
                    item.get("i").and_then(|x| x.as_str()),
                    item.get("x").and_then(|x| x.as_str()),
                ) {
                    id_to_text.insert(id.to_string(), text.to_string());
                }
            }
        }
    }
    tracing::info!(
        "Loaded {} phrase texts from {} bundles",
        id_to_text.len(),
        bundles.len()
    );

    // 2) re-tokenize the index
    let content = fs::read_to_string(&index_path).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to read {}: {e}", index_path.display()),
    })?;
    let mut json: Value =
        serde_json::from_str(&content).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to parse {}: {e}", index_path.display()),
        })?;

    let phrases_arr = json
        .get_mut("phrases")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| OrigaError::TokenizerError {
            reason: "phrases array missing".to_string(),
        })?;

    let total = phrases_arr.len();
    tracing::info!("Re-tokenizing {total} phrases");

    let mut changed = 0usize;
    let mut no_text = 0usize;
    let mut old_tokens: HashSet<String> = HashSet::new();
    let mut new_tokens: HashSet<String> = HashSet::new();
    let mut new_to_old_extra: HashMap<String, String> = HashMap::new(); // для отчёта новых составных

    for (idx, entry) in phrases_arr.iter_mut().enumerate() {
        let id = entry
            .get("i")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let Some(id) = id else { continue };
        let Some(text) = id_to_text.get(&id) else {
            no_text += 1;
            continue;
        };

        if let Some(old) = entry.get("t").and_then(|v| v.as_array()) {
            for t in old {
                if let Some(s) = t.as_str() {
                    old_tokens.insert(s.to_string());
                }
            }
        }

        let toks = tokenize_text(text)?;
        let mut vocab: Vec<String> = toks
            .iter()
            .filter(|t| t.part_of_speech().is_vocabulary_word())
            .map(|t| t.orthographic_base_form().to_string())
            .collect();
        vocab.sort();
        vocab.dedup();
        for t in &vocab {
            new_tokens.insert(t.clone());
        }
        // примеры новых составных (для ревью-отчёта)
        for t in &vocab {
            if !old_tokens.contains(t) && new_to_old_extra.len() < 40 {
                new_to_old_extra
                    .entry(t.clone())
                    .or_insert_with(|| id.clone());
            }
        }

        let old_sorted = {
            let mut o: Vec<String> = entry
                .get("t")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            o.sort();
            o.dedup();
            o
        };

        if old_sorted != vocab {
            changed += 1;
            if let Some(obj) = entry.as_object_mut() {
                obj.insert(
                    "t".to_string(),
                    Value::Array(vocab.into_iter().map(Value::String).collect()),
                );
            }
        }
        if (idx + 1) % 20000 == 0 {
            tracing::info!("  {}/{}...", idx + 1, total);
        }
    }

    let gone: Vec<&String> = old_tokens.difference(&new_tokens).collect();
    let added: Vec<&String> = new_tokens.difference(&old_tokens).collect();
    tracing::info!("changed: {changed}/{total} | no-text entries: {no_text}");
    tracing::info!(
        "old unique tokens: {} | new unique tokens: {}",
        old_tokens.len() + added.len() - added.len() + new_tokens.len(),
        new_tokens.len()
    );
    tracing::info!(
        "tokens disappeared: {} | tokens added: {}",
        gone.len(),
        added.len()
    );
    tracing::info!(
        "sample new compound tokens: {:?}",
        new_to_old_extra.keys().take(20).collect::<Vec<_>>()
    );

    let serialized = serde_json::to_string(&json).map_err(|e| OrigaError::TokenizerError {
        reason: format!("serialize failed: {e}"),
    })?;
    fs::write(&index_path, serialized).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to write {}: {e}", index_path.display()),
    })?;
    tracing::info!("Written: {}", index_path.display());
    Ok(())
}

#[allow(dead_code)]
fn unused(_: &Path) {}
