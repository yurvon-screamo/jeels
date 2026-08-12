use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use origa::domain::OrigaError;

use super::cdn_provider::cdn_cache_url;
use crate::utils::now_ms;

/// Legacy cache name for raw DictionaryData rkyv (pre-built lindera era).
/// Kept for invalidation only — no new entries are written here.
#[cfg(target_arch = "wasm32")]
pub const RKYV_CACHE_NAME: &str = "origa-dictionary-rkyv-v2";

/// Cache name for pre-built lindera structures (CachedLinderaDictionary).
pub const LINDERA_CACHE_NAME: &str = "origa-dictionary-lindera-v1";

/// Cache name for pre-parsed VocabularyDatabase (rkyv).
pub const VOCABULARY_CACHE_NAME: &str = "origa-vocabulary-rkyv-v1";

/// Cache key for CachedLinderaDictionary blob.
pub const LINDERA_CACHE_KEY: &str = "/__origa_lindera_cached__";

/// Cache key for VocabularyDatabase rkyv blob.
pub const VOCABULARY_CACHE_KEY: &str = "/__origa_vocabulary_cached__";

// ---------------------------------------------------------------------------
// Public API — thin wrappers over the shared get/save helpers.
// ---------------------------------------------------------------------------

/// Get cached lindera structures as raw rkyv bytes.
pub async fn get_cached_lindera_rkyv() -> Result<Option<Vec<u8>>, OrigaError> {
    let start = now_ms();
    let result = get_cached_blob(LINDERA_CACHE_NAME, LINDERA_CACHE_KEY).await?;
    if let Some(ref bytes) = result {
        tracing::info!(
            "Lindera cache loaded: {} bytes ({:.2}s)",
            bytes.len(),
            (now_ms() - start) / 1000.0
        );
    }
    Ok(result)
}

/// Save pre-built lindera structures (rkyv bytes) to the Cache API.
pub async fn save_lindera_to_cache_rkyv(bytes: &mut [u8]) -> Result<(), OrigaError> {
    let start = now_ms();
    save_blob_to_cache(LINDERA_CACHE_NAME, LINDERA_CACHE_KEY, bytes).await?;
    tracing::info!(
        "Lindera cache saved: {} bytes ({:.2}s)",
        bytes.len(),
        (now_ms() - start) / 1000.0
    );
    Ok(())
}

/// Get cached VocabularyDatabase as raw rkyv bytes.
pub async fn get_cached_vocabulary_rkyv() -> Result<Option<Vec<u8>>, OrigaError> {
    let result = get_cached_blob(VOCABULARY_CACHE_NAME, VOCABULARY_CACHE_KEY).await?;
    if let Some(ref bytes) = result {
        tracing::debug!("Vocabulary cache loaded: {} bytes", bytes.len());
    }
    Ok(result)
}

/// Delete the cached lindera blob (corrupted cache recovery).
pub async fn delete_cached_lindera() -> Result<(), OrigaError> {
    let cache = open_named_cache(LINDERA_CACHE_NAME).await?;
    JsFuture::from(cache.delete_with_str(&cdn_cache_url(LINDERA_CACHE_KEY)))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to delete lindera cache: {e:?}"),
        })?;
    Ok(())
}

/// Save pre-parsed VocabularyDatabase (rkyv bytes) to the Cache API.
pub async fn save_vocabulary_to_cache_rkyv(bytes: &mut [u8]) -> Result<(), OrigaError> {
    save_blob_to_cache(VOCABULARY_CACHE_NAME, VOCABULARY_CACHE_KEY, bytes).await
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Get a blob from a named Cache API store by key. Returns `None` on cache miss.
async fn get_cached_blob(cache_name: &str, cache_key: &str) -> Result<Option<Vec<u8>>, OrigaError> {
    let cache = open_named_cache(cache_name).await?;

    let response_option = JsFuture::from(cache.match_with_str(&cdn_cache_url(cache_key)))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to check cache '{cache_name}': {e:?}"),
        })?;

    if response_option.is_undefined() || response_option.is_null() {
        return Ok(None);
    }

    let response: web_sys::Response =
        response_option
            .dyn_into()
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to cast cache response: {e:?}"),
            })?;

    if !response.ok() {
        return Ok(None);
    }

    let array_buffer =
        JsFuture::from(
            response
                .array_buffer()
                .map_err(|e| OrigaError::RepositoryError {
                    reason: format!("Failed to get array_buffer: {e:?}"),
                })?,
        )
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to read cache '{cache_name}': {e:?}"),
        })?;

    let bytes = js_sys::Uint8Array::new(&array_buffer).to_vec();
    drop(array_buffer);
    Ok(Some(bytes))
}

/// Save a blob to a named Cache API store under the given key.
///
/// Uses `Uint8Array::view` for zero-copy: the typed array is backed
/// directly by the WASM linear memory slice — no JS ArrayBuffer allocation
/// or `copy_from`. Safe as long as no WASM memory growth happens during
/// the `cache.put` await (no Rust code runs in that window).
async fn save_blob_to_cache(
    cache_name: &str,
    cache_key: &str,
    bytes: &mut [u8],
) -> Result<(), OrigaError> {
    let cache = open_named_cache(cache_name).await?;

    // SAFETY: `Uint8Array::view` creates a view backed directly by WASM
    // linear memory. This is safe because no WASM memory growth can occur
    // during the `cache.put` await below — no Rust code runs in that window,
    // so the allocator cannot trigger `memory.grow` and invalidate the view.
    let view = unsafe { js_sys::Uint8Array::view(bytes) };

    let blob_property_bag = web_sys::BlobPropertyBag::new();
    blob_property_bag.set_type("application/octet-stream");

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&view);

    let blob =
        web_sys::Blob::new_with_buffer_source_sequence_and_options(&blob_parts, &blob_property_bag)
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to create blob: {e:?}"),
            })?;

    let response_init = web_sys::ResponseInit::new();
    response_init.set_status(200);
    response_init.set_status_text("OK");

    let response = web_sys::Response::new_with_opt_blob_and_init(Some(&blob), &response_init)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create response: {e:?}"),
        })?;

    JsFuture::from(cache.put_with_str(&cdn_cache_url(cache_key), &response))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to save to cache '{cache_name}': {e:?}"),
        })?;

    Ok(())
}

/// Open a named Cache API store.
async fn open_named_cache(cache_name: &str) -> Result<web_sys::Cache, OrigaError> {
    let window = web_sys::window().ok_or_else(|| OrigaError::RepositoryError {
        reason: "No window found".to_string(),
    })?;

    let caches = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("caches"))
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Cache API not available: {e:?}"),
        })?;

    let caches: web_sys::CacheStorage =
        caches.dyn_into().map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to cast CacheStorage: {e:?}"),
        })?;

    let cache = JsFuture::from(caches.open(cache_name))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to open cache '{cache_name}': {e:?}"),
        })?
        .into();

    Ok(cache)
}
