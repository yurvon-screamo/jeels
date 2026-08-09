use std::sync::Arc;

use futures::stream::{self, StreamExt};
use origa::dictionary::phrase::index_version;
use origa::domain::OrigaError;
use origa::traits::CdnProvider;

use crate::repository::cdn_provider;

const BUNDLE_DOWNLOADED_KEY: &str = "/__origa_bundle_downloaded__";
const CONCURRENCY: usize = 20;
const PHRASE_BUNDLE_COUNT: usize = 4;

#[derive(Clone, Default)]
pub struct PreCacheProgress {
    pub completed: usize,
    pub total: usize,
    pub current_file: String,
}

#[derive(Clone, Default)]
pub struct DownloadResult {
    pub total: usize,
    pub succeeded: usize,
}

pub fn get_base_bundle_resources() -> Vec<String> {
    let mut resources: Vec<String> = Vec::new();

    resources.extend([
        "dictionaries/char_def.bin".to_string(),
        "dictionaries/matrix.mtx".to_string(),
        "dictionaries/dict.da".to_string(),
        "dictionaries/dict.vals".to_string(),
        "dictionaries/unk.bin".to_string(),
        "dictionaries/dict.wordsidx".to_string(),
        "dictionaries/dict.words".to_string(),
        "dictionaries/metadata.json".to_string(),
        "dictionaries/JmdictFurigana.txt".to_string(),
    ]);

    for i in 1..=11 {
        resources.push(format!("dictionary/chunk_{:02}.json", i));
    }

    resources.extend([
        "dictionary/kanji.json".to_string(),
        "dictionary/radicals.json".to_string(),
    ]);

    resources.push("grammar/grammar.json".to_string());
    resources.push("phrases/phrase_index.json".to_string());

    // Phrase data bundles (4 files replace 198 individual chunks).
    // After download, extract_phrase_bundles_to_cache() parses each bundle
    // and stores individual chunks in Cache API so phrase_data_loader
    // gets cache hits without per-chunk CDN requests.
    for i in 0..PHRASE_BUNDLE_COUNT {
        resources.push(format!("phrases/data_bundle_{}.json", i));
    }

    resources.push("pitch/index.json".to_string());

    resources.extend([
        "well_known_set/well_known_sets_meta.json".to_string(),
        "well_known_set/well_known_types_meta.json".to_string(),
        "well_known_set/jlpt_n1.json".to_string(),
        "well_known_set/jlpt_n2.json".to_string(),
        "well_known_set/jlpt_n3.json".to_string(),
        "well_known_set/jlpt_n4.json".to_string(),
        "well_known_set/jlpt_n5.json".to_string(),
    ]);

    // Kanji SVGs are pre-cached on demand via card_precache_loader
    // for only the kanji the user actually studies.
    // Pre-fetching all ~6000+ kanji from the dictionary generates
    // many 404s for rare kanji that have no SVG files on the CDN.

    resources
}

pub async fn batch_download(
    paths: Vec<String>,
    on_progress: impl Fn(PreCacheProgress) + Clone + 'static,
) -> Result<DownloadResult, OrigaError> {
    let total = paths.len();
    let completed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let succeeded = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    stream::iter(paths)
        .map(|path| {
            let completed = completed.clone();
            let succeeded = succeeded.clone();
            let on_progress = on_progress.clone();
            async move {
                match cdn_provider::prefetch_to_cache(&path).await {
                    Ok(()) => {
                        succeeded.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    },
                    Err(e) => {
                        tracing::warn!(path = %path, error = ?e, "Failed to prefetch");
                    },
                }
                let done = completed.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                on_progress(PreCacheProgress {
                    completed: done,
                    total,
                    current_file: path,
                });
            }
        })
        .buffer_unordered(CONCURRENCY)
        .collect::<Vec<()>>()
        .await;

    Ok(DownloadResult {
        total,
        succeeded: succeeded.load(std::sync::atomic::Ordering::Relaxed),
    })
}

async fn mark_bundle_downloaded() -> Result<(), OrigaError> {
    cdn_provider::store_cache_marker(BUNDLE_DOWNLOADED_KEY, "ok").await
}

pub async fn is_bundle_downloaded() -> bool {
    cdn_provider::is_cached(BUNDLE_DOWNLOADED_KEY).await
}

pub async fn precache_base_bundle(
    on_progress: impl Fn(PreCacheProgress) + Clone + 'static,
) -> Result<DownloadResult, OrigaError> {
    let resources = get_base_bundle_resources();
    tracing::info!("Starting base bundle download: {} files", resources.len());
    let result = batch_download(resources, on_progress).await?;

    let success_rate = if result.total > 0 {
        result.succeeded as f64 / result.total as f64
    } else {
        0.0
    };

    if success_rate >= 0.95 {
        mark_bundle_downloaded().await?;
        tracing::info!(
            "Bundle downloaded: {}/{} succeeded",
            result.succeeded,
            result.total
        );
    } else {
        tracing::warn!(
            "Bundle download incomplete: {}/{} succeeded",
            result.succeeded,
            result.total
        );
    }

    Ok(result)
}

/// After downloading phrase data bundles, extract individual chunks into
/// Cache API. This lets phrase_data_loader.rs work unchanged (cache hits).
///
/// Each bundle is `{"p0000": [...], "p0001": [...], ...}`.
/// We store each value under `phrases/data/p0000.json?v=HASH` so the cache
/// key matches what phrase_data_loader fetches.
pub async fn extract_phrase_bundles_to_cache() -> Result<usize, OrigaError> {
    let cdn = cdn_provider();
    let (_, hash) = index_version();
    let mut extracted = 0usize;

    for bundle_idx in 0..PHRASE_BUNDLE_COUNT {
        let bundle_path = format!("phrases/data_bundle_{}.json", bundle_idx);
        let bundle_json = match cdn.fetch_text(&bundle_path).await {
            Ok(text) => text,
            Err(e) => {
                tracing::warn!(
                    bundle = bundle_idx,
                    error = ?e,
                    "Failed to fetch phrase data bundle for extraction, skipping"
                );
                continue;
            }
        };

        // Parse as generic JSON to avoid coupling to phrase types
        let bundle: std::collections::HashMap<String, serde_json::Value> =
            match serde_json::from_str(&bundle_json) {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(
                        bundle = bundle_idx,
                        error = %e,
                        "Failed to parse phrase data bundle, skipping"
                    );
                    continue;
                }
            };

        for (chunk_key, chunk_value) in &bundle {
            let cache_path = format!("phrases/data/{}.json?v={}", chunk_key, hash);
            let chunk_text = match serde_json::to_string(chunk_value) {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(chunk = %chunk_key, error = %e, "Failed to serialize phrase chunk, skipping");
                    continue;
                }
            };
            if let Err(e) = cdn_provider::store_text_in_cache(&cache_path, &chunk_text).await {
                tracing::warn!(
                    chunk = %chunk_key,
                    error = ?e,
                    "Failed to store phrase chunk in cache during extraction"
                );
            } else {
                extracted += 1;
            }
        }

        tracing::info!(
            bundle = bundle_idx,
            chunks = bundle.len(),
            "Extracted phrase data bundle to cache"
        );
    }

    tracing::info!(total_extracted = extracted, "Phrase data extraction complete");
    Ok(extracted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_bundle_includes_dictionaries() {
        let resources = get_base_bundle_resources();

        assert!(resources.contains(&"dictionaries/char_def.bin".to_string()));
        assert!(resources.contains(&"dictionaries/matrix.mtx".to_string()));
        assert!(resources.contains(&"dictionaries/JmdictFurigana.txt".to_string()));
        assert!(resources.contains(&"dictionary/kanji.json".to_string()));
        assert!(resources.contains(&"dictionary/radicals.json".to_string()));
        assert!(resources.contains(&"grammar/grammar.json".to_string()));
        assert!(resources.contains(&"phrases/phrase_index.json".to_string()));
        assert!(resources.contains(&"pitch/index.json".to_string()));
        assert!(resources.contains(&"well_known_set/jlpt_n5.json".to_string()));
    }

    #[test]
    fn base_bundle_includes_vocabulary_chunks() {
        let resources = get_base_bundle_resources();
        for i in 1..=11 {
            assert!(resources.contains(&format!("dictionary/chunk_{:02}.json", i)));
        }
    }

    #[test]
    fn base_bundle_includes_phrase_data_bundles() {
        let resources = get_base_bundle_resources();
        for i in 0..PHRASE_BUNDLE_COUNT {
            assert!(
                resources.contains(&format!("phrases/data_bundle_{}.json", i)),
                "Missing phrases/data_bundle_{}.json",
                i
            );
        }
    }

    #[test]
    fn base_bundle_does_not_include_individual_phrase_chunks() {
        // Individual phrase data files should NOT be in the base bundle
        // anymore — they're replaced by data_bundle_0..3
        let resources = get_base_bundle_resources();
        assert!(!resources.iter().any(|r| r.starts_with("phrases/data/p")));
    }
}
