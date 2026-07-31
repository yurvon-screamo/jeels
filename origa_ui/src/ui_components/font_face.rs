//! Self-hosted `@font-face` declarations for the app's typography system.
//!
//! The font files live in the canonical `cdn/fonts/` directory
//! (`<logical>-<sha8>.woff2`), enumerated at build time by `build.rs` into the
//! `FONT_FILES` table. The content-hash suffix is the cache-bust key (ADR-028):
//! a regenerated subset changes the hash, so immutable CDN edge caches can
//! never serve a stale font.
//!
//! Serving: Tauri builds serve the woff2 from the local origin so there is no
//! first-launch network round-trip and the app works offline. `build.rs` stages
//! `cdn/fonts/*.woff2` into `origa_ui/public/fonts/` (mirroring the canonical
//! source), and the existing trunk `copy-dir href="public"` then ships them at
//! `dist/fonts/`. The two-step staging exists because trunk does not accept
//! assets outside the `index.html` directory (trunk-rs/trunk#1045). Web builds
//! have no bundled fonts and resolve the same filenames on the CDN via
//! `cdn_url`, keeping `cdn/fonts/` the single source of truth.
//!
//! This runtime-injected stylesheet is the canonical `@font-face` source. The
//! `font-display` strategy is per-face: CJK faces use `block` to avoid flashing
//! wrong-glyph system CJK (Xiaomi ships Chinese forms); Latin/Cyrillic use
//! `swap` so UI text paints immediately. See ADR-028 / ADR-030.
//!
//! The stylesheet is injected imperatively into `document.head` (from the
//! `App` component body) rather than rendered through the `view!` macro.
//! The `origa_ui_bin` crate mounts the whole `<App/>` view tree and encodes
//! it as deeply nested generic tuples; adding any element to the root view
//! tips the bin past its monomorphization depth limit (`error: queries
//! overflow the depth limit!`). A `<style>` node added via the DOM API adds
//! nothing to the view tuple, so the bin keeps compiling.

use crate::core::config::cdn_url;
use crate::core::tauri;

include!(concat!(env!("OUT_DIR"), "/fonts.rs"));

const STYLE_ID: &str = "origa-font-faces";

struct FaceSpec {
    logical: &'static str,
    family: &'static str,
    weight: &'static str,
    style: &'static str,
    /// `font-display` strategy. CJK faces use `block` so the browser holds an
    /// invisible fallback rather than flashing a wrong-glyph system CJK font
    /// (Xiaomi MIUI defaults to Chinese forms) before Noto JP arrives; Latin/
    /// Cyrillic faces use `swap` so UI text paints immediately. See ADR-030.
    display: &'static str,
    unicode_range: &'static str,
}

// Cyrillic + Latin-Extended for Cormorant Garamond and IBM Plex Mono (both ship
// the glyphs; subsetting them in keeps Cyrillic UI text from falling back to a
// system font). See ADR-030.
const LATIN_CYRILLIC_RANGE: &str = "U+0000-007F,U+00A0-00FF,U+0100-017F,U+0400-052F,U+1C80-1C88,U+1E00-1EFF,U+2000-206F,U+20A0-20CF,U+20B4,U+2100-214F,U+2DE0-2DFF,U+A640-A69F";
const CJK_RANGE: &str = "U+3000-303F,U+3040-30FF,U+3400-4DBF,U+4E00-9FFF,U+F900-FAFF,U+FF00-FFEF";

const FACES: &[FaceSpec] = &[
    FaceSpec {
        logical: "cormorant-garamond",
        family: "Cormorant Garamond",
        weight: "300 700",
        style: "normal",
        display: "swap",
        unicode_range: LATIN_CYRILLIC_RANGE,
    },
    FaceSpec {
        logical: "cormorant-garamond-italic",
        family: "Cormorant Garamond",
        weight: "300 700",
        style: "italic",
        display: "swap",
        unicode_range: LATIN_CYRILLIC_RANGE,
    },
    FaceSpec {
        logical: "ibm-plex-mono-300",
        family: "IBM Plex Mono",
        weight: "300",
        style: "normal",
        display: "swap",
        unicode_range: LATIN_CYRILLIC_RANGE,
    },
    FaceSpec {
        logical: "ibm-plex-mono-400",
        family: "IBM Plex Mono",
        weight: "400",
        style: "normal",
        display: "swap",
        unicode_range: LATIN_CYRILLIC_RANGE,
    },
    FaceSpec {
        logical: "ibm-plex-mono-500",
        family: "IBM Plex Mono",
        weight: "500",
        style: "normal",
        display: "swap",
        unicode_range: LATIN_CYRILLIC_RANGE,
    },
    FaceSpec {
        logical: "ibm-plex-mono-400-italic",
        family: "IBM Plex Mono",
        weight: "400",
        style: "italic",
        display: "swap",
        unicode_range: LATIN_CYRILLIC_RANGE,
    },
    FaceSpec {
        logical: "noto-sans-jp-400",
        family: "Noto Sans JP",
        weight: "400",
        style: "normal",
        display: "block",
        unicode_range: CJK_RANGE,
    },
];

/// Inject the `@font-face` stylesheet into `document.head`.
///
/// Idempotent: skips when a `<style id="origa-font-faces">` already exists, so
/// repeated calls (or test re-mounts) never duplicate it. A no-op when no subset
/// files are registered, leaving the app on system fallback fonts.
pub fn inject_font_faces() {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    if document.get_element_by_id(STYLE_ID).is_some() {
        return;
    }
    let css = font_face_css();
    if css.is_empty() {
        return;
    }
    let Ok(style) = document.create_element("style") else {
        return;
    };
    style.set_id(STYLE_ID);
    style.set_text_content(Some(&css));
    if let Some(head) = document.head() {
        let _ = head.append_child(&style);
    }
}

/// Build the `@font-face` stylesheet for every font present in `FONT_FILES`.
///
/// Returns an empty string when no subset files are registered (e.g. a build
/// without `cdn/fonts/` populated), so the app falls back to system fonts
/// instead of emitting dangling `@font-face` rules.
fn font_face_css() -> String {
    if FONT_FILES.is_empty() {
        return String::new();
    }

    let mapped: Vec<&str> = FACES
        .iter()
        .filter_map(|spec| filename_for(spec.logical).map(|_| spec.logical))
        .collect();
    let unmapped: Vec<&str> = FONT_FILES
        .iter()
        .filter_map(|(logical, _)| {
            (!FACES.iter().any(|spec| spec.logical == *logical)).then_some(*logical)
        })
        .collect();
    if !unmapped.is_empty() {
        tracing::warn!(
            "cdn/fonts has files with no @font-face spec: {unmapped:?}; \
             they will be downloaded but never served"
        );
    }
    if mapped.len() != FACES.len() {
        tracing::warn!(
            "{} @font-face spec(s) have no matching woff2 in cdn/fonts",
            FACES.len() - mapped.len()
        );
    }

    let mut css = String::new();
    let is_tauri = tauri::is_tauri();
    for spec in FACES {
        let Some(filename) = filename_for(spec.logical) else {
            continue;
        };
        let url = font_file_url(is_tauri, filename);
        css.push_str("@font-face{font-family:\"");
        css.push_str(spec.family);
        css.push_str("\";font-weight:");
        css.push_str(spec.weight);
        css.push_str(";font-style:");
        css.push_str(spec.style);
        css.push_str(";font-display:");
        css.push_str(spec.display);
        css.push_str(";unicode-range:");
        css.push_str(spec.unicode_range);
        css.push_str(";src:url(\"");
        css.push_str(&url);
        css.push_str("\") format(\"woff2\");}\n");
    }
    css
}

/// Resolve a font filename to its served URL.
///
/// Tauri builds serve the woff2 from the local origin (`/fonts/<file>`), which
/// `build.rs` stages from `cdn/fonts/` into `public/fonts/` and trunk ships at
/// `dist/fonts/` — no first-launch network round-trip, works offline. Web
/// builds have no bundled fonts and resolve the same filename on the CDN via
/// `cdn_url`, preserving the single `cdn/fonts/` source of truth (ADR-028).
///
/// `is_tauri` is injected (rather than read inside) so the routing is unit-
/// testable without a WebView/`__TAURI__` global.
fn font_file_url(is_tauri: bool, filename: &str) -> String {
    if is_tauri {
        format!("/fonts/{filename}")
    } else {
        cdn_url(&format!("/fonts/{filename}"))
    }
}

fn filename_for(logical: &str) -> Option<&'static str> {
    FONT_FILES
        .iter()
        .find(|(name, _)| *name == logical)
        .map(|(_, filename)| *filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tauri_build_serves_fonts_from_local_origin() {
        assert_eq!(
            font_file_url(true, "noto-sans-jp-400-a1fd00ef.woff2"),
            "/fonts/noto-sans-jp-400-a1fd00ef.woff2"
        );
    }

    #[test]
    fn web_build_serves_fonts_from_cdn() {
        let url = font_file_url(false, "noto-sans-jp-400-a1fd00ef.woff2");
        assert!(
            url.ends_with("/fonts/noto-sans-jp-400-a1fd00ef.woff2"),
            "web URL must keep the canonical path: {url}"
        );
        assert!(
            url.starts_with("http"),
            "web URL must be absolute against the CDN: {url}"
        );
    }

    #[test]
    fn cjk_faces_block_to_avoid_wrong_glyph_flash() {
        for spec in FACES {
            let is_cjk = spec.unicode_range == CJK_RANGE;
            let expected = if is_cjk { "block" } else { "swap" };
            assert_eq!(
                spec.display, expected,
                "face `{}` (range {}): CJK must block, Latin/Cyrillic must swap",
                spec.logical, spec.display
            );
        }
    }
}
