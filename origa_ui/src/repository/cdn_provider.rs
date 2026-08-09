use std::collections::HashMap;
use std::collections::VecDeque;
use std::future::Future;
use std::sync::Mutex;
use std::sync::OnceLock;

use leptos::wasm_bindgen::JsCast;
use origa::domain::OrigaError;
use origa::traits::CdnProvider;
use wasm_bindgen_futures::JsFuture;

use crate::core::config::cdn_url;

pub const CDN_CACHE_NAME: &str = "origa-cdn-v1";

pub struct CacheFirstCdnProvider;

static CDN_PROVIDER: OnceLock<CacheFirstCdnProvider> = OnceLock::new();

pub fn cdn() -> &'static CacheFirstCdnProvider {
    CDN_PROVIDER.get_or_init(|| CacheFirstCdnProvider)
}

static BLOB_URL_CACHE: OnceLock<Mutex<BlobUrlCache>> = OnceLock::new();

/// Ceiling on the number of retained `blob:` URLs. Each entry holds an
/// underlying `Blob` + `ArrayBuffer` alive in JS heap until the URL is
/// `revoke_object_url`'d. Bounded to keep memory predictable on low-end
/// Android WebViews: ~500 phrases × ~3-50 KB/opus ≈ 1.5-25 MB ceiling. When
/// the limit is reached the oldest entry is evicted and its blob URL revoked.
const MAX_BLOB_URL_CACHE_SIZE: usize = 500;

/// Insertion-ordered blob URL cache with bounded LRU-ish (FIFO) eviction.
/// Tracks keys in `order` so the oldest materialised URL can be revoked when
/// the cap is reached, preventing unbounded `Blob`/`ArrayBuffer` leaks.
struct BlobUrlCache {
    urls: HashMap<String, String>,
    order: VecDeque<String>,
}

impl BlobUrlCache {
    fn get(&self, key: &str) -> Option<&String> {
        self.urls.get(key)
    }

    fn insert(&mut self, key: String, url: String) {
        if self.urls.insert(key.clone(), url).is_some() {
            return;
        }
        while self.order.len() >= MAX_BLOB_URL_CACHE_SIZE {
            let Some(evicted_key) = self.order.pop_front() else {
                break;
            };
            if let Some(evicted_url) = self.urls.remove(&evicted_key) {
                let _ = web_sys::Url::revoke_object_url(&evicted_url);
            }
        }
        self.order.push_back(key);
    }

    #[cfg(test)]
    fn remove(&mut self, key: &str) {
        if self.urls.remove(key).is_some() {
            self.order.retain(|k| k != key);
        }
    }
}

fn blob_url_cache() -> &'static Mutex<BlobUrlCache> {
    BLOB_URL_CACHE.get_or_init(|| {
        Mutex::new(BlobUrlCache {
            urls: HashMap::new(),
            order: VecDeque::new(),
        })
    })
}

async fn open_cache() -> Result<web_sys::Cache, OrigaError> {
    let window = web_sys::window().ok_or_else(|| OrigaError::RepositoryError {
        reason: "No window found".to_string(),
    })?;

    let caches = window.caches().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Cache API not available: {:?}", e),
    })?;

    let cache = JsFuture::from(caches.open(CDN_CACHE_NAME))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to open cache: {:?}", e),
        })?;

    cache.dyn_into().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to cast Cache: {:?}", e),
    })
}

/// Store a simple marker value in the CDN cache.
pub async fn store_cache_marker(key: &str, value: &str) -> Result<(), OrigaError> {
    let cache = open_cache().await?;
    let response = web_sys::Response::new_with_opt_str(Some(value)).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("Failed to create marker response: {:?}", e),
        }
    })?;
    let url = cdn_cache_url(key);
    let request =
        web_sys::Request::new_with_str(&url).map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create marker request: {:?}", e),
        })?;
    JsFuture::from(cache.put_with_request(&request, &response))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to store marker: {:?}", e),
        })?;
    Ok(())
}

fn ensure_leading_slash(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    }
}

/// Absolute URL used as the Cache API key for `path`.
///
/// The Cache API rejects request URLs that are not HTTP(S): a relative path
/// resolves against the WebView origin (`tauri://localhost` on iOS and desktop
/// production builds) and WebKit throws `Request url is not HTTP/HTTPS`. Keys
/// are therefore always built on the full CDN base.
///
/// Unlike [`crate::core::config::cdn_url`], preserves any `?v=` query string —
/// phrase chunks and versioned content use it as a content hash, so it must be
/// part of the cache key or a content change would keep serving the stale entry.
pub(crate) fn cdn_cache_url(path: &str) -> String {
    let base = env!("ORIGA_CDN_BASE_URL").trim_end_matches('/');
    format!("{}{}", base, ensure_leading_slash(path))
}

async fn get_text_from_cache(cache: &web_sys::Cache, path: &str) -> Option<String> {
    let url = cdn_cache_url(path);
    let result = JsFuture::from(cache.match_with_str(&url)).await.ok()?;
    if result.is_null() || result.is_undefined() {
        return None;
    }
    let response: web_sys::Response = result.dyn_into().ok()?;
    if !response.ok() {
        return None;
    }
    let text = JsFuture::from(response.text().ok()?).await.ok()?;
    text.as_string()
}

async fn get_bytes_from_cache(cache: &web_sys::Cache, path: &str) -> Option<Vec<u8>> {
    let url = cdn_cache_url(path);
    let result = JsFuture::from(cache.match_with_str(&url)).await.ok()?;
    if result.is_null() || result.is_undefined() {
        return None;
    }
    let response: web_sys::Response = result.dyn_into().ok()?;
    if !response.ok() {
        return None;
    }
    let ab = JsFuture::from(response.array_buffer().ok()?).await.ok()?;
    let arr = js_sys::Uint8Array::new(&ab);
    Some(arr.to_vec())
}

async fn save_response_to_cache(
    cache: &web_sys::Cache,
    path: &str,
    response: &web_sys::Response,
) -> Result<(), OrigaError> {
    let url = cdn_cache_url(path);
    let request =
        web_sys::Request::new_with_str(&url).map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create cache request: {:?}", e),
        })?;

    JsFuture::from(cache.put_with_request(&request, response))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to save to cache: {:?}", e),
        })?;

    Ok(())
}

async fn fetch_text_from_cdn(path: &str) -> Result<(web_sys::Response, String), OrigaError> {
    let url = cdn_url(&ensure_leading_slash(path));

    let window = web_sys::window().ok_or_else(|| OrigaError::NetworkError {
        url: url.clone(),
        reason: "No window found".to_string(),
    })?;

    let resp_value = JsFuture::from(window.fetch_with_str(&url))
        .await
        .map_err(|e| OrigaError::NetworkError {
            url: url.clone(),
            reason: format!("Failed to fetch: {:?}", e),
        })?;

    let response: web_sys::Response =
        resp_value
            .dyn_into()
            .map_err(|e| OrigaError::NetworkError {
                url: url.clone(),
                reason: format!("Failed to cast response: {:?}", e),
            })?;

    if !response.ok() {
        return Err(OrigaError::NetworkError {
            url: url.clone(),
            reason: format!("HTTP {}", response.status()),
        });
    }

    let cloned = response.clone().map_err(|e| OrigaError::NetworkError {
        url: url.clone(),
        reason: format!("Failed to clone response: {:?}", e),
    })?;

    let text = JsFuture::from(cloned.text().map_err(|e| OrigaError::NetworkError {
        url: url.clone(),
        reason: format!("Failed to get text promise: {:?}", e),
    })?)
    .await
    .map_err(|e| OrigaError::NetworkError {
        url: url.clone(),
        reason: format!("Failed to read text: {:?}", e),
    })?;

    let text_str = text.as_string().ok_or_else(|| OrigaError::NetworkError {
        url: url.clone(),
        reason: "Response is not a string".to_string(),
    })?;

    Ok((response, text_str))
}

async fn fetch_bytes_from_cdn(path: &str) -> Result<(web_sys::Response, Vec<u8>), OrigaError> {
    let url = cdn_url(&ensure_leading_slash(path));

    let window = web_sys::window().ok_or_else(|| OrigaError::NetworkError {
        url: url.clone(),
        reason: "No window found".to_string(),
    })?;

    let resp_value = JsFuture::from(window.fetch_with_str(&url))
        .await
        .map_err(|e| OrigaError::NetworkError {
            url: url.clone(),
            reason: format!("Failed to fetch: {:?}", e),
        })?;

    let response: web_sys::Response =
        resp_value
            .dyn_into()
            .map_err(|e| OrigaError::NetworkError {
                url: url.clone(),
                reason: format!("Failed to cast response: {:?}", e),
            })?;

    if !response.ok() {
        return Err(OrigaError::NetworkError {
            url: url.clone(),
            reason: format!("HTTP {}", response.status()),
        });
    }

    let cloned = response.clone().map_err(|e| OrigaError::NetworkError {
        url: url.clone(),
        reason: format!("Failed to clone response: {:?}", e),
    })?;

    let ab = JsFuture::from(
        cloned
            .array_buffer()
            .map_err(|e| OrigaError::NetworkError {
                url: url.clone(),
                reason: format!("Failed to get array_buffer promise: {:?}", e),
            })?,
    )
    .await
    .map_err(|e| OrigaError::NetworkError {
        url: url.clone(),
        reason: format!("Failed to read array_buffer: {:?}", e),
    })?;

    let arr = js_sys::Uint8Array::new(&ab);
    Ok((response, arr.to_vec()))
}

impl CdnProvider for CacheFirstCdnProvider {
    fn fetch_text(&self, path: &str) -> impl Future<Output = Result<String, OrigaError>> {
        let path = path.to_string();
        async move {
            let cache = open_cache().await?;

            if let Some(text) = get_text_from_cache(&cache, &path).await {
                tracing::debug!(path = %path, "Cache hit (text)");
                return Ok(text);
            }

            tracing::debug!(path = %path, "Cache miss, fetching from CDN");
            let (response, text) = fetch_text_from_cdn(&path).await?;

            if let Err(e) = save_response_to_cache(&cache, &path, &response).await {
                tracing::warn!(path = %path, error = ?e, "Failed to cache text response");
            }

            Ok(text)
        }
    }

    fn fetch_bytes(&self, path: &str) -> impl Future<Output = Result<Vec<u8>, OrigaError>> {
        let path = path.to_string();
        async move {
            let cache = open_cache().await?;

            if let Some(bytes) = get_bytes_from_cache(&cache, &path).await {
                tracing::debug!(path = %path, "Cache hit (bytes)");
                return Ok(bytes);
            }

            tracing::debug!(path = %path, "Cache miss, fetching from CDN");
            let (response, bytes) = fetch_bytes_from_cdn(&path).await?;

            if let Err(e) = save_response_to_cache(&cache, &path, &response).await {
                tracing::warn!(path = %path, error = ?e, "Failed to cache bytes response");
            }

            Ok(bytes)
        }
    }
}

pub fn get_cached_blob_url(path: &str) -> Option<String> {
    let key = ensure_leading_slash(path);
    let cache = blob_url_cache().lock().ok()?;
    cache.get(&key).cloned()
}

/// Derive the Blob MIME type from the CDN path extension. Hardcoding
/// `audio/opus` couples decoding to a CDN filename convention; if a non-opus
/// record ever lands on the CDN the decoder would break silently. Falling back
/// to `audio/opus` matches the current CDN content while keeping other formats
/// decodeable if they appear.
fn mime_type_for_path(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("opus") => "audio/opus",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("ogg") | Some("oga") => "audio/ogg",
        Some("m4a") | Some("mp4") | Some("aac") => "audio/mp4",
        Some("webm") => "audio/webm",
        _ => "audio/opus",
    }
}

pub async fn prefetch_blob_url(path: &str) -> Result<String, OrigaError> {
    let key = ensure_leading_slash(path);

    if let Some(blob_url) = get_cached_blob_url(&key) {
        return Ok(blob_url);
    }

    let bytes = cdn().fetch_bytes(&key).await?;

    let blob_options = web_sys::BlobPropertyBag::new();
    blob_options.set_type(mime_type_for_path(&key));

    let uint8_array = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
    uint8_array.copy_from(&bytes);
    let parts = js_sys::Array::of1(&uint8_array);
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &blob_options)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create audio Blob: {:?}", e),
        })?;

    let blob_url = web_sys::Url::create_object_url_with_blob(&blob).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("Failed to create blob URL: {:?}", e),
        }
    })?;

    {
        let mut cache = blob_url_cache()
            .lock()
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("Failed to lock blob URL cache: {:?}", e),
            })?;
        cache.insert(key, blob_url.clone());
    }

    Ok(blob_url)
}

/// Resolve a playable audio URL for the given CDN path.
///
/// # Bug A root cause (see module-level `tests` doc)
///
/// The Railway Hikari reverse proxy gzip-encodes `.opus` responses on-the-fly,
/// and Chromium's `<audio>` element does not apply transport decompression —
/// feeding the raw gzip bytes to the Opus decoder yields
/// `ERR_CONTENT_DECODING_FAILED`. The only correct way to feed `<audio>` is
/// therefore a `blob:` URL produced from `fetch()`-decompressed bytes.
///
/// # Contract
///
/// Returns `Some(blob_url)` only when the blob URL has already been materialised
/// in the in-memory cache; returns `None` otherwise. Callers MUST NOT
/// substitute the CDN URL when this returns `None`: they must either await
/// `prefetch_blob_url` directly (preferred for playback paths) or fall back to
/// TTS. This function is a **pure synchronous lookup** with no side effects —
/// callers own the async prefetch, which avoids a double-fetch race when a
/// consumer needs the blob URL reactively.
pub fn resolve_audio_url(path: &str) -> Option<String> {
    let key = ensure_leading_slash(path);
    get_cached_blob_url(&key)
}

pub async fn prefetch_to_cache(path: &str) -> Result<(), OrigaError> {
    let cache = open_cache().await?;
    let key = ensure_leading_slash(path);

    let existing = JsFuture::from(cache.match_with_str(&cdn_cache_url(&key)))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Cache match failed: {:?}", e),
        })?;
    if !existing.is_null() && !existing.is_undefined() {
        return Ok(());
    }

    let url = cdn_url(&key);
    let window = web_sys::window().ok_or_else(|| OrigaError::NetworkError {
        url: url.clone(),
        reason: "No window found".to_string(),
    })?;

    let resp_value = JsFuture::from(window.fetch_with_str(&url))
        .await
        .map_err(|e| OrigaError::NetworkError {
            url: url.clone(),
            reason: format!("Failed to fetch: {:?}", e),
        })?;

    let response: web_sys::Response =
        resp_value
            .dyn_into()
            .map_err(|e| OrigaError::NetworkError {
                url: url.clone(),
                reason: format!("Failed to cast response: {:?}", e),
            })?;

    if !response.ok() {
        return Err(OrigaError::NetworkError {
            url,
            reason: format!("HTTP {}", response.status()),
        });
    }

    let request = web_sys::Request::new_with_str(&cdn_cache_url(&key)).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("Failed to create request: {:?}", e),
        }
    })?;

    JsFuture::from(cache.put_with_request(&request, &response))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to cache: {:?}", e),
        })?;

    Ok(())
}

pub async fn is_cached(path: &str) -> bool {
    let Ok(cache) = open_cache().await else {
        return false;
    };
    let key = ensure_leading_slash(path);
    let Ok(result) = JsFuture::from(cache.match_with_str(&cdn_cache_url(&key))).await else {
        return false;
    };
    !result.is_null() && !result.is_undefined()
}

/// Store arbitrary text in the CDN Cache API under the given path.
/// Used by bundle extraction: download a bundle, parse it, then store
/// individual chunks in cache so that lazy readers (phrase_data_loader)
/// get cache hits without individual CDN requests.
pub async fn store_text_in_cache(path: &str, text: &str) -> Result<(), OrigaError> {
    let cache = open_cache().await?;
    let key = ensure_leading_slash(path);
    let response = web_sys::Response::new_with_opt_str(Some(text)).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("Failed to create response for cache store: {:?}", e),
        }
    })?;
    let request = web_sys::Request::new_with_str(&cdn_cache_url(&key)).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("Failed to create request: {:?}", e),
        }
    })?;
    JsFuture::from(cache.put_with_request(&request, &response))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to store in cache: {:?}", e),
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Bug A — root cause (proven via curl by the architect):
    //
    // Railway's Hikari reverse proxy applies `Content-Encoding: gzip` to `.opus`
    // responses on-the-fly whenever the client advertises `Accept-Encoding` (every
    // browser / WebView does this automatically; it is a "forbidden header name"
    // and cannot be disabled from JS).
    //
    //   curl -H 'Accept-Encoding: identity'  → no Content-Encoding, 3139 bytes ✅
    //   curl -H 'Accept-Encoding: gzip'      → Content-Encoding: gzip, 3145 bytes ❌
    //   curl -H 'Accept-Encoding: br'        → Content-Encoding: gzip (!), 3145 bytes ❌
    //
    // Gzip makes the file LARGER (3145 > 3139) — negative compression for an
    // already-compressed Opus stream. Chromium's <audio> element historically
    // does not apply transport-layer decompression (crbug 1029876 / 596361 /
    // 1247420), so it hands the raw gzip bytes to the media decoder which then
    // fails with `ERR_CONTENT_DECODING_FAILED`.
    //
    // The fix is "always-blob-URL playback": the Fetch API transparently
    // decompresses `Content-Encoding`, after which the resulting bytes are
    // stored in a Blob. The `blob:` URL handed to <audio> therefore has no
    // transport encoding and decodes correctly.
    //
    // As a consequence, `resolve_audio_url` must NEVER return the raw CDN URL
    // as a synchronous fallback. The contract below is a compile-time
    // regression guard: callers must receive `Option<String>` and either await
    // the prefetched blob URL or fall back to TTS — never play the CDN URL.

    /// Compile-time contract: `resolve_audio_url` returns `Option<String>`.
    ///
    /// Before Slice-2 this function returned `String` (the CDN URL when the
    /// blob URL had not been prefetched yet). That is precisely the code path
    /// that triggered `ERR_CONTENT_DECODING_FAILED`. If this test fails to
    /// compile, the contract has regressed.
    #[test]
    fn resolve_audio_url_contract_returns_option() {
        let _type_witness: fn(&str) -> Option<String> = resolve_audio_url;
        // Touch the symbol so the assignment is not dead-code eliminated.
        let _ = _type_witness as fn(&str) -> Option<String>;
    }

    #[test]
    fn ensure_leading_slash_adds_missing() {
        assert_eq!(
            ensure_leading_slash("phrases/audio/x.opus"),
            "/phrases/audio/x.opus"
        );
    }

    #[test]
    fn ensure_leading_slash_preserves_existing() {
        assert_eq!(
            ensure_leading_slash("/phrases/audio/x.opus"),
            "/phrases/audio/x.opus"
        );
    }

    #[test]
    fn blob_url_cache_returns_none_for_uncached_key() {
        // Bug A contract: cache miss MUST surface as None to the caller so the
        // caller can fall back to TTS or await prefetch — it MUST NOT be silently
        // substituted with the raw CDN URL (see module-level root cause comment).
        let unique = "/blob-cache-probe-uncached-7e3a9f1c-e2d4-4b8a-9c11-1f5a2c7e9b8d";
        // Defensive cleanup in case a prior test populated this slot.
        if let Ok(mut cache) = blob_url_cache().lock() {
            cache.remove(unique);
        }
        let result = get_cached_blob_url(unique);
        assert!(
            result.is_none(),
            "uncached path must resolve to None; got {:?}",
            result
        );
    }

    #[test]
    fn blob_url_cache_round_trips_inserted_value() {
        let key = "/blob-cache-probe-roundtrip-3f8a1c0b-9d2e-4a6f-b117-2c4d8e1f0a3b";
        let value = "blob:roundtrip-test";
        {
            let mut cache = blob_url_cache().lock().expect("blob cache poisoned");
            cache.insert(key.to_string(), value.to_string());
        }
        assert_eq!(get_cached_blob_url(key).as_deref(), Some(value));
        // Cleanup so this test remains idempotent.
        if let Ok(mut cache) = blob_url_cache().lock() {
            cache.remove(key);
        }
    }

    #[test]
    fn mime_type_for_path_known_extensions() {
        assert_eq!(mime_type_for_path("phrases/audio/abc.opus"), "audio/opus");
        assert_eq!(mime_type_for_path("/x.mp3"), "audio/mpeg");
        assert_eq!(mime_type_for_path("/x.wav"), "audio/wav");
        assert_eq!(mime_type_for_path("/x.ogg"), "audio/ogg");
        assert_eq!(mime_type_for_path("/x.m4a"), "audio/mp4");
        assert_eq!(mime_type_for_path("/x.webm"), "audio/webm");
    }

    #[test]
    fn mime_type_for_path_unknown_defaults_to_opus() {
        // Current CDN only serves opus; the default keeps decode working for
        // the existing content while other extensions map explicitly above.
        assert_eq!(mime_type_for_path("noext"), "audio/opus");
        assert_eq!(mime_type_for_path("file.unknown"), "audio/opus");
    }

    #[test]
    fn blob_url_cache_insert_updates_existing_key_without_duplication() {
        let key = "/blob-cache-probe-update-1a2b3c4d-1111-2222-3333-444455556666";
        {
            let mut cache = blob_url_cache().lock().expect("blob cache poisoned");
            cache.insert(key.to_string(), "blob:first".to_string());
            // Re-inserting the same key must not append a duplicate order entry.
            cache.insert(key.to_string(), "blob:second".to_string());
            assert_eq!(cache.order.iter().filter(|k| *k == key).count(), 1);
        }
        assert_eq!(get_cached_blob_url(key).as_deref(), Some("blob:second"));
        if let Ok(mut cache) = blob_url_cache().lock() {
            cache.remove(key);
        }
    }
}
