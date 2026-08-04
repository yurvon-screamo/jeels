use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use origa::domain::{DictionaryData, OrigaError};

use super::cdn_provider::cdn_cache_url;
use crate::core::config::urls;

pub const RKYV_CACHE_NAME: &str = "origa-dictionary-rkyv-v2";

async fn open_cache() -> Result<web_sys::Cache, OrigaError> {
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

    let cache_promise = caches.open(RKYV_CACHE_NAME);
    let cache: web_sys::Cache = JsFuture::from(cache_promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to open cache: {:?}", e),
        })?
        .into();

    Ok(cache)
}

use crate::utils::now_ms;

/// Get cached dictionary as raw rkyv bytes
pub async fn get_cached_dictionary_rkyv() -> Result<Option<Vec<u8>>, OrigaError> {
    let start = now_ms();
    let cache = open_cache().await?;
    tracing::debug!("Cache opened ({:.2}s)", (now_ms() - start) / 1000.0);

    let url = urls().dictionary;

    let match_start = now_ms();
    let match_promise = cache.match_with_str(&cdn_cache_url(url));
    let response_option =
        JsFuture::from(match_promise)
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to check cache: {:?}", e),
            })?;
    tracing::debug!(
        "Cache match checked ({:.2}s)",
        (now_ms() - match_start) / 1000.0
    );

    if response_option.is_undefined() || response_option.is_null() {
        return Ok(None);
    }

    let response: web_sys::Response =
        response_option
            .dyn_into()
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to cast Response: {:?}", e),
            })?;

    if !response.ok() {
        return Ok(None);
    }

    let _read_start = now_ms();
    let array_buffer_promise =
        response
            .array_buffer()
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to get array buffer: {:?}", e),
            })?;

    let array_buffer =
        JsFuture::from(array_buffer_promise)
            .await
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to read array buffer: {:?}", e),
            })?;

    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let bytes = uint8_array.to_vec();
    tracing::info!(
        "Cache loaded: {} ({} bytes, {:.2}s)",
        url,
        bytes.len(),
        (now_ms() - start) / 1000.0
    );
    Ok(Some(bytes))
}

/// Save dictionary as rkyv bytes to cache
pub async fn save_dictionary_to_cache_rkyv(data: &DictionaryData) -> Result<(), OrigaError> {
    let start = now_ms();

    let serialize_start = now_ms();
    let bytes = origa::domain::serialize_dictionary_to_rkyv(data).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("Failed to serialize dictionary: {:?}", e),
        }
    })?;
    tracing::info!(
        "Dictionary serialized rkyv ({} bytes, {:.2}s)",
        bytes.len(),
        (now_ms() - serialize_start) / 1000.0
    );

    let cache = open_cache().await?;
    tracing::debug!(
        "Cache opened for save ({:.2}s)",
        (now_ms() - start) / 1000.0
    );

    let array_buffer = js_sys::ArrayBuffer::new(bytes.len() as u32);
    let view = js_sys::Uint8Array::new(&array_buffer);
    view.copy_from(&bytes);

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
                reason: format!("Failed to create blob: {:?}", e),
            })?;

    let response = web_sys::Response::new_with_opt_blob_and_init(Some(&blob), &response_init)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create response: {:?}", e),
        })?;

    let url = urls().dictionary;
    let put_start = now_ms();
    let put_promise = cache.put_with_str(&cdn_cache_url(url), &response);
    JsFuture::from(put_promise)
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to save to cache: {:?}", e),
        })?;
    tracing::info!(
        "Cache saved: {} (put {:.2}s, total {:.2}s)",
        url,
        (now_ms() - put_start) / 1000.0,
        (now_ms() - start) / 1000.0
    );

    Ok(())
}
