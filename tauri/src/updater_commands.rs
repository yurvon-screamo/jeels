//! Tauri IPC commands for the auto-updater (`tauri-plugin-updater`).
//!
//! The frontend (WASM) calls these commands through `invoke` to check for
//! available updates and install them. Progress events are emitted via
//! `app.emit("origa-update-progress", DownloadEvent)` so the WASM side can
//! subscribe through the existing `core::tauri::event_listen_fn()` helper
//! without any new JS-reflection code.
//!
//! The `Update` returned by `check` is stored in `PendingUpdate` state and
//! consumed by `install_update` — this avoids a second network round-trip and
//! the race condition of the manifest changing between check and install.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_updater::{Update, UpdaterExt};

/// Holds the `Update` returned by `check_for_update` until `install_update`
/// consumes it. Without this state, `install_update` would have to re-run the
/// check (extra network round-trip + manifest race).
pub struct PendingUpdate(pub Mutex<Option<Update>>);

impl PendingUpdate {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }
}

impl Default for PendingUpdate {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize)]
pub struct UpdateMetadata {
    pub version: String,
    pub current_version: String,
}

/// Event emitted on the `"origa-update-progress"` channel during
/// `install_update`. Per-variant `rename_all = "camelCase"` so the WASM side
/// receives canonical JS-style field names (`contentLength`, `chunkLength`).
#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum DownloadEvent {
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

const PROGRESS_EVENT: &str = "origa-update-progress";

/// Checks the configured updater endpoint for a newer release.
///
/// Returns `Ok(None)` when the current build is already up to date. When an
/// update exists, its metadata is returned AND the `Update` object is stashed
/// in `PendingUpdate` so `install_update` can consume it without re-checking.
#[tauri::command]
#[cfg(any(windows, target_os = "linux"))]
pub async fn check_for_update(
    app: AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<Option<UpdateMetadata>, String> {
    let update = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;

    let metadata = update.as_ref().map(|u| UpdateMetadata {
        version: u.version.clone(),
        current_version: u.current_version.clone(),
    });

    tracing::info!(?metadata, "updater check result");

    *pending.0.lock().unwrap_or_else(|e| e.into_inner()) = update;

    Ok(metadata)
}

/// Downloads and installs the update previously stashed by `check_for_update`.
///
/// Emits `Started` (once, on the first chunk), `Progress` (per chunk), and
/// `Finished` on the `"origa-update-progress"` channel, then restarts the app.
///
/// On Windows the NSIS installer terminates the process during
/// `download_and_install`, so `app.restart()` is unreachable there — it
/// remains as the restart path for Linux and macOS.
#[tauri::command]
#[cfg(any(windows, target_os = "linux"))]
pub async fn install_update(
    app: AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<(), String> {
    let update_opt = pending.0.lock().unwrap_or_else(|e| e.into_inner()).take();
    let update = update_opt.ok_or_else(|| "no pending update".to_string())?;

    let started = AtomicBool::new(false);
    let app_for_progress = app.clone();
    let app_for_finish = app.clone();

    update
        .download_and_install(
            move |chunk_length, content_length| {
                if !started.swap(true, Ordering::Relaxed) {
                    let _ = app_for_progress
                        .emit(PROGRESS_EVENT, DownloadEvent::Started { content_length });
                }
                let _ =
                    app_for_progress.emit(PROGRESS_EVENT, DownloadEvent::Progress { chunk_length });
            },
            move || {
                let _ = app_for_finish.emit(PROGRESS_EVENT, DownloadEvent::Finished);
            },
        )
        .await
        .map_err(|e| e.to_string())?;

    app.restart();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Wire-format contract: the WASM side (`origa_ui/src/core/updater.rs`)
    /// mirrors `DownloadEvent` with the same serde tags and per-variant
    /// `rename_all = "camelCase"`. This round-trip test pins the canonical
    /// JSON shape; if either side drifts, deserialization on the WASM side
    /// fails silently (progress events stop arriving) — the same class of
    /// bug this updater rewrite was meant to eliminate.
    #[test]
    fn download_event_started_serializes_camel_case() {
        let event = DownloadEvent::Started {
            content_length: Some(12345),
        };
        let json = serde_json::to_string(&event).expect("serialize Started");
        assert_eq!(
            json,
            r#"{"event":"Started","data":{"contentLength":12345}}"#
        );
    }

    #[test]
    fn download_event_started_with_null_content_length() {
        let event = DownloadEvent::Started {
            content_length: None,
        };
        let json = serde_json::to_string(&event).expect("serialize Started None");
        assert_eq!(json, r#"{"event":"Started","data":{"contentLength":null}}"#);
    }

    #[test]
    fn download_event_progress_serializes_camel_case() {
        let event = DownloadEvent::Progress { chunk_length: 1024 };
        let json = serde_json::to_string(&event).expect("serialize Progress");
        assert_eq!(json, r#"{"event":"Progress","data":{"chunkLength":1024}}"#);
    }

    #[test]
    fn download_event_finished_serializes_with_event_tag() {
        let event = DownloadEvent::Finished;
        let json = serde_json::to_string(&event).expect("serialize Finished");
        // serde tag/content for a unit variant may omit the `data` slot — the
        // event tag alone identifies the variant. Assert only the tag.
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("Finished JSON must parse");
        assert_eq!(parsed["event"], "Finished");
    }

    #[test]
    fn update_metadata_serializes_with_both_version_fields() {
        let metadata = UpdateMetadata {
            version: "0.4.2".to_string(),
            current_version: "0.4.1-rc3".to_string(),
        };
        let json = serde_json::to_string(&metadata).expect("serialize UpdateMetadata");
        assert_eq!(json, r#"{"version":"0.4.2","current_version":"0.4.1-rc3"}"#);
    }

    #[test]
    fn pending_update_new_starts_empty() {
        let pending = PendingUpdate::new();
        assert!(pending.0.lock().unwrap().is_none());
    }
}
