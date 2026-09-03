use origa::dictionary::cdn_blob::{guard_matches, split_blob};
use origa::dictionary::grammar::{GrammarData, init_grammar, is_grammar_loaded};
use origa::dictionary::kanji::{KanjiData, init_kanji, is_kanji_loaded};
use origa::dictionary::radical::{RadicalData, init_radicals, is_radicals_loaded};
use origa::dictionary::vocabulary::{
    VocabularyChunkData, VocabularyDatabase, init_vocabulary, init_vocabulary_from_rkyv,
    is_vocabulary_loaded, serialize_vocabulary_to_rkyv, set_vocabulary_database,
    vocabulary_database_from_blob_rkyv,
};
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

use crate::repository::cache_manager::guard_expectation_for;
use crate::repository::cdn_provider;
use crate::repository::{get_cached_vocabulary_rkyv, save_vocabulary_to_cache_rkyv};
use crate::utils::{now_ms, yield_to_browser};

/// CDN path of the pre-built deterministic vocabulary blob.
const VOCABULARY_BLOB_PATH: &str = "dictionary/vocabulary.rkyv";

/// Chunk paths feeding the blob — used for the manifest guard expectation.
const VOCABULARY_CHUNK_PATHS: [&str; 11] = [
    "dictionary/chunk_01.json",
    "dictionary/chunk_02.json",
    "dictionary/chunk_03.json",
    "dictionary/chunk_04.json",
    "dictionary/chunk_05.json",
    "dictionary/chunk_06.json",
    "dictionary/chunk_07.json",
    "dictionary/chunk_08.json",
    "dictionary/chunk_09.json",
    "dictionary/chunk_10.json",
    "dictionary/chunk_11.json",
];

pub async fn load_vocabulary() -> Result<(), OrigaError> {
    if is_vocabulary_loaded() {
        tracing::debug!("📖 Vocabulary already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading vocabulary...");

    // Fastest path: pre-built rkyv blob from the CDN (cache-first). Skips
    // both the JSON parsing of ~35 MB of chunks and the client-side
    // re-serialization into the local rkyv cache.
    if let Some(database) = load_vocabulary_from_cdn_blob(cdn_provider()).await? {
        set_vocabulary_database(database)?;
        tracing::info!(
            "📖 Vocabulary loaded from CDN rkyv blob ({:.2}s)",
            (now_ms() - start) / 1000.0
        );
        return Ok(());
    }

    // Known coverage gap (accepted at review): the fallback chain below —
    // client rkyv cache → JSON chunks → client re-serialization — lives in
    // this wrapper because the Cache API layer is not natively testable; it
    // is behaviour-covered by e2e only. The CDN-rkyv core above carries the
    // native mock tests.
    // Fast path: try loading from rkyv cache (pre-parsed VocabularyDatabase).
    match get_cached_vocabulary_rkyv().await {
        Ok(Some(bytes)) => {
            tracing::info!("📖 Cached vocabulary found, {} bytes", bytes.len());
            yield_to_browser().await;
            match init_vocabulary_from_rkyv(&bytes) {
                Ok(()) => {
                    tracing::info!(
                        "📖 Vocabulary loaded from rkyv cache ({:.2}s)",
                        (now_ms() - start) / 1000.0
                    );
                    return Ok(());
                },
                Err(e) => {
                    tracing::warn!(
                        error = ?e,
                        "Failed to load vocabulary from rkyv cache, falling back to network"
                    );
                },
            }
        },
        Ok(None) => {
            tracing::debug!("📖 No rkyv vocabulary cache found, loading from network");
        },
        Err(e) => {
            tracing::warn!(
                "Vocabulary cache read failed, loading from network: {:?}",
                e
            );
        },
    }

    // Slow path: fetch JSON chunks from CDN and parse.
    let cdn = cdn_provider();

    // Batched fetch: 11 JSON chunks (~35 MB total). Fetching all 11 at once
    // via join_all caused WASM OOM on iOS. Batches of 3 keep peak memory
    // bounded while still parallelizing for speed (~4 rounds instead of 11
    // sequential requests).
    const BATCH_SIZE: usize = 3;
    let mut all_chunks: Vec<String> = Vec::with_capacity(11);

    for batch_start in (1..=11).step_by(BATCH_SIZE) {
        let batch_end = (batch_start + BATCH_SIZE - 1).min(11);
        let batch_futures: Vec<_> = (batch_start..=batch_end)
            .map(|i| {
                let path = format!("dictionary/chunk_{:02}.json", i);
                async move { cdn.fetch_text(&path).await }
            })
            .collect();
        let batch_results = futures::future::join_all(batch_futures).await;
        for result in batch_results {
            all_chunks.push(result?);
        }
        yield_to_browser().await;
    }

    let data = VocabularyChunkData {
        chunk_01: all_chunks[0].clone(),
        chunk_02: all_chunks[1].clone(),
        chunk_03: all_chunks[2].clone(),
        chunk_04: all_chunks[3].clone(),
        chunk_05: all_chunks[4].clone(),
        chunk_06: all_chunks[5].clone(),
        chunk_07: all_chunks[6].clone(),
        chunk_08: all_chunks[7].clone(),
        chunk_09: all_chunks[8].clone(),
        chunk_10: all_chunks[9].clone(),
        chunk_11: all_chunks[10].clone(),
    };

    yield_to_browser().await;
    init_vocabulary(data)?;

    // Cache the parsed VocabularyDatabase for future fast-path loads.
    let cache_start = now_ms();
    match serialize_vocabulary_to_rkyv() {
        Ok(mut bytes) => {
            if let Err(e) = save_vocabulary_to_cache_rkyv(&mut bytes).await {
                tracing::warn!("Failed to cache vocabulary (rkyv): {:?}", e);
            } else {
                tracing::info!(
                    "📖 Vocabulary cached (rkyv, {} bytes, {:.2}s)",
                    bytes.len(),
                    (now_ms() - cache_start) / 1000.0
                );
            }
        },
        Err(e) => {
            tracing::warn!("Failed to serialize vocabulary for caching: {:?}", e);
        },
    }

    tracing::info!("📖 Vocabulary loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}

/// CDN-rkyv fast path core. `Ok(None)` means "no usable blob — use the
/// fallback chain" (fetch error, invalid header, guard mismatch or payload
/// failure are all non-fatal: the original sources remain loadable).
async fn load_vocabulary_from_cdn_blob<P: CdnProvider>(
    provider: &P,
) -> Result<Option<VocabularyDatabase>, OrigaError> {
    let blob = match provider.fetch_bytes(VOCABULARY_BLOB_PATH).await {
        Ok(blob) => blob,
        Err(e) => {
            tracing::warn!("📖 Vocabulary rkyv blob unavailable ({e:?}), using fallback chain");
            return Ok(None);
        },
    };

    let (header, payload) = match split_blob(&blob) {
        Ok(split) => split,
        Err(e) => {
            tracing::warn!("📖 Vocabulary blob header invalid: {e}");
            return Ok(None);
        },
    };

    let expectation = guard_expectation_for(&VOCABULARY_CHUNK_PATHS);
    if !guard_matches(&header, &expectation) {
        tracing::warn!("📖 Vocabulary blob stale relative to manifest, using fallback chain");
        return Ok(None);
    }

    match vocabulary_database_from_blob_rkyv(payload) {
        Ok(database) => Ok(Some(database)),
        Err(e) => {
            tracing::warn!("📖 Vocabulary blob payload failed to deserialize: {e:?}");
            Ok(None)
        },
    }
}

pub async fn load_kanji() -> Result<(), OrigaError> {
    if is_kanji_loaded() {
        tracing::debug!("📖 Kanji already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading kanji...");

    let cdn = cdn_provider();
    let json = cdn.fetch_text("dictionary/kanji.json").await?;
    let data = KanjiData { kanji_json: json };

    yield_to_browser().await;
    init_kanji(data)?;

    tracing::info!("📖 Kanji loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}

pub async fn load_grammar() -> Result<(), OrigaError> {
    if is_grammar_loaded() {
        tracing::debug!("📖 Grammar already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading grammar...");

    let cdn = cdn_provider();
    let json = cdn.fetch_text("grammar/grammar.json").await?;
    let data = GrammarData { grammar_json: json };

    yield_to_browser().await;
    init_grammar(data)?;

    tracing::info!("📖 Grammar loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}

pub async fn load_radicals() -> Result<(), OrigaError> {
    if is_radicals_loaded() {
        tracing::debug!("📖 Radicals already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading radicals...");

    let cdn = cdn_provider();
    let json = cdn.fetch_text("dictionary/radicals.json").await?;
    let data = RadicalData {
        radicals_json: json,
    };

    yield_to_browser().await;
    init_radicals(data)?;

    tracing::info!("📖 Radicals loaded ({:.2}s)", (now_ms() - start) / 1000.0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::future::Future;

    use origa::dictionary::cdn_blob::{
        BlobHeader, SCHEMA_VERSION, build_blob, manifest_guard_from_hex_hashes, sha256_bytes,
    };
    use origa::dictionary::vocabulary::{
        build_vocabulary_database_from_chunks, serialize_vocabulary_blob_to_rkyv,
    };

    use super::*;

    fn chunk_json() -> String {
        r#"{"猫": {"russian_translation": "кот", "english_translation": "cat"}}"#.to_string()
    }

    fn chunk_data() -> VocabularyChunkData {
        VocabularyChunkData {
            chunk_01: chunk_json(),
            chunk_02: "{}".to_string(),
            chunk_03: "{}".to_string(),
            chunk_04: "{}".to_string(),
            chunk_05: "{}".to_string(),
            chunk_06: "{}".to_string(),
            chunk_07: "{}".to_string(),
            chunk_08: "{}".to_string(),
            chunk_09: "{}".to_string(),
            chunk_10: "{}".to_string(),
            chunk_11: "{}".to_string(),
        }
    }

    /// Recording mock over the external CDN boundary: canned bytes per path
    /// plus a log of every requested path (asserts the fast path never
    /// touches the JSON chunks).
    struct MockCdn {
        blob: Option<Vec<u8>>,
        requested: RefCell<Vec<String>>,
    }

    impl MockCdn {
        fn with_blob(blob: Option<Vec<u8>>) -> Self {
            Self {
                blob,
                requested: RefCell::new(Vec::new()),
            }
        }

        fn requested_paths(&self) -> Vec<String> {
            self.requested.borrow().clone()
        }
    }

    impl CdnProvider for MockCdn {
        fn fetch_text(&self, path: &str) -> impl Future<Output = Result<String, OrigaError>> {
            self.requested.borrow_mut().push(path.to_string());
            std::future::ready(Err(OrigaError::NetworkError {
                url: path.to_string(),
                reason: "text fetch is not stubbed in blob tests".to_string(),
            }))
        }

        fn fetch_bytes(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, OrigaError>> {
            self.requested.borrow_mut().push(path.to_string());
            let result = match (path, &self.blob) {
                (VOCABULARY_BLOB_PATH, Some(bytes)) => Ok(bytes.clone()),
                _ => Err(OrigaError::NetworkError {
                    url: path.to_string(),
                    reason: "not stubbed".to_string(),
                }),
            };
            std::future::ready(result)
        }
    }

    fn valid_blob() -> Vec<u8> {
        let database = build_vocabulary_database_from_chunks(chunk_data()).unwrap();
        let payload = serialize_vocabulary_blob_to_rkyv(&database).unwrap();

        let chunk_bytes = chunk_json().into_bytes();
        let hex_hashes: Vec<String> = std::iter::repeat_n(
            {
                let digest = sha256_bytes(&chunk_bytes);
                digest
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            },
            VOCABULARY_CHUNK_PATHS.len(),
        )
        .collect();
        let hex_refs: Vec<&str> = hex_hashes.iter().map(String::as_str).collect();

        let header = BlobHeader {
            schema_version: SCHEMA_VERSION,
            source_sha256: sha256_bytes(b"concatenated chunks"),
            manifest_guard: manifest_guard_from_hex_hashes(&hex_refs),
        };
        build_blob(&header, &payload)
    }

    #[tokio::test]
    async fn valid_blob_loads_database_without_fetching_chunks() {
        // Arrange
        let provider = MockCdn::with_blob(Some(valid_blob()));

        // Act
        let database = load_vocabulary_from_cdn_blob(&provider)
            .await
            .unwrap()
            .expect("valid blob must load");

        // Assert
        assert_eq!(
            database.get_translations("猫", &origa::domain::NativeLanguage::Russian),
            Some(vec!["кот".to_string()])
        );
        let chunk_requested = provider
            .requested_paths()
            .iter()
            .any(|path| path.starts_with("dictionary/chunk_"));
        assert!(!chunk_requested, "fast path must not fetch JSON chunks");
    }

    #[tokio::test]
    async fn missing_blob_yields_none_for_the_fallback_chain() {
        // Arrange
        let provider = MockCdn::with_blob(None);

        // Act
        let database = load_vocabulary_from_cdn_blob(&provider).await.unwrap();

        // Assert
        assert!(database.is_none());
    }

    #[tokio::test]
    async fn corrupted_header_yields_none_for_the_fallback_chain() {
        // Arrange
        let mut blob = valid_blob();
        blob[0] = b'X';
        let provider = MockCdn::with_blob(Some(blob));

        // Act
        let database = load_vocabulary_from_cdn_blob(&provider).await.unwrap();

        // Assert
        assert!(database.is_none());
    }

    #[tokio::test]
    async fn future_schema_version_yields_none_for_the_fallback_chain() {
        // Arrange
        let mut blob = valid_blob();
        let version = SCHEMA_VERSION + 1;
        blob[4..8].copy_from_slice(&version.to_le_bytes());
        let provider = MockCdn::with_blob(Some(blob));

        // Act
        let database = load_vocabulary_from_cdn_blob(&provider).await.unwrap();

        // Assert
        assert!(database.is_none());
    }
}
