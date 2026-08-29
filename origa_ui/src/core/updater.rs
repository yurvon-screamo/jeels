use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use leptos::wasm_bindgen::JsValue;
use leptos::wasm_bindgen::closure::Closure;
use serde::Deserialize;

use crate::core::tauri::{event_listen_fn, invoke_with_args, is_tauri};

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "event", content = "data")]
enum DownloadEvent {
    #[serde(rename_all = "camelCase")]
    Started {
        content_length: Option<u64>,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        chunk_length: usize,
    },
    Finished,
}

#[derive(Default)]
struct ProgressState {
    total_size: Option<u64>,
    downloaded: u64,
}

const PROGRESS_EVENT: &str = "origa-update-progress";

/// Returns `true` when the desktop binary is an app-store distribution
/// (Microsoft Store MSIX / Mac App Store): the self-update machinery is
/// compiled out there and this UI must not offer update checks
/// (Store policy 10.2.5 / Guideline 2.4.5(vii)).
///
/// Always `false` outside the Tauri WebView — the browser build has no
/// store identity and no update UI either.
pub async fn is_store_build() -> bool {
    if !is_tauri() {
        return false;
    }

    match invoke_with_args("is_store_build", &JsValue::UNDEFINED).await {
        Ok(result) => result.as_bool().unwrap_or(false),
        Err(e) => {
            tracing::warn!("updater: is_store_build invoke failed: {e}");
            false
        },
    }
}

/// Checks the Tauri updater endpoint for a newer release.
///
/// Returns `None` outside the Tauri WebView (browser build) or when no update
/// is available. The runtime `is_tauri()` gate is preferred over a compile-time
/// `cfg(target_os = …)` because Tauri CLI does not always inject `target_os`
/// cfg-flags for the WASM build, while `window.__TAURI__` is always present
/// inside the desktop WebView.
pub async fn check_for_updates() -> Option<UpdateInfo> {
    if !is_tauri() {
        tracing::debug!("updater: skipping check outside Tauri WebView");
        return None;
    }

    match invoke_with_args("check_for_update", &JsValue::UNDEFINED).await {
        Ok(result) if result.is_null() || result.is_undefined() => {
            tracing::debug!("updater: no update available");
            None
        },
        Ok(result) => match serde_wasm_bindgen::from_value::<UpdateInfo>(result) {
            Ok(info) => {
                tracing::info!(?info, "updater: update available");
                Some(info)
            },
            Err(e) => {
                tracing::warn!("updater: failed to parse UpdateInfo: {e:?}");
                None
            },
        },
        Err(e) => {
            tracing::warn!("updater: check_for_update invoke failed: {e}");
            None
        },
    }
}

/// Downloads and installs the pending update.
///
/// Subscribes to `"origa-update-progress"` events emitted by the Rust
/// `install_update` command, converts chunks into a 0–100 percent value and
/// forwards it to `on_progress`. Outside Tauri this is a no-op success.
pub async fn download_and_install<F>(on_progress: F) -> Result<(), String>
where
    F: Fn(u8) + Send + Sync + 'static,
{
    if !is_tauri() {
        tracing::debug!("updater: skipping install outside Tauri WebView");
        return Ok(());
    }

    let listen_fn = event_listen_fn().ok_or_else(|| {
        "Tauri event.listen unavailable — cannot subscribe to update progress".to_string()
    })?;

    let state = Rc::new(RefCell::new(ProgressState::default()));
    let on_progress = Arc::new(on_progress);

    let state_for_closure = state.clone();
    let on_progress_for_closure = on_progress.clone();
    let progress_closure = Closure::<dyn Fn(JsValue)>::new(move |envelope: JsValue| {
        let Some(event) = extract_event(&envelope) else {
            return;
        };
        let mut s = state_for_closure.borrow_mut();
        match event {
            DownloadEvent::Started { content_length } => {
                s.total_size = content_length;
            },
            DownloadEvent::Progress { chunk_length } => {
                s.downloaded = s.downloaded.saturating_add(chunk_length as u64);
                if let Some(total) = s.total_size
                    && total > 0
                {
                    let ratio = (s.downloaded as f64 / total as f64).clamp(0.0, 1.0);
                    on_progress_for_closure((ratio * 100.0) as u8);
                }
            },
            DownloadEvent::Finished => {
                on_progress_for_closure(100);
            },
        }
    });

    let event_name = JsValue::from_str(PROGRESS_EVENT);
    if listen_fn
        .call2(&JsValue::UNDEFINED, &event_name, progress_closure.as_ref())
        .is_err()
    {
        // Bail out before invoking install_update: without a listener we cannot
        // surface download progress, and the failure mode is silent (install
        // would still download but the user would see a frozen progress bar).
        return Err(
            "Tauri event.listen call failed — cannot subscribe to update progress".to_string(),
        );
    }
    // Intentionally leak: the listener must survive for the duration of
    // download_and_install (and the app exits/restarts immediately after
    // install), so there is no natural unlisten point.
    progress_closure.forget();

    // install_update never returns on Windows (NSIS terminates the process
    // mid-install); on Linux/macOS the Rust command calls `app.restart()`
    // which terminates the process before the promise resolves. The only way
    // this await resolves is when install_update itself returns an Err
    // (e.g. no pending update) — propagate that, otherwise the success path
    // never reaches here.
    invoke_with_args("install_update", &JsValue::UNDEFINED)
        .await
        .map(|_| ())
}

fn extract_event(envelope: &JsValue) -> Option<DownloadEvent> {
    let payload = js_sys::Reflect::get(envelope, &JsValue::from_str("payload"))
        .ok()
        .filter(|v| !v.is_undefined() && !v.is_null())?;
    serde_wasm_bindgen::from_value::<DownloadEvent>(payload)
        .map_err(|e| tracing::warn!("updater: failed to parse DownloadEvent payload: {e:?}"))
        .ok()
}
