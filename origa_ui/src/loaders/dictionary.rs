use crate::repository::{cdn_provider, get_cached_lindera_rkyv, save_lindera_to_cache_rkyv};
use crate::utils::{now_ms, yield_to_browser};
use flate2::read::DeflateDecoder;
use origa::domain::{
    DictionaryData, OrigaError, init_tokenizer_from_rkyv_cached, is_dictionary_loaded,
    serialize_cached_lindera_to_rkyv,
};
use origa::traits::CdnProvider;
use std::io::Read;

fn decompress(data: Vec<u8>) -> Result<Vec<u8>, OrigaError> {
    let mut decoder = DeflateDecoder::new(&data[..]);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| OrigaError::TokenizerError {
            reason: format!("Decompression failed: {}", e),
        })?;
    Ok(decompressed)
}

pub async fn load_dictionary() -> Result<(), OrigaError> {
    if is_dictionary_loaded() {
        tracing::debug!("📖 Dictionary already loaded");
        return Ok(());
    }
    let start = now_ms();
    tracing::info!("📖 Loading Unidic dictionary...");

    // Fast path: try loading from cached lindera structures (pre-built
    // lindera Dictionary components serialized via rkyv). This skips
    // all lindera load() calls on cache hit.
    match get_cached_lindera_rkyv().await {
        Ok(Some(bytes)) => {
            tracing::info!("📖 Cached lindera structures found, {} bytes", bytes.len());
            yield_to_browser().await;
            match init_tokenizer_from_rkyv_cached(&bytes) {
                Ok(()) => {
                    tracing::info!(
                        "📖 Dictionary loaded from cached lindera structures ({:.2}s)",
                        (now_ms() - start) / 1000.0
                    );
                    return Ok(());
                },
                Err(e) => {
                    tracing::warn!(
                        error = ?e,
                        "Failed to load from cached lindera structures, falling back to network"
                    );
                },
            }
        },
        Ok(None) => {
            tracing::debug!("📖 No cached lindera structures found, loading from network");
        },
        Err(e) => {
            tracing::warn!("Cache read failed, loading from network: {:?}", e);
        },
    }

    // Slow path (cache miss): fetch from CDN, decompress, build lindera
    // structures, then serialize them for future cache hits.
    let data = load_dictionary_from_network().await?;
    yield_to_browser().await;

    // Build lindera structures — DictionaryData is consumed (moved into
    // lindera load() calls), no ~204 MB clone.
    let cached = origa::domain::build_cached_lindera(data)?;
    yield_to_browser().await;

    // Serialize for future cache hits BEFORE moving into the tokenizer.
    let cache_start = now_ms();
    match serialize_cached_lindera_to_rkyv(&cached) {
        Ok(bytes) => {
            tracing::info!(
                "📖 Cached lindera structures serialized ({} bytes, {:.2}s)",
                bytes.len(),
                (now_ms() - cache_start) / 1000.0
            );
            if let Err(e) = save_lindera_to_cache_rkyv(&bytes).await {
                tracing::warn!("Failed to cache lindera structures: {:?}", e);
            }
        },
        Err(e) => {
            tracing::warn!(
                "Failed to serialize lindera structures for caching: {:?}",
                e
            );
        },
    }

    // Now move the structures into the static Tokenizer.
    origa::domain::init_tokenizer_from_cached(cached)?;

    tracing::info!(
        "📖 Dictionary loaded from network ({:.2}s)",
        (now_ms() - start) / 1000.0
    );
    Ok(())
}

async fn fetch_file(field: &str, path: &str) -> Result<(String, Vec<u8>), OrigaError> {
    let cdn = cdn_provider();
    let bytes = cdn.fetch_bytes(path).await?;
    let decompressed = if field == "metadata" {
        bytes
    } else {
        decompress(bytes)?
    };
    Ok((field.to_string(), decompressed))
}

async fn load_dictionary_from_network() -> Result<DictionaryData, OrigaError> {
    tracing::info!("📖 Fetching dictionary files...");

    let files = [
        ("char_def", "dictionaries/char_def.bin"),
        ("matrix", "dictionaries/matrix.mtx"),
        ("dict_da", "dictionaries/dict.da"),
        ("dict_vals", "dictionaries/dict.vals"),
        ("unk", "dictionaries/unk.bin"),
        ("words_idx", "dictionaries/dict.wordsidx"),
        ("words", "dictionaries/dict.words"),
        ("metadata", "dictionaries/metadata.json"),
    ];

    let fetch_start = now_ms();

    // Sequential fetch: UniDic files are large (matrix.mtx alone is 25 MB
    // compressed → ~100 MB decompressed). Fetching all 8 simultaneously via
    // join_all caused WASM OOM on iOS. Sequential download keeps peak memory
    // to one file at a time; each fetch_bytes → decompress → store cycle
    // drops the compressed bytes before the next download starts.
    let mut results = Vec::with_capacity(files.len());
    for (field, path) in &files {
        results.push(fetch_file(field, path).await?);
    }

    tracing::info!(
        "📖 Dictionary files fetched ({:.2}s)",
        (now_ms() - fetch_start) / 1000.0
    );

    let mut data = DictionaryData {
        char_def: Vec::new(),
        matrix: Vec::new(),
        dict_da: Vec::new(),
        dict_vals: Vec::new(),
        unk: Vec::new(),
        words_idx: Vec::new(),
        words: Vec::new(),
        metadata: Vec::new(),
    };
    for (field, decompressed) in results {
        match field.as_str() {
            "char_def" => data.char_def = decompressed,
            "matrix" => data.matrix = decompressed,
            "dict_da" => data.dict_da = decompressed,
            "dict_vals" => data.dict_vals = decompressed,
            "unk" => data.unk = decompressed,
            "words_idx" => data.words_idx = decompressed,
            "words" => data.words = decompressed,
            "metadata" => data.metadata = decompressed,
            _ => {},
        }
    }
    Ok(data)
}
