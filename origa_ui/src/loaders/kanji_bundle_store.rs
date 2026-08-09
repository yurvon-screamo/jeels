//! In-memory store for kanji SVG JLPT bundles.
//!
//! Loaded lazily when a kanji card needs an SVG. Shared between
//! `card_precache_loader` (which decides whether to CDN-fetch) and
//! `kanji_animation` (which renders the SVG).

use std::collections::HashMap;
use std::sync::OnceLock;

use origa::domain::OrigaError;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum KanjiBundleType {
    Animations,
    Frames,
}

impl KanjiBundleType {
    fn cdn_prefix(&self) -> &'static str {
        match self {
            Self::Animations => "kanji_animations",
            Self::Frames => "kanji_frames",
        }
    }
}

/// JLPT level normalised to lowercase for CDN path matching.
fn normalize_jlpt(jlpt: &str) -> &str {
    match jlpt {
        "N5" | "n5" => "n5",
        "N4" | "n4" => "n4",
        "N3" | "n3" => "n3",
        "N2" | "n2" => "n2",
        _ => "n1",
    }
}

/// Map (bundle_type, level) → (kanji → SVG).
type Store = HashMap<(KanjiBundleType, String), HashMap<String, String>>;

static STORE: OnceLock<std::sync::Mutex<Store>> = OnceLock::new();

fn store() -> &'static std::sync::Mutex<Store> {
    STORE.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

/// Check if a specific kanji's SVG is already loaded in memory.
pub fn get_svg(bundle_type: KanjiBundleType, jlpt: &str, kanji: &str) -> Option<String> {
    let level = normalize_jlpt(jlpt).to_string();
    let s = store().lock().ok()?;
    s.get(&(bundle_type, level))?.get(kanji).cloned()
}

/// Load a JLPT bundle from CDN into memory. No-op if already loaded.
pub async fn load_bundle(
    bundle_type: KanjiBundleType,
    jlpt: &str,
) -> Result<(), OrigaError> {
    let level = normalize_jlpt(jlpt).to_string();

    // Check if already loaded
    {
        let s = store().lock().map_err(|e| OrigaError::RepositoryError {
            reason: format!("Kanji store lock: {:?}", e),
        })?;
        if s.contains_key(&(bundle_type, level.clone())) {
            return Ok(());
        }
    }

    // Fetch from CDN
    let path = format!("{}_{}.json", bundle_type.cdn_prefix(), level);
    let cdn = crate::repository::cdn_provider();
    let json = cdn.fetch_text(&path).await?;

    // Parse
    let bundle: HashMap<String, String> = serde_json::from_str(&json).map_err(|e| {
        OrigaError::RepositoryError {
            reason: format!("Failed to parse kanji bundle {}: {}", path, e),
        }
    })?;

    let chunk_count = bundle.len();

    // Store
    {
        let mut s = store().lock().map_err(|e| OrigaError::RepositoryError {
            reason: format!("Kanji store lock: {:?}", e),
        })?;
        s.insert((bundle_type, level), bundle);
    }

    tracing::info!(
        bundle_type = ?bundle_type,
        kanji_count = chunk_count,
        "Loaded kanji JLPT bundle"
    );

    Ok(())
}

/// Check if both bundle types for a JLPT level are loaded.
pub fn is_level_loaded(jlpt: &str) -> bool {
    let level = normalize_jlpt(jlpt).to_string();
    let s = match store().lock() {
        Ok(s) => s,
        Err(_) => return false,
    };
    s.contains_key(&(KanjiBundleType::Animations, level.clone()))
        && s.contains_key(&(KanjiBundleType::Frames, level))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_type_cdn_prefix() {
        assert_eq!(KanjiBundleType::Animations.cdn_prefix(), "kanji_animations");
        assert_eq!(KanjiBundleType::Frames.cdn_prefix(), "kanji_frames");
    }

    #[test]
    fn normalize_jlpt_levels() {
        assert_eq!(normalize_jlpt("N5"), "n5");
        assert_eq!(normalize_jlpt("N1"), "n1");
        assert_eq!(normalize_jlpt("n3"), "n3");
        assert_eq!(normalize_jlpt("unknown"), "n1");
    }
}
