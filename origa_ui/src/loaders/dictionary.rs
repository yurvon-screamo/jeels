use std::collections::HashMap;

use origa::domain::{
    DictionaryData, OrigaError, SUDACHIDICT_DIR, init_dictionary, is_dictionary_loaded,
};

use crate::repository::{
    DICTIONARY_FILE_NAMES, cdn_provider, cleanup_legacy_dictionary_cache,
    get_cached_dictionary_files, save_dictionary_file_to_cache,
};
use crate::utils::{now_ms, yield_to_browser};
use origa::traits::CdnProvider;

/// Processing order of the fetch→inflate→persist pipeline: the eight
/// lindera files deflated on the CDN (words first — the largest single
/// inflate, cheapest while every other file is still compressed), then
/// `metadata.json` which is stored uncompressed. lindera walks the trie in
/// place, so no pre-built rkyv blob is needed — the raw files ARE the
/// runtime structure.
const PIPELINE_ORDER: &[&str] = &[
    "dict.words",
    "matrix.mtx",
    "dict.trie",
    "dict.vals",
    "dict.valsidx",
    "dict.wordsidx",
    "unk.bin",
    "char_def.bin",
    "metadata.json",
];
const METADATA_FILE: &str = "metadata.json";

/// Full CDN path of a file inside the versioned SudachiDict directory
/// (e.g. `dict.words` → `dictionaries/sudachidict-20260723/dict.words`).
pub fn dict_path(file: &str) -> String {
    format!("dictionaries/{SUDACHIDICT_DIR}/{file}")
}

/// A cached set of files is usable when every expected file name is
/// present. Callers pass map keys, which are unique by construction —
/// duplicate detection is not part of this contract.
pub fn dictionary_file_set_is_complete(names: &[&str]) -> bool {
    DICTIONARY_FILE_NAMES
        .iter()
        .all(|expected| names.contains(expected))
}

/// Only the eight lindera files are deflate-compressed on the CDN;
/// metadata.json is stored uncompressed.
fn is_inflatable(file_name: &str) -> bool {
    file_name != METADATA_FILE
}

/// Load the SudachiDict tokenizer dictionary.
///
/// Cache-hit path: the v2 cache already holds RAW (inflated) files — no
/// decompression happens, which removes the dominant CPU cost of every
/// repeated start.
///
/// Cache-miss path: files are fetched deflated (from the CDN cache or the
/// network), inflated one at a time and persisted to the v2 cache right
/// away, so the inflate cost is paid exactly once per dictionary version.
/// Peak WASM heap stays bounded by `sum(raw processed so far) + deflated
/// current file + its transient JS copy during the cache write`.
pub async fn load_dictionary() -> Result<(), OrigaError> {
    if is_dictionary_loaded() {
        tracing::debug!("📖 Dictionary already loaded");
        return Ok(());
    }
    let start = now_ms();
    tracing::info!("📖 Loading tokenizer dictionary...");

    let files = match get_cached_dictionary_files().await {
        Some(raw_files) => {
            tracing::debug!("📖 Raw dictionary cache hit ({} files)", raw_files.len());
            cleanup_legacy_dictionary_cache().await;
            raw_files
        },
        None => fetch_inflate_and_cache_raw().await?,
    };

    let data = assemble_dictionary_data(files)?;
    init_dictionary(data)?;

    tracing::info!("📖 Dictionary loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}

/// Persistence sink for the fetch→inflate→persist pipeline. Production
/// writes each raw file into the Cache API; tests inject failures to pin
/// the best-effort contract (a cache failure must never kill the load).
trait RawFileSink {
    async fn persist(&self, path: &str, raw: &[u8]) -> Result<(), OrigaError>;
}

/// Production sink: the v2 raw Cache API entry for one dictionary file.
struct CacheApiSink;

impl RawFileSink for CacheApiSink {
    async fn persist(&self, path: &str, raw: &[u8]) -> Result<(), OrigaError> {
        save_dictionary_file_to_cache(path, raw).await
    }
}

/// Fetch deflated files from the CDN one at a time (words first), inflate
/// them and hand each raw file to the sink right away — no buffer clones:
/// the single-file cache API borrows the bytes and only the Cache API's
/// own JS-side copy coexists with the raw Vec. Cache persistence is
/// best-effort: a sink failure is logged and the load continues, because
/// the v2 raw cache (~344 MB, single 223 MB entries) can exceed the Cache
/// API quota on quota-tight WebViews (iOS/macOS WKWebView) — a refused
/// cache write must never take the tokenizer down (v0.7.5 semantics; the
/// fatal `?` silently killed furigana and phrase tokenization in
/// v0.7.6-rc1).
async fn fetch_inflate_and_persist_via<P: CdnProvider, S: RawFileSink>(
    provider: &P,
    sink: &S,
) -> Result<Vec<(String, Vec<u8>)>, OrigaError> {
    let mut raw_files: Vec<(String, Vec<u8>)> = Vec::with_capacity(PIPELINE_ORDER.len());
    for file in PIPELINE_ORDER {
        let path = dict_path(file);
        let compressed = provider.fetch_bytes(&path).await?;
        let raw = if is_inflatable(file) {
            inflate(&compressed)?
        } else {
            compressed
        };
        yield_to_browser().await;
        if let Err(e) = sink.persist(&path, &raw).await {
            tracing::warn!("Failed to cache raw dictionary file {path}: {e:?}");
        } else {
            tracing::debug!("📖 Cached raw {path} ({} bytes)", raw.len());
        }
        raw_files.push((path, raw));
    }

    Ok(raw_files)
}

async fn fetch_inflate_and_cache_raw() -> Result<Vec<(String, Vec<u8>)>, OrigaError> {
    let files = fetch_inflate_and_persist_via(cdn_provider(), &CacheApiSink).await?;
    // The retired v1 cache is deleted only after the full v2 set is stored,
    // so an offline user never loses a working dictionary to a half-finished
    // migration. Wasm-only side effect — kept out of the hermetic pipeline
    // core above (native tests exercise the core without a window).
    cleanup_legacy_dictionary_cache().await;
    Ok(files)
}

/// Group raw file bytes under their bare names (dropping the versioned
/// directory prefix) and assemble `DictionaryData`.
fn assemble_dictionary_data(files: Vec<(String, Vec<u8>)>) -> Result<DictionaryData, OrigaError> {
    let mut by_name: HashMap<String, Vec<u8>> = HashMap::with_capacity(files.len());
    for (path, bytes) in files {
        let name = path.rsplit('/').next().unwrap_or(&path).to_string();
        by_name.insert(name.clone(), bytes);
    }

    if !dictionary_file_set_is_complete(&by_name.keys().map(String::as_str).collect::<Vec<_>>()) {
        return Err(OrigaError::TokenizerError {
            reason: "dictionary file set incomplete after load".to_string(),
        });
    }

    let mut get = |name: &str| -> Result<Vec<u8>, OrigaError> {
        by_name
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

#[cfg(test)]
mod tests {
    use super::*;

    fn complete_set() -> Vec<&'static str> {
        DICTIONARY_FILE_NAMES.to_vec()
    }

    fn set_without(name: &str) -> Vec<&'static str> {
        complete_set().into_iter().filter(|n| *n != name).collect()
    }

    fn set_with_extra_file() -> Vec<&'static str> {
        let mut names = complete_set();
        names.push("unexpected.bin");
        names
    }

    #[rstest::rstest]
    #[case::complete_set(complete_set(), true)]
    #[case::missing_words(set_without("dict.words"), false)]
    #[case::missing_metadata(set_without("metadata.json"), false)]
    #[case::extra_file_alongside_complete_set(set_with_extra_file(), true)]
    fn file_set_completeness_classifies_cached_names(
        #[case] names: Vec<&str>,
        #[case] expected: bool,
    ) {
        assert_eq!(dictionary_file_set_is_complete(&names), expected);
    }

    #[test]
    fn metadata_is_not_inflatable_but_lindera_files_are() {
        assert!(is_inflatable("dict.words"));
        assert!(is_inflatable("char_def.bin"));
        assert!(!is_inflatable("metadata.json"));
    }

    #[test]
    fn inflate_round_trips_raw_deflate_stream() {
        use std::io::Write;
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(b"payload for the tokenizer").unwrap();
        let compressed = encoder.finish().unwrap();

        let raw = inflate(&compressed).unwrap();

        assert_eq!(raw, b"payload for the tokenizer");
    }

    fn deflate(payload: &[u8]) -> Vec<u8> {
        use std::io::Write;
        let mut encoder =
            flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(payload).unwrap();
        encoder.finish().unwrap()
    }

    /// CdnProvider mock over the external CDN boundary: serves a tiny valid
    /// payload for every pipeline path (deflated for the eight lindera
    /// files, raw JSON for metadata.json) and fails for anything else.
    struct TinyDictCdn;

    impl CdnProvider for TinyDictCdn {
        fn fetch_text(&self, path: &str) -> impl Future<Output = Result<String, OrigaError>> {
            std::future::ready(Err(OrigaError::NetworkError {
                url: path.to_string(),
                reason: "text fetch is not stubbed in pipeline tests".to_string(),
            }))
        }

        fn fetch_bytes(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, OrigaError>> {
            let result = match PIPELINE_ORDER.iter().find(|f| dict_path(f) == path) {
                Some(file) if !is_inflatable(file) => Ok(br#"{"name":"sudachidict"}"#.to_vec()),
                Some(_) => Ok(deflate(b"tiny-raw-payload")),
                None => Err(OrigaError::NetworkError {
                    url: path.to_string(),
                    reason: "not stubbed".to_string(),
                }),
            };
            std::future::ready(result)
        }
    }

    /// Sink whose every write is rejected, mirroring an exhausted Cache API
    /// quota (`QuotaExceededError` on put — the v0.7.6-rc1 regression
    /// trigger on quota-tight WebViews).
    struct QuotaExhaustedSink;

    impl RawFileSink for QuotaExhaustedSink {
        async fn persist(&self, path: &str, _raw: &[u8]) -> Result<(), OrigaError> {
            Err(OrigaError::RepositoryError {
                reason: format!("Cache put failed for {path}: simulated QuotaExceededError"),
            })
        }
    }

    struct RecordingSink {
        persisted: std::cell::RefCell<Vec<String>>,
    }

    impl RawFileSink for RecordingSink {
        async fn persist(&self, path: &str, _raw: &[u8]) -> Result<(), OrigaError> {
            self.persisted.borrow_mut().push(path.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn quota_exhausted_cache_write_does_not_kill_the_dictionary_load() {
        // Arrange
        let provider = TinyDictCdn;
        let sink = QuotaExhaustedSink;

        // Act
        let files = fetch_inflate_and_persist_via(&provider, &sink).await;

        // Assert
        let files = files
            .expect("a cache-write failure must not kill the dictionary load (v0.7.5 semantics)");
        assert_eq!(files.len(), PIPELINE_ORDER.len());
        for (path, raw) in &files {
            let bare_name = path.rsplit('/').next().unwrap_or(path);
            if is_inflatable(bare_name) {
                assert_eq!(raw, b"tiny-raw-payload", "{path} must be inflated");
            }
        }
    }

    #[tokio::test]
    async fn successful_pipeline_persists_every_file() {
        // Arrange
        let sink = RecordingSink {
            persisted: std::cell::RefCell::new(Vec::new()),
        };

        // Act
        let files = fetch_inflate_and_persist_via(&TinyDictCdn, &sink)
            .await
            .expect("the tiny pipeline must load");

        // Assert: best-effort must not degrade into never-persisting.
        assert_eq!(files.len(), PIPELINE_ORDER.len());
        assert_eq!(sink.persisted.borrow().len(), PIPELINE_ORDER.len());
    }
}
