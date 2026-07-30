//! device-ai native ASR paths.
//!
//! File-based recognition requires the macOS-only `device_ai_recognize_file`
//! Tauri command (the JS plugin surface accepts only live-microphone input).
//! Live recognition goes through the plugin's `speech_recognize` command.
//! Both return `None` when unavailable so callers fall back to Whisper WASM.

use crate::core::device_ai::{self, Feature};
use tracing::warn;

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
        return None;
    }

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

/// Perform one-shot live-microphone recognition via native device-ai.
/// Returns `Some(text)` on success, or `None` when unavailable/failed.
pub(super) async fn recognize_live_via_device_ai() -> Option<String> {
    if !device_ai::available(Feature::SpeechRecognition).await {
        return None;
    }

    match device_ai::recognize_live(JA_JP).await {
        Ok(result) => Some(result.text),
        Err(e) => {
            warn!("device-ai live ASR failed: {e}");
            None
        },
    }
}
