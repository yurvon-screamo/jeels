//! device-ai native ASR paths.
//!
//! File-based recognition requires the macOS-only `device_ai_recognize_file`
//! Tauri command (the JS plugin surface accepts only live-microphone input).
//! Live recognition goes through the plugin's `speech_recognize` command.
//! Both return `None` when unavailable so callers fall back to Whisper WASM.

use crate::core::device_ai::{self, Feature};
use tracing::{info, warn};

const JA_JP: &str = "ja-JP";

/// Recognize speech in a base64-encoded audio buffer via native device-ai
/// (macOS desktop). Returns `Some(text)` on success, or `None` when
/// unavailable/failed — callers fall back to Whisper WASM.
///
/// WASM-only: invoked solely from the WASM transcription path.
#[cfg(target_arch = "wasm32")]
pub(super) async fn recognize_file_via_device_ai(base64_audio: &str) -> Option<String> {
    use crate::core::tauri;
    use leptos::wasm_bindgen::JsValue;

    const CMD_RECOGNIZE_FILE: &str = "device_ai_recognize_file";

    if !device_ai::available(Feature::SpeechRecognition).await {
        info!("ASR (file): native device-ai unavailable, using Whisper WASM fallback");
        return None;
    }
    info!("ASR (file): using native device-ai recognition");

    let inner = js_sys::Object::new();
    let _ = js_sys::Reflect::set(
        &inner,
        &JsValue::from_str("base64"),
        &JsValue::from_str(base64_audio),
    );
    let _ = js_sys::Reflect::set(
        &inner,
        &JsValue::from_str("language"),
        &JsValue::from_str(JA_JP),
    );

    let payload = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&payload, &JsValue::from_str("args"), &inner);

    match tauri::invoke_with_args(CMD_RECOGNIZE_FILE, &payload).await {
        Ok(value) => js_sys::Reflect::get(&value, &JsValue::from_str("text"))
            .ok()
            .and_then(|t| t.as_string()),
        Err(e) => {
            warn!("device-ai file ASR failed, falling back to Whisper: {e}");
            None
        },
    }
}

/// Error code the plugin reports when a live session ended without any
/// recognized speech (bounded no-speech budget, see the fork's
/// `speech_live_ctrl.rs`).
const NO_SPEECH_CODE: &str = "[NO_SPEECH_DETECTED]";

/// Perform one-shot live-microphone recognition via native device-ai.
/// Returns `Some(text)` on success, or `None` when unavailable/failed.
pub(super) async fn recognize_live_via_device_ai() -> Option<String> {
    if !device_ai::available(Feature::SpeechRecognition).await {
        info!("ASR (live): native device-ai unavailable, using Whisper WASM fallback");
        return None;
    }
    info!("ASR (live): using native device-ai recognition");

    match device_ai::recognize_live(JA_JP).await {
        Ok(result) => Some(result.text),
        Err(e) => {
            // "No speech" is an expected user outcome, not a failure of the
            // native path — returning an empty string routes the caller to
            // its no-speech message instead of disabling the button.
            if e.contains(NO_SPEECH_CODE) {
                info!("ASR (live): no speech detected");
                return Some(String::new());
            }
            warn!("device-ai live ASR failed: {e}");
            None
        },
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::NO_SPEECH_CODE;

    /// The marker must match how `describe_rejection` formats plugin errors
    /// (see `core/device_ai/invoke.rs`) — both sides of the contract are
    /// plain strings, so a drift would silently route "no speech" to the
    /// "unavailable" fallback.
    #[test]
    fn no_speech_marker_matches_invoke_error_format() {
        let rejection = format!("plugin command rejected: {NO_SPEECH_CODE} No speech detected");
        assert!(rejection.contains(NO_SPEECH_CODE));

        // Unrelated failures must not match the marker.
        let timeout = "plugin command timed out after 45s".to_string();
        assert!(!timeout.contains(NO_SPEECH_CODE));
        let generic = "plugin command rejected: [SPEECH_RECOGNITION_FAILED] boom".to_string();
        assert!(!generic.contains(NO_SPEECH_CODE));
    }
}
