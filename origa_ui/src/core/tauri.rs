//! Unified access to Tauri v2 JS API (`window.__TAURI__`).
//!
//! All Tauri platform detection and JS API access should go through this module.
//! No other file should use `js_sys::Reflect::get(...("__TAURI__"))` directly.

use js_sys::{Function, Object, Promise, Reflect};
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;

/// Returns `true` if running inside a Tauri WebView (desktop or mobile).
pub fn is_tauri() -> bool {
    tauri_object().is_some()
}

/// Returns `true` on iOS (iPhone/iPad/iPod WKWebView).
///
/// Uses `navigator.platform` first — on iPad with "Request Desktop Site"
/// (iOS 13+) the userAgent switches to a desktop-class string without
/// "iPad", but `navigator.platform` still returns "iPad". Falls back to
/// userAgent for older WebViews where platform may be unavailable.
pub fn is_ios() -> bool {
    window().is_some_and(|w| {
        let nav = w.navigator();
        if let Ok(platform) = nav.platform() {
            if platform.contains("iPhone") || platform.contains("iPad") || platform.contains("iPod")
            {
                return true;
            }
        }
        nav.user_agent()
            .is_ok_and(|ua| ua.contains("iPhone") || ua.contains("iPad") || ua.contains("iPod"))
    })
}

/// Returns the `window.__TAURI__` object if available.
pub fn tauri_object() -> Option<Object> {
    window()
        .and_then(|w| Reflect::get(&w, &JsValue::from_str("__TAURI__")).ok())
        .and_then(|v| {
            if v.is_undefined() || v.is_null() {
                None
            } else {
                v.dyn_into::<Object>().ok()
            }
        })
}

/// Returns `window.__TAURI__.core.invoke` function if available.
///
/// Used to call Tauri commands from WASM.
pub fn invoke_fn() -> Option<Function> {
    let obj = tauri_object()?;
    let core = Reflect::get(&obj, &JsValue::from_str("core")).ok()?;
    Reflect::get(&core, &JsValue::from_str("invoke"))
        .ok()
        .and_then(|v| v.dyn_into::<Function>().ok())
}

/// Returns `window.__TAURI__.event.listen` function if available.
///
/// Used to register event listeners from WASM.
pub fn event_listen_fn() -> Option<Function> {
    let obj = tauri_object()?;
    let event = Reflect::get(&obj, &JsValue::from_str("event")).ok()?;
    Reflect::get(&event, &JsValue::from_str("listen"))
        .ok()
        .and_then(|v| v.dyn_into::<Function>().ok())
}

/// Returns `window.__TAURI__.opener.openUrl` function if available.
///
/// Used to open URLs in the system browser from Tauri.
pub fn opener_open_url_fn() -> Option<Function> {
    let obj = tauri_object()?;
    let opener = Reflect::get(&obj, &JsValue::from_str("opener")).ok()?;
    Reflect::get(&opener, &JsValue::from_str("openUrl"))
        .ok()
        .and_then(|v| v.dyn_into::<Function>().ok())
}

/// Opens `url` in the system browser (Tauri) or navigates the current tab
/// (browser). Canonical implementation shared by every call site that needs to
/// leave the WebView — legal-document links, OAuth provider redirect, etc.
///
/// Tauri path: `opener.openUrl` returns a Promise on Tauri v2 (mobile +
/// desktop). The synchronous `call1` only catches construction-time
/// exceptions; async rejections (capability scope mismatch, browser-launch
/// failure on Android) are handled by awaiting the Promise and falling back to
/// `window.open` on rejection. This mirrors the historic OAuth fix for the
/// "button does nothing on Android" symptom.
///
/// Browser path: `location.href = url` navigates the current tab.
pub fn open_url_external(url: &str) {
    let Some(window) = window() else {
        return;
    };

    if !is_tauri() {
        let _ = window.location().set_href(url);
        return;
    }

    let Some(open_url_fn) = opener_open_url_fn() else {
        let _ = window.open_with_url_and_target(url, "_blank");
        return;
    };

    match open_url_fn.call1(&JsValue::UNDEFINED, &JsValue::from_str(url)) {
        Ok(value) => {
            if let Ok(promise) = value.dyn_into::<Promise>() {
                let url_owned = url.to_string();
                spawn_local(async move {
                    if JsFuture::from(promise).await.is_err()
                        && let Some(w) = web_sys::window()
                    {
                        let _ = w.open_with_url_and_target(&url_owned, "_blank");
                    }
                });
            }
        },
        Err(_) => {
            let _ = window.open_with_url_and_target(url, "_blank");
        },
    }
}

/// Reload the current Tauri WebView.
///
/// Primary path: `window.__TAURI__.webview.getCurrentWebview().reload()`
/// (Tauri v2.4.0+, added in tauri-apps/tauri#12818). This is the native WebView
/// reload and, by design, does not perform a top-level navigation to the origin,
/// so it does not reach the URL-navigation policy that can leak
/// `tauri.localhost` to the system browser on Android.
///
/// Fallback: `window.location.reload()`, used when the native `reload` method is
/// unavailable on the running Tauri version. Combined with `preventDefault()` on
/// the keydown accelerator (see `hooks::keyboard_shortcuts`), this keeps the
/// reload inside the WebView instead of letting the accelerator's default
/// handling run.
///
/// Returns `Err` only if `window` itself is unavailable (effectively never in a
/// browser/WASM context).
pub fn reload_current_webview() -> Result<(), String> {
    let window = window().ok_or("window not available")?;

    if try_native_webview_reload() {
        return Ok(());
    }

    tracing::warn!(
        "Native Tauri WebView reload unavailable; falling back to window.location.reload()"
    );
    window
        .location()
        .reload()
        .map_err(|e| format!("window.location.reload() failed: {e:?}"))
}

/// Attempt `window.__TAURI__.webview.getCurrentWebview().reload()`.
///
/// Returns `false` if any step of the JS property/call chain is unavailable so
/// the caller can fall back. Resilient to Tauri versions that do not expose the
/// `reload` method on the `Webview` instance.
fn try_native_webview_reload() -> bool {
    let obj = match tauri_object() {
        Some(o) => o,
        None => return false,
    };

    let webview_mod = match Reflect::get(&obj, &JsValue::from_str("webview")) {
        Ok(v) if !v.is_undefined() && !v.is_null() => v,
        _ => return false,
    };

    let get_current = match Reflect::get(&webview_mod, &JsValue::from_str("getCurrentWebview")) {
        Ok(v) => match v.dyn_into::<Function>() {
            Ok(f) => f,
            Err(_) => return false,
        },
        Err(_) => return false,
    };

    let webview = match get_current.call0(&JsValue::UNDEFINED) {
        Ok(v) if !v.is_undefined() && !v.is_null() => v,
        _ => return false,
    };

    let reload_fn = match Reflect::get(&webview, &JsValue::from_str("reload")) {
        Ok(v) => match v.dyn_into::<Function>() {
            Ok(f) => f,
            // reload() method absent on this Tauri version → fall back.
            Err(_) => return false,
        },
        Err(_) => return false,
    };

    // reload() is an instance method — call with the webview as `this`.
    reload_fn.call0(&webview).is_ok()
}

/// Calls a Tauri command with the given arguments object and awaits the result.
///
/// `args` must be a JS object whose keys match the command's parameter names
/// (e.g. `{ key: "trailbase_session", value: "{...}" }`). Pass `&JsValue::UNDEFINED`
/// for commands that take no arguments.
///
/// Returns the resolved promise value, or an error string describing the failure.
pub async fn invoke_with_args(command: &str, args: &JsValue) -> Result<JsValue, String> {
    let promise = invoke_promise(command, args)?;
    JsFuture::from(promise)
        .await
        .map_err(|e| format!("invoke('{command}') rejected: {e:?}"))
}

/// Returns the raw JS Promise for a Tauri command invocation, without awaiting.
///
/// Callers that need to race the call against a timeout (see
/// `device_ai::invoke`) use this to keep the promise pending while the race
/// runs; awaiting first (as `invoke_with_args` does) would hand back the
/// resolved value, which is no longer a Promise.
pub fn invoke_promise(command: &str, args: &JsValue) -> Result<js_sys::Promise, String> {
    let invoke = invoke_fn().ok_or_else(|| "Tauri invoke not available".to_string())?;
    let result = invoke
        .call2(&JsValue::UNDEFINED, &JsValue::from_str(command), args)
        .map_err(|e| format!("invoke('{command}') call failed: {e:?}"))?;
    result
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| format!("invoke('{command}') did not return a Promise"))
}
