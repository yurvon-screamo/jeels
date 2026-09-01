use origa::domain::{
    DictionaryData, OrigaError, SUDACHIDICT_DIR, init_dictionary, is_dictionary_loaded,
};

use crate::repository::{
    cdn_provider, get_cached_dictionary_files, save_dictionary_files_to_cache,
};
use crate::utils::{now_ms, yield_to_browser};
use origa::traits::CdnProvider;

/// The eight raw lindera dictionary files (deflate-compressed on the CDN,
/// inflated on device). lindera walks the trie in place, so no pre-built
/// rkyv blob is needed — the raw files ARE the cache.
/// Ordered so the largest file (dict.words, 28 MB deflated → 223 MB raw)
/// inflates FIRST, while every other file is still compressed: peak heap
/// stays at ~dict.words-raw + everything-compressed instead of the reverse.
const DICTIONARY_FILES: &[&str] = &[
    "dict.words",
    "matrix.mtx",
    "dict.trie",
    "dict.vals",
    "dict.valsidx",
    "dict.wordsidx",
    "unk.bin",
    "char_def.bin",
];
const METADATA_FILE: &str = "metadata.json";

/// Full CDN path of a file inside the versioned SudachiDict directory
/// (e.g. `dict.words` → `dictionaries/sudachidict-20260723/dict.words`).
pub fn dict_path(file: &str) -> String {
    format!("dictionaries/{SUDACHIDICT_DIR}/{file}")
}

/// Load the SudachiDict tokenizer dictionary.
///
/// Unified path for cache-hit and cache-miss: fetch eight deflate-compressed
/// files (from Cache API or CDN), inflate, hand the raw bytes to the lindera
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
    let mut names: Vec<String> = DICTIONARY_FILES.iter().map(|n| dict_path(n)).collect();
    names.push(dict_path(METADATA_FILE));

    let mut results: Vec<(String, Vec<u8>)> = Vec::with_capacity(names.len());
    // Sequential fetch keeps peak memory at one file at a time.
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
        let inflated = if DICTIONARY_FILES.contains(&name.as_str()) {
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
    // Pre-size the output buffer (~8x the deflated size for the word list)
    // so the 223 MB words buffer never doubles through a realloc spike.
    let mut out = Vec::with_capacity(data.len() * 8);
    decoder
        .read_to_end(&mut out)
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("failed to inflate dictionary file: {e}"),
        })?;
    Ok(out)
}
