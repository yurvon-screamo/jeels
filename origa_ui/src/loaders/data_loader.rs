use origa::dictionary::grammar::{GrammarData, init_grammar, is_grammar_loaded};
use origa::dictionary::kanji::{KanjiData, init_kanji, is_kanji_loaded};
use origa::dictionary::radical::{RadicalData, init_radicals, is_radicals_loaded};
use origa::dictionary::vocabulary::{
    VocabularyChunkData, init_vocabulary, init_vocabulary_from_rkyv, is_vocabulary_loaded,
    serialize_vocabulary_to_rkyv,
};
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

use crate::repository::cdn_provider;
use crate::repository::{get_cached_vocabulary_rkyv, save_vocabulary_to_cache_rkyv};
use crate::utils::{now_ms, yield_to_browser};

pub async fn load_vocabulary() -> Result<(), OrigaError> {
    if is_vocabulary_loaded() {
        tracing::debug!("📖 Vocabulary already loaded");
        return Ok(());
    }

    let start = now_ms();
    tracing::info!("📖 Loading vocabulary...");

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
        Ok(bytes) => {
            if let Err(e) = save_vocabulary_to_cache_rkyv(&bytes).await {
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
