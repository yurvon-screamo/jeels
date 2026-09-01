use std::collections::HashMap;

use origa::domain::OrigaError;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;

#[cfg(target_arch = "wasm32")]
use super::cdn_provider::{CDN_CACHE_NAME, cdn_cache_url};
#[cfg(target_arch = "wasm32")]
use super::dictionary_cache::{DICTIONARY_FILES_CACHE_NAME, VOCABULARY_CACHE_NAME};
#[cfg(target_arch = "wasm32")]
const MANIFEST_CACHE_KEY: &str = "__origa_cache_manifest__";

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheManifest {
    pub version: u32,
    pub files: HashMap<String, String>,
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn check_and_invalidate() -> Result<(), OrigaError> {
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn check_and_invalidate() -> Result<(), OrigaError> {
    let remote = match fetch_remote_manifest().await {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!(error = ?e, "Failed to fetch remote manifest, skipping invalidation");
            return Ok(());
        },
    };

    let cache = open_cdn_cache().await?;

    let local = get_local_manifest(&cache).await;

    let Some(local) = local else {
        let all_paths: Vec<String> = remote.files.keys().cloned().collect();
        invalidate_stale_entries(&cache, &all_paths).await;

        let has_dict = all_paths.iter().any(|p| p.starts_with("dictionaries/"));
        if has_dict {
            if let Err(e) = invalidate_dictionary_files().await {
                tracing::warn!(error = ?e, "Failed to invalidate dictionary file cache");
            }
        }

        let has_vocab = all_paths.iter().any(|p| p.starts_with("dictionary/chunk_"));
        if has_vocab {
            if let Err(e) = invalidate_cached_vocabulary().await {
                tracing::warn!(error = ?e, "Failed to invalidate cached vocabulary");
            }
        }

        save_local_manifest(&cache, &remote).await?;
        tracing::info!(
            invalidated = all_paths.len(),
            "First run — cleared pre-manifest cache entries"
        );
        return Ok(());
    };

    let stale = find_stale_entries(&local, &remote);

    if stale.is_empty() {
        save_local_manifest(&cache, &remote).await?;
        tracing::debug!("No stale entries, manifest updated");
        return Ok(());
    }

    invalidate_stale_entries(&cache, &stale).await;

    let has_dict_stale = stale.iter().any(|p| p.starts_with("dictionaries/"));
    if has_dict_stale {
        if let Err(e) = invalidate_dictionary_files().await {
            tracing::warn!(error = ?e, "Failed to invalidate dictionary file cache");
        }
    }

    let has_vocab_stale = stale.iter().any(|p| p.starts_with("dictionary/chunk_"));
    if has_vocab_stale {
        if let Err(e) = invalidate_cached_vocabulary().await {
            tracing::warn!(error = ?e, "Failed to invalidate cached vocabulary");
        }
    }

    save_local_manifest(&cache, &remote).await?;

    tracing::info!(stale_count = stale.len(), "Cache invalidated");

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn build_manifest_url() -> String {
    let base = env!("ORIGA_CDN_BASE_URL").trim_end_matches('/');
    let d = js_sys::Date::new_0();
    let date = format!(
        "{:04}{:02}{:02}",
        d.get_full_year(),
        d.get_month() + 1,
        d.get_date()
    );
    format!("{}/manifest.json?t={}", base, date)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_remote_manifest() -> Result<CacheManifest, OrigaError> {
    let url = build_manifest_url();

    let window = web_sys::window().ok_or_else(|| OrigaError::RepositoryError {
        reason: "No window found".to_string(),
    })?;

    let resp_value = JsFuture::from(window.fetch_with_str(&url))
        .await
        .map_err(|e| OrigaError::NetworkError {
            url: url.clone(),
            reason: format!("Network error: {:?}", e),
        })?;

    let response: web_sys::Response =
        resp_value
            .dyn_into()
            .map_err(|e| OrigaError::NetworkError {
                url: url.clone(),
                reason: format!("Failed to cast response: {:?}", e),
            })?;

    if response.status() == 404 {
        return Err(OrigaError::RepositoryError {
            reason: "Manifest not found (404)".to_string(),
        });
    }

    if !response.ok() {
        return Err(OrigaError::NetworkError {
            url: url.clone(),
            reason: format!("HTTP {}", response.status()),
        });
    }

    let text = JsFuture::from(response.text().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to get text() promise: {:?}", e),
    })?)
    .await
    .map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to read response text: {:?}", e),
    })?;

    let text_str = text
        .as_string()
        .ok_or_else(|| OrigaError::RepositoryError {
            reason: "Response text is not a string".to_string(),
        })?;

    serde_json::from_str(&text_str).map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to parse manifest JSON: {:?}", e),
    })
}

#[cfg(target_arch = "wasm32")]
async fn open_cdn_cache() -> Result<web_sys::Cache, OrigaError> {
    let window = web_sys::window().ok_or_else(|| OrigaError::RepositoryError {
        reason: "No window found".to_string(),
    })?;

    let caches = window.caches().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Cache API not available: {:?}", e),
    })?;

    let cache = JsFuture::from(caches.open(CDN_CACHE_NAME))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to open CDN cache: {:?}", e),
        })?;

    cache.dyn_into().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to cast Cache: {:?}", e),
    })
}

#[cfg(target_arch = "wasm32")]
async fn get_local_manifest(cache: &web_sys::Cache) -> Option<CacheManifest> {
    let result = JsFuture::from(cache.match_with_str(&cdn_cache_url(MANIFEST_CACHE_KEY)))
        .await
        .ok()?;

    if result.is_null() || result.is_undefined() {
        return None;
    }

    let response: web_sys::Response = result.dyn_into().ok()?;
    if !response.ok() {
        return None;
    }

    let text = JsFuture::from(response.text().ok()?).await.ok()?;
    let text_str = text.as_string()?;

    serde_json::from_str(&text_str)
        .map_err(|e| {
            tracing::warn!(error = ?e, "Failed to parse local manifest");
            e
        })
        .ok()
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn find_stale_entries(local: &CacheManifest, remote: &CacheManifest) -> Vec<String> {
    local
        .files
        .iter()
        .filter(|(path, local_hash)| match remote.files.get(*path) {
            Some(remote_hash) => *local_hash != remote_hash,
            None => true,
        })
        .map(|(path, _)| path.clone())
        .collect()
}

#[cfg(target_arch = "wasm32")]
async fn invalidate_stale_entries(cache: &web_sys::Cache, stale_paths: &[String]) {
    for path in stale_paths {
        match JsFuture::from(cache.delete_with_str(&cdn_cache_url(path))).await {
            Ok(result) => {
                let deleted = result.is_truthy();
                tracing::debug!(path = %path, deleted = deleted, "Cache entry invalidation");
            },
            Err(e) => {
                tracing::warn!(path = %path, error = ?e, "Failed to delete cache entry");
            },
        }
    }
}

/// Invalidate the raw dictionary-file cache (introduced when the app switched
/// to raw lindera dictionary files) so the next
/// load fetches fresh files from the CDN.
#[cfg(target_arch = "wasm32")]
async fn invalidate_dictionary_files() -> Result<(), OrigaError> {
    let window = web_sys::window().ok_or_else(|| OrigaError::RepositoryError {
        reason: "No window found".to_string(),
    })?;

    let caches = window.caches().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Cache API not available: {:?}", e),
    })?;

    let _ = JsFuture::from(caches.delete(DICTIONARY_FILES_CACHE_NAME)).await;

    tracing::info!("Invalidated dictionary file cache");
    Ok(())
}

/// Invalidate the cached VocabularyDatabase entry so the next load fetches
/// fresh JSON chunks from CDN and re-parses.
#[cfg(target_arch = "wasm32")]
async fn invalidate_cached_vocabulary() -> Result<(), OrigaError> {
    let window = web_sys::window().ok_or_else(|| OrigaError::RepositoryError {
        reason: "No window found".to_string(),
    })?;

    let caches = window.caches().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Cache API not available: {:?}", e),
    })?;

    let cache = JsFuture::from(caches.open(VOCABULARY_CACHE_NAME))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to open vocabulary cache: {:?}", e),
        })?;

    let cache: web_sys::Cache = cache.dyn_into().map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to cast Cache: {:?}", e),
    })?;

    let cache_key =
        super::cdn_provider::cdn_cache_url(super::dictionary_cache::VOCABULARY_CACHE_KEY);
    JsFuture::from(cache.delete_with_str(&cache_key))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to delete vocabulary cache: {:?}", e),
        })?;

    tracing::info!("Invalidated cached vocabulary");
    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn save_local_manifest(
    cache: &web_sys::Cache,
    manifest: &CacheManifest,
) -> Result<(), OrigaError> {
    let json = serde_json::to_string(manifest).map_err(|e| OrigaError::RepositoryError {
        reason: format!("Failed to serialize manifest: {:?}", e),
    })?;

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&wasm_bindgen::JsValue::from_str(&json));

    let blob_property_bag = web_sys::BlobPropertyBag::new();
    blob_property_bag.set_type("application/json");

    let blob = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &blob_property_bag)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create manifest blob: {:?}", e),
        })?;

    let response_init = web_sys::ResponseInit::new();
    response_init.set_status(200);
    response_init.set_status_text("OK");

    let response = web_sys::Response::new_with_opt_blob_and_init(Some(&blob), &response_init)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to create manifest response: {:?}", e),
        })?;

    let request =
        web_sys::Request::new_with_str(&cdn_cache_url(MANIFEST_CACHE_KEY)).map_err(|e| {
            OrigaError::RepositoryError {
                reason: format!("Failed to create manifest request: {:?}", e),
            }
        })?;

    JsFuture::from(cache.put_with_request(&request, &response))
        .await
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("Failed to save manifest to cache: {:?}", e),
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_stale_entries_detects_changed_hash() {
        let mut local_files = HashMap::new();
        local_files.insert("a.json".to_string(), "hash_a_v1".to_string());
        local_files.insert("b.json".to_string(), "hash_b_v1".to_string());

        let mut remote_files = HashMap::new();
        remote_files.insert("a.json".to_string(), "hash_a_v2".to_string());
        remote_files.insert("b.json".to_string(), "hash_b_v1".to_string());

        let local = CacheManifest {
            version: 1,
            files: local_files,
        };
        let remote = CacheManifest {
            version: 2,
            files: remote_files,
        };

        let stale = find_stale_entries(&local, &remote);
        assert_eq!(stale, vec!["a.json"]);
    }

    #[test]
    fn find_stale_entries_detects_removed_path() {
        let mut local_files = HashMap::new();
        local_files.insert("a.json".to_string(), "hash_a".to_string());
        local_files.insert("removed.json".to_string(), "hash_r".to_string());

        let mut remote_files = HashMap::new();
        remote_files.insert("a.json".to_string(), "hash_a".to_string());

        let local = CacheManifest {
            version: 1,
            files: local_files,
        };
        let remote = CacheManifest {
            version: 2,
            files: remote_files,
        };

        let stale = find_stale_entries(&local, &remote);
        assert_eq!(stale, vec!["removed.json"]);
    }

    #[test]
    fn find_stale_entries_empty_when_no_changes() {
        let mut files = HashMap::new();
        files.insert("a.json".to_string(), "hash_a".to_string());

        let local = CacheManifest {
            version: 1,
            files: files.clone(),
        };
        let remote = CacheManifest { version: 2, files };

        let stale = find_stale_entries(&local, &remote);
        assert!(stale.is_empty());
    }

    #[test]
    fn find_stale_entries_empty_when_local_is_empty() {
        let local = CacheManifest {
            version: 1,
            files: HashMap::new(),
        };
        let mut remote_files = HashMap::new();
        remote_files.insert("a.json".to_string(), "hash_a".to_string());

        let remote = CacheManifest {
            version: 2,
            files: remote_files,
        };

        let stale = find_stale_entries(&local, &remote);
        assert!(stale.is_empty());
    }
}
