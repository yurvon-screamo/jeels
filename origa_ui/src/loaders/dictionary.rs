use origa::domain::{DictionaryData, OrigaError, init_dictionary, is_dictionary_loaded};

use crate::repository::{
    cdn_provider, get_cached_dictionary_files, save_dictionary_files_to_cache,
};
use crate::utils::{now_ms, yield_to_browser};
use origa::traits::CdnProvider;

/// The eight raw lindera dictionary files (deflate-compressed on the CDN,
/// inflated on device). lindera 5.x walks the trie in place, so no pre-built
/// rkyv blob is needed anymore — the raw files ARE the cache.
const DICTIONARY_FILES: &[(&str, bool)] = &[
    ("char_def.bin", true),
    ("matrix.mtx", true),
    ("dict.trie", true),
    ("dict.valsidx", true),
    ("dict.vals", true),
    ("unk.bin", true),
    ("dict.wordsidx", true),
    ("dict.words", true),
];
const METADATA_FILE: &str = "metadata.json";

/// Load the SudachiDict tokenizer dictionary.
///
/// Unified path for cache-hit and cache-miss: fetch eight deflate-compressed
/// files (from Cache API or CDN), inflate, hand the raw bytes to lindera 5.x
/// loaders. Peak WASM heap is bounded by compressed+inflated bytes coexisting
/// during inflate (~420 MB on first load, ~350 MB steady state).
pub async fn load_dictionary() -> Result<(), OrigaError> {
    if is_dictionary_loaded() {
        tracing::debug!("📖 Dictionary already loaded");
        return Ok(());
    }
    let start = now_ms();
    tracing::info!("📖 Loading tokenizer dictionary...");

    let mut cached: Option<Vec<(String, Vec<u8>)>> = get_cached_dictionary_files().await;

    // Cache-miss (or partial cache): fetch deflated files from the CDN.
    if cached.is_none() {
        let fetched = fetch_deflated_files().await?;
        // Best-effort cache save — failure here is not fatal.
        if let Err(e) = save_dictionary_files_to_cache(&fetched).await {
            tracing::warn!("Failed to cache dictionary files: {e:?}");
        }
        cached = Some(fetched);
    }
    let compressed = cached.expect("just fetched");

    yield_to_browser().await;

    let data = inflate_dictionary_data(compressed).await?;
    init_dictionary(data)?;

    tracing::info!("📖 Dictionary loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}

/// Fetch all dictionary files (deflated) from the CDN in parallel.
async fn fetch_deflated_files() -> Result<Vec<(String, Vec<u8>)>, OrigaError> {
    let provider = cdn_provider();
    let mut names: Vec<String> = DICTIONARY_FILES
        .iter()
        .map(|(n, _)| format!("dictionaries/{n}"))
        .collect();
    names.push(format!("dictionaries/{METADATA_FILE}"));

    let mut results: Vec<(String, Vec<u8>)> = Vec::with_capacity(names.len());
    // Sequential fetch keeps peak memory at one file at a time; the biggest
    // (dict.words, 28 MB deflated) dominates.
    for path in names {
        let bytes = provider.fetch_bytes(&path).await?;
        tracing::debug!("📖 Fetched {path} ({} bytes)", bytes.len());
        results.push((path, bytes));
        yield_to_browser().await;
    }
    Ok(results)
}

/// Inflate the deflated dictionary files into `DictionaryData`.
async fn inflate_dictionary_data(
    compressed: Vec<(String, Vec<u8>)>,
) -> Result<DictionaryData, OrigaError> {
    let mut files: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();
    for (path, bytes) in compressed {
        let name = path.rsplit('/').next().unwrap_or(&path).to_string();
        let inflated = if DICTIONARY_FILES.iter().any(|(n, _)| *n == name) {
            inflate(&bytes)?
        } else {
            // metadata.json is stored uncompressed
            bytes
        };
        files.insert(name, inflated);
    }
    yield_to_browser().await;

    let mut get = |name: &str| -> Result<Vec<u8>, OrigaError> {
        files
            .remove(name)
            .ok_or_else(|| OrigaError::TokenizerError {
                reason: format!("dictionary file missing: {name}"),
            })
    };

    Ok(DictionaryData {
        char_def: get("char_def.bin")?,
        matrix: get("matrix.mtx")?,
        dict_trie: get("dict.trie")?,
        dict_vals_idx: get("dict.valsidx")?,
        dict_vals: get("dict.vals")?,
        unk: get("unk.bin")?,
        words_idx: get("dict.wordsidx")?,
        words: get("dict.words")?,
        metadata: get("metadata.json")?,
    })
}

/// Raw-deflate decompression (no zlib header), same scheme the CDN deploy
/// produces.
fn inflate(data: &[u8]) -> Result<Vec<u8>, OrigaError> {
    use std::io::Read;
    let mut decoder = flate2::read::DeflateDecoder::new(data);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("failed to inflate dictionary file: {e}"),
        })?;
    Ok(out)
}
