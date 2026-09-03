use origa::domain::OrigaError;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

/// Cache name for the raw (inflated) dictionary files.
///
/// v2 stores INFLATED bytes: a repeated start reads them and skips the
/// 28→223 MB deflate decompression entirely. v1 stored deflated files and
/// is deleted on migration (see `cleanup_legacy_dictionary_cache`); the
/// name bump keeps a rolled-back client from inflating raw bytes.
pub const DICTIONARY_FILES_CACHE_NAME: &str = "origa-dictionary-files-v2-raw";

/// Retired v1 cache (deflated format) — dropped once v2 is populated.
pub const LEGACY_DICTIONARY_FILES_CACHE_NAME: &str = "origa-dictionary-files-v1";

/// Cache key prefix for each deflated dictionary file.
pub const DICTIONARY_FILE_KEY_PREFIX: &str = "/__origa_dict_file__";

/// Cache name for pre-parsed VocabularyDatabase (rkyv).
pub const VOCABULARY_CACHE_NAME: &str = "origa-vocabulary-rkyv-v1";

/// Cache key for VocabularyDatabase rkyv blob.
pub const VOCABULARY_CACHE_KEY: &str = "/__origa_vocabulary_cached__";

/// The nine dictionary files cached under DICTIONARY_FILES_CACHE_NAME.
pub const DICTIONARY_FILE_NAMES: &[&str] = &[
    "char_def.bin",
    "matrix.mtx",
    "dict.trie",
    "dict.valsidx",
    "dict.vals",
    "unk.bin",
    "dict.wordsidx",
    "dict.words",
    "metadata.json",
];

async fn open_cache(name: &str) -> Result<web_sys::Cache, OrigaError> {
    let window = web_sys::window().ok_or_else(|| OrigaError::RepositoryError {
        reason: "No window found".to_string(),
    })?;
    let caches = window.caches().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Cache API not available: {:?}", e),
    })?;
    let cache =
        JsFuture::from(caches.open(name))
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to open cache {name}: {:?}", e),
            })?;
    let cache: web_sys::Cache = cache.dyn_into().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to cast cache {name}: {:?}", e),
    })?;
    Ok(cache)
}

async fn cache_read(cache: &web_sys::Cache, key: &str) -> Result<Option<Vec<u8>>, OrigaError> {
    let result = JsFuture::from(cache.match_with_str(key))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Cache match failed for {key}: {:?}", e),
        })?;

    if result.is_null() || result.is_undefined() {
        return Ok(None);
    }
    let response: web_sys::Response =
        result.dyn_into().map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to cast cached response for {key}: {:?}", e),
        })?;
    let ab_promise = response
        .array_buffer()
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to get array_buffer promise for {key}: {:?}", e),
        })?;
    let ab = JsFuture::from(ab_promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to read cached body for {key}: {:?}", e),
        })?;
    let arr = js_sys::Uint8Array::new(&ab);
    Ok(Some(arr.to_vec()))
}

async fn cache_write(cache: &web_sys::Cache, key: &str, bytes: &[u8]) -> Result<(), OrigaError> {
    let request = web_sys::Request::new_with_str(key).map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to create cache request for {key}: {:?}", e),
    })?;

    let uint8_array = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
    uint8_array.copy_from(bytes);
    let parts = js_sys::Array::of1(&uint8_array);
    let blob = web_sys::Blob::new_with_u8_array_sequence(&parts).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("Failed to build blob for {key}: {:?}", e),
        }
    })?;
    let response = web_sys::Response::new_with_opt_blob(Some(&blob)).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("Failed to build Response for {key}: {:?}", e),
        }
    })?;

    JsFuture::from(cache.put_with_request(&request, &response))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Cache put failed for {key}: {:?}", e),
        })?;
    Ok(())
}

/// Get all cached raw dictionary files. Returns None if any file is
/// missing (partial caches are treated as a miss).
pub async fn get_cached_dictionary_files() -> Option<Vec<(String, Vec<u8>)>> {
    let cache = open_cache(DICTIONARY_FILES_CACHE_NAME).await.ok()?;
    let mut out = Vec::with_capacity(DICTIONARY_FILE_NAMES.len());
    let mut total = 0usize;
    for name in DICTIONARY_FILE_NAMES {
        let key = format!("{DICTIONARY_FILE_KEY_PREFIX}{name}");
        let bytes = cache_read(&cache, &key).await.ok()??;
        total += bytes.len();
        out.push((format!("dictionaries/{name}"), bytes));
    }
    tracing::info!(
        "Dictionary file cache hit: {} files, {} bytes",
        out.len(),
        total
    );
    Some(out)
}

/// Delete the retired v1 (deflated) dictionary cache. Best-effort: a failure
/// only leaves ~40 MB of stale data behind, the next start retries.
pub async fn cleanup_legacy_dictionary_cache() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(caches) = window.caches().ok() else {
        return;
    };
    match JsFuture::from(caches.delete(LEGACY_DICTIONARY_FILES_CACHE_NAME)).await {
        Ok(_) => tracing::info!("Legacy v1 dictionary cache deleted"),
        Err(e) => tracing::warn!("Failed to delete legacy v1 dictionary cache: {e:?}"),
    }
}

/// Save deflated dictionary files to the Cache API.
pub async fn save_dictionary_files_to_cache(files: &[(String, Vec<u8>)]) -> Result<(), OrigaError> {
    let cache = open_cache(DICTIONARY_FILES_CACHE_NAME).await?;
    for (path, bytes) in files {
        let name = path.rsplit('/').next().unwrap_or(path);
        let key = format!("{DICTIONARY_FILE_KEY_PREFIX}{name}");
        cache_write(&cache, &key, bytes).await?;
    }
    tracing::info!("Dictionary files cached: {}", files.len());
    Ok(())
}

/// Get cached VocabularyDatabase as raw rkyv bytes.
pub async fn get_cached_vocabulary_rkyv() -> Result<Option<Vec<u8>>, OrigaError> {
    let cache = open_cache(VOCABULARY_CACHE_NAME).await?;
    cache_read(&cache, VOCABULARY_CACHE_KEY).await
}

/// Save VocabularyDatabase (rkyv bytes) to the Cache API.
pub async fn save_vocabulary_to_cache_rkyv(bytes: &mut [u8]) -> Result<(), OrigaError> {
    let cache = open_cache(VOCABULARY_CACHE_NAME).await?;
    cache_write(&cache, VOCABULARY_CACHE_KEY, bytes).await
}
