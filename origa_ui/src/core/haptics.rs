//! Haptic feedback for FSRS rating actions via Tauri plugin.
//!
//! Maps each `Rating` variant to a Tauri haptics JS call:
//! - `Again` → notification(Error) — strong negative signal
//! - `Hard`  → impact(Medium)     — moderate friction
//! - `Good`  → impact(Light)      — gentle confirmation
//! - `Easy`  → notification(Success) — positive reward
//!
//! On desktop (no haptic hardware) and on browsers (not running in Tauri
//! WebView) the calls silently no-op: `window.__TAURI__` is undefined or the
//! haptics module is absent (e.g. plugin not registered on this platform).
//!
//! JS bridge follows the same pattern as `core::tauri::opener_open_url_fn`:
//! resolve `window.__TAURI__.haptics.<fn>` via Reflect, call with one string
//! arg, ignore the returned Promise (fire-and-forget — the user feels the
//! haptic immediately, no need to await completion).

use js_sys::{Function, Reflect};
use leptos::wasm_bindgen::{JsCast, JsValue};
use origa::domain::Rating;

use super::tauri::{is_tauri, tauri_object};

const HAPTICS_MODULE: &str = "haptics";

fn haptics_fn(name: &str) -> Option<Function> {
    let obj = tauri_object()?;
    let haptics = Reflect::get(&obj, &JsValue::from_str(HAPTICS_MODULE)).ok()?;
    Reflect::get(&haptics, &JsValue::from_str(name))
        .ok()
        .and_then(|v| v.dyn_into::<Function>().ok())
}

/// Trigger haptic feedback appropriate for the given FSRS rating.
///
/// Must be called synchronously from the click/keyboard handler (before
/// `spawn_local` for the actual rating persistence) so the user feels the
/// haptic at the moment of contact, not after the network round-trip.
pub fn rating_feedback(rating: Rating) {
    if !is_tauri() {
        return;
    }

    let (fn_name, style) = match rating {
        Rating::Again => ("notification", "Error"),
        Rating::Hard => ("impact", "Medium"),
        Rating::Good => ("impact", "Light"),
        Rating::Easy => ("notification", "Success"),
    };

    let Some(f) = haptics_fn(fn_name) else {
        return;
    };

    if let Err(e) = f.call1(&JsValue::UNDEFINED, &JsValue::from_str(style)) {
        tracing::warn!("[haptics] {fn_name}({style}) call failed: {e:?}");
    }
}
