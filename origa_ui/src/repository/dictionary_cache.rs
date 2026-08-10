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
/// Separate from RKYV_CACHE_NAME (which stores raw DictionaryData) so old
/// entries don't interfere.
pub const LINDERA_CACHE_NAME: &str = "origa-dictionary-lindera-v1";

/// Cache name for pre-parsed VocabularyDatabase (rkyv).
pub const VOCABULARY_CACHE_NAME: &str = "origa-vocabulary-rkyv-v1";

/// Cache key for CachedLinderaDictionary blob.
pub const LINDERA_CACHE_KEY: &str = "/__origa_lindera_cached__";

/// Cache key for VocabularyDatabase rkyv blob.
pub const VOCABULARY_CACHE_KEY: &str = "/__origa_vocabulary_cached__";

/// Get cached lindera structures as raw rkyv bytes.
///
/// Uses a separate cache (`LINDERA_CACHE_NAME`) from the raw dictionary
/// cache (`RKYV_CACHE_NAME`).
pub async fn get_cached_lindera_rkyv() -> Result<Option<Vec<u8>>, OrigaError> {
    let start = now_ms();
    let cache = open_lindera_cache().await?;

    let match_promise = cache.match_with_str(&cdn_cache_url(LINDERA_CACHE_KEY));
    let response_option =
        JsFuture::from(match_promise)
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to check lindera cache: {:?}", e),
            })?;

    if response_option.is_undefined() || response_option.is_null() {
        return Ok(None);
    }

    let response: web_sys::Response =
        response_option
            .dyn_into()
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to cast lindera cache response: {:?}", e),
            })?;

    if !response.ok() {
        return Ok(None);
    }

    let array_buffer_promise =
        response
            .array_buffer()
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to get array_buffer promise: {:?}", e),
            })?;

    let array_buffer =
        JsFuture::from(array_buffer_promise)
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to read lindera cache: {:?}", e),
            })?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let bytes = uint8_array.to_vec();
    tracing::info!(
        "Lindera cache loaded: {} bytes ({:.2}s)",
        bytes.len(),
        (now_ms() - start) / 1000.0
    );
    Ok(Some(bytes))
}

/// Save pre-built lindera structures (rkyv bytes) to the Cache API.
pub async fn save_lindera_to_cache_rkyv(bytes: &[u8]) -> Result<(), OrigaError> {
    let start = now_ms();
    let cache = open_lindera_cache().await?;

    let array_buffer = js_sys::ArrayBuffer::new(bytes.len() as u32);
    let view = js_sys::Uint8Array::new(&array_buffer);
    view.copy_from(bytes);

    let response_init = web_sys::ResponseInit::new();
    response_init.set_status(200);
    response_init.set_status_text("OK");

    let blob_property_bag = web_sys::BlobPropertyBag::new();
    blob_property_bag.set_type("application/octet-stream");

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&array_buffer);

    let blob =
        web_sys::Blob::new_with_buffer_source_sequence_and_options(&blob_parts, &blob_property_bag)
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to create lindera cache blob: {:?}", e),
            })?;

    let response = web_sys::Response::new_with_opt_blob_and_init(Some(&blob), &response_init)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create lindera cache response: {:?}", e),
        })?;

    let put_promise = cache.put_with_str(&cdn_cache_url(LINDERA_CACHE_KEY), &response);
    JsFuture::from(put_promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to save lindera cache: {:?}", e),
        })?;
    tracing::info!(
        "Lindera cache saved: {} bytes ({:.2}s)",
        bytes.len(),
        (now_ms() - start) / 1000.0
    );

    Ok(())
}

async fn open_lindera_cache() -> Result<web_sys::Cache, OrigaError> {
    let window = web_sys::window().ok_or_else(|| OrigaError::RepositoryError {
        reason: "No window found".to_string(),
    })?;

    let caches = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("caches"))
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Cache API not available: {:?}", e),
        })?;

    let caches: web_sys::CacheStorage =
        caches.dyn_into().map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to cast CacheStorage: {:?}", e),
        })?;

    let cache_promise = caches.open(LINDERA_CACHE_NAME);
    let cache = open_cache_from_promise(cache_promise).await?;

    Ok(cache)
}

/// Get cached VocabularyDatabase as raw rkyv bytes.
pub async fn get_cached_vocabulary_rkyv() -> Result<Option<Vec<u8>>, OrigaError> {
    let cache = open_vocabulary_cache().await?;

    let match_promise = cache.match_with_str(&cdn_cache_url(VOCABULARY_CACHE_KEY));
    let response_option =
        JsFuture::from(match_promise)
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to check vocabulary cache: {:?}", e),
            })?;

    if response_option.is_undefined() || response_option.is_null() {
        return Ok(None);
    }

    let response: web_sys::Response =
        response_option
            .dyn_into()
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to cast vocabulary cache response: {:?}", e),
            })?;

    if !response.ok() {
        return Ok(None);
    }

    let array_buffer_promise =
        response
            .array_buffer()
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to get array_buffer promise: {:?}", e),
            })?;

    let array_buffer =
        JsFuture::from(array_buffer_promise)
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to read vocabulary cache: {:?}", e),
            })?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let bytes = uint8_array.to_vec();
    tracing::debug!("Vocabulary cache loaded: {} bytes", bytes.len());
    Ok(Some(bytes))
}

/// Save pre-parsed VocabularyDatabase (rkyv bytes) to the Cache API.
pub async fn save_vocabulary_to_cache_rkyv(bytes: &[u8]) -> Result<(), OrigaError> {
    let cache = open_vocabulary_cache().await?;

    let array_buffer = js_sys::ArrayBuffer::new(bytes.len() as u32);
    let view = js_sys::Uint8Array::new(&array_buffer);
    view.copy_from(bytes);

    let response_init = web_sys::ResponseInit::new();
    response_init.set_status(200);
    response_init.set_status_text("OK");

    let blob_property_bag = web_sys::BlobPropertyBag::new();
    blob_property_bag.set_type("application/octet-stream");

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&array_buffer);

    let blob =
        web_sys::Blob::new_with_buffer_source_sequence_and_options(&blob_parts, &blob_property_bag)
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to create vocabulary cache blob: {:?}", e),
            })?;

    let response = web_sys::Response::new_with_opt_blob_and_init(Some(&blob), &response_init)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create vocabulary cache response: {:?}", e),
        })?;

    let put_promise = cache.put_with_str(&cdn_cache_url(VOCABULARY_CACHE_KEY), &response);
    JsFuture::from(put_promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to save vocabulary cache: {:?}", e),
        })?;

    Ok(())
}

async fn open_vocabulary_cache() -> Result<web_sys::Cache, OrigaError> {
    let window = web_sys::window().ok_or_else(|| OrigaError::RepositoryError {
        reason: "No window found".to_string(),
    })?;

    let caches = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("caches"))
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Cache API not available: {:?}", e),
        })?;

    let caches: web_sys::CacheStorage =
        caches.dyn_into().map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to cast CacheStorage: {:?}", e),
        })?;

    let cache_promise = caches.open(VOCABULARY_CACHE_NAME);
    let cache = open_cache_from_promise(cache_promise).await?;

    Ok(cache)
}

/// Shared helper: await a `caches.open()` promise and cast to `web_sys::Cache`.
async fn open_cache_from_promise(promise: js_sys::Promise) -> Result<web_sys::Cache, OrigaError> {
    let cache = JsFuture::from(promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to open cache: {:?}", e),
        })?
        .into();
    Ok(cache)
}
