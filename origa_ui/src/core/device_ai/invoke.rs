//! JS-bridge wrappers for `device-ai-apis` plugin commands.
//!
//! Each function builds the exact command payload the plugin expects (see
//! `guest-js/{speech,vision,capabilities}.ts`) and awaits the resulting
//! promise via [`crate::core::tauri::invoke_with_args`]. A timeout guards
//! against hangs (e.g. a live speech-recognition session waiting on noise that
//! never settles) — on expiry the caller receives an error and routes to the
//! fallback stack.

use std::time::Duration;

use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

use crate::core::tauri;

use super::contracts::{Capabilities, RecognitionResult, TextRecognitionResult, Voice};

const CMD_GET_CAPABILITIES: &str = "plugin:device-ai-apis|get_capabilities";
const CMD_VISION_RECOGNIZE_TEXT: &str = "plugin:device-ai-apis|vision_recognize_text";
const CMD_SPEECH_SYNTHESIZE: &str = "plugin:device-ai-apis|speech_synthesize";
const CMD_SPEECH_GET_VOICES: &str = "plugin:device-ai-apis|speech_get_voices";
const CMD_SPEECH_RECOGNIZE: &str = "plugin:device-ai-apis|speech_recognize";

/// Default guard for quick operations (capabilities, OCR, synthesis). Live
/// recognition may run for the full utterance, so it passes a longer timeout.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(20);
const LIVE_RECOGNITION_TIMEOUT: Duration = Duration::from_secs(45);

/// Sentinel value a timeout promise resolves with so the race can tell a
/// genuine result apart from an expiry.
const TIMEOUT_SENTINEL: &str = "__device_ai_timeout__";

/// Query the native AI capabilities on the current platform.
pub async fn get_capabilities() -> Result<Capabilities, String> {
    let raw = invoke(CMD_GET_CAPABILITIES, &JsValue::UNDEFINED, DEFAULT_TIMEOUT).await?;
    serde_wasm_bindgen::from_value::<Capabilities>(raw)
        .map_err(|e| format!("get_capabilities decode failed: {e:?}"))
}

/// Recognize text in a base64-encoded image. Japanese is prioritised.
pub async fn recognize_text(base64: &str) -> Result<TextRecognitionResult, String> {
    let image = js_sys::Object::new();
    set_str(&image, "base64", base64);

    let languages = js_sys::Array::new();
    languages.push(&JsValue::from_str("ja"));

    let options = js_sys::Object::new();
    set_ref(&options, "languages", &languages);
    set_str(&options, "recognitionLevel", "accurate");

    let payload = js_sys::Object::new();
    set_ref(&payload, "image", &image);
    set_ref(&payload, "options", &options);

    let raw = invoke(CMD_VISION_RECOGNIZE_TEXT, &payload, DEFAULT_TIMEOUT).await?;
    serde_wasm_bindgen::from_value::<TextRecognitionResult>(raw)
        .map_err(|e| format!("vision_recognize_text decode failed: {e:?}"))
}

/// Synthesize and play `text`. `voice_id` selects a voice returned by
/// [`get_voices`]; pass `None` for the system default.
pub async fn synthesize(text: &str, voice_id: Option<&str>, rate: f32) -> Result<(), String> {
    let options = js_sys::Object::new();
    set_f64(&options, "rate", rate as f64);
    // Pitch slightly raised — the macOS/AVFoundation Japanese voices (Kyoko)
    // read as too flat at the default 1.0; 1.2 matches the legacy plugin:tts
    // path's chosen pitch for consistent voice character across backends.
    set_f64(&options, "pitch", 1.2);
    set_f64(&options, "volume", 1.0);
    if let Some(id) = voice_id {
        set_str(&options, "voice", id);
    }

    let payload = js_sys::Object::new();
    set_str(&payload, "text", text);
    set_ref(&payload, "options", &options);

    invoke(CMD_SPEECH_SYNTHESIZE, &payload, DEFAULT_TIMEOUT)
        .await
        .map(|_| ())
}

/// List the voices available for synthesis on the current platform.
pub async fn get_voices() -> Result<Vec<Voice>, String> {
    let raw = invoke(CMD_SPEECH_GET_VOICES, &JsValue::UNDEFINED, DEFAULT_TIMEOUT).await?;
    serde_wasm_bindgen::from_value::<Vec<Voice>>(raw)
        .map_err(|e| format!("speech_get_voices decode failed: {e:?}"))
}

/// Perform one-shot live-microphone speech recognition. Returns when the
/// speaker pauses. Not streaming — partial results are not reported.
pub async fn recognize_live(language: &str) -> Result<RecognitionResult, String> {
    let options = js_sys::Object::new();
    set_str(&options, "language", language);

    let payload = js_sys::Object::new();
    set_ref(&payload, "options", &options);

    let raw = invoke(CMD_SPEECH_RECOGNIZE, &payload, LIVE_RECOGNITION_TIMEOUT).await?;
    serde_wasm_bindgen::from_value::<RecognitionResult>(raw)
        .map_err(|e| format!("speech_recognize decode failed: {e:?}"))
}

/// Invokes a plugin command and races the result promise against a timeout so
/// a hung native call cannot block the UI permanently.
async fn invoke(command: &str, args: &JsValue, timeout: Duration) -> Result<JsValue, String> {
    let result = tauri::invoke_with_args(command, args).await?;
    race_with_timeout(result, timeout).await
}

async fn race_with_timeout(promise_value: JsValue, timeout: Duration) -> Result<JsValue, String> {
    let promise = promise_value
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| "plugin command did not return a Promise".to_string())?;

    let timeout_promise = make_timeout_promise(timeout);

    let raced = js_sys::Promise::race(&js_sys::Array::of2(&promise, &timeout_promise));
    let settled = JsFuture::from(raced)
        .await
        .map_err(|e| format!("plugin command rejected or timed out: {e:?}"))?;

    if settled.as_string().as_deref() == Some(TIMEOUT_SENTINEL) {
        return Err(format!("plugin command timed out after {timeout:?}"));
    }
    Ok(settled)
}

/// Builds a promise that resolves with [`TIMEOUT_SENTINEL`] after `timeout`.
/// Implemented with `gloo_timers::callback::Timeout`, which takes a plain
/// Rust `FnOnce`; the timer owns itself via `forget()`.
fn make_timeout_promise(timeout: Duration) -> js_sys::Promise {
    js_sys::Promise::new(&mut |resolve, _reject| {
        let resolve_fn = resolve.clone();
        gloo_timers::callback::Timeout::new(timeout.as_millis() as u32, move || {
            let _ = resolve_fn
                .unchecked_ref::<js_sys::Function>()
                .call1(&JsValue::UNDEFINED, &JsValue::from_str(TIMEOUT_SENTINEL));
        })
        .forget();
    })
}

fn set_str(obj: &js_sys::Object, key: &str, value: &str) {
    if js_sys::Reflect::set(obj, &JsValue::from_str(key), &JsValue::from_str(value)).is_err() {
        tracing::warn!("device-ai invoke: failed to set `{key}` (string) on payload");
    }
}

fn set_f64(obj: &js_sys::Object, key: &str, value: f64) {
    if js_sys::Reflect::set(obj, &JsValue::from_str(key), &JsValue::from_f64(value)).is_err() {
        tracing::warn!("device-ai invoke: failed to set `{key}` (f64) on payload");
    }
}

fn set_ref(obj: &js_sys::Object, key: &str, value: &JsValue) {
    if js_sys::Reflect::set(obj, &JsValue::from_str(key), value).is_err() {
        tracing::warn!("device-ai invoke: failed to set `{key}` (ref) on payload");
    }
}
