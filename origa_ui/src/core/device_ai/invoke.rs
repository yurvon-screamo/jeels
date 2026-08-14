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

use super::contracts::{Capabilities, RecognitionResult, TextRecognitionResult};

const CMD_GET_CAPABILITIES: &str = "plugin:device-ai-apis|get_capabilities";
const CMD_VISION_RECOGNIZE_TEXT: &str = "plugin:device-ai-apis|vision_recognize_text";
const CMD_SPEECH_RECOGNIZE: &str = "plugin:device-ai-apis|speech_recognize";

/// Default guard for quick operations (capabilities, OCR). Live recognition
/// may run for the full utterance, so it passes a longer timeout.
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
    tracing::debug!("device-ai: invoking {command} (timeout {timeout:?})");
    let promise = tauri::invoke_promise(command, args)?;
    race_with_timeout(promise, timeout).await
}

async fn race_with_timeout(promise: js_sys::Promise, timeout: Duration) -> Result<JsValue, String> {
    let timeout_promise = make_timeout_promise(timeout);

    let raced = js_sys::Promise::race(&js_sys::Array::of2(&promise, &timeout_promise));
    let settled = JsFuture::from(raced)
        .await
        .map_err(|e| format!("plugin command rejected: {}", describe_rejection(&e)))?;

    if settled.as_string().as_deref() == Some(TIMEOUT_SENTINEL) {
        return Err(format!("plugin command timed out after {timeout:?}"));
    }
    Ok(settled)
}

/// Formats a plugin rejection for logs, keeping the structured `code` field
/// (e.g. `NO_SPEECH_DETECTED`, `PERMISSION_DENIED`) if the plugin sent one.
///
/// The code is what routing decisions key on (see `asr_provider`), so it must
/// survive the `String` error transport — this keeps `{code} {message}` in
/// the log line while callers parse it back out.
fn describe_rejection(e: &JsValue) -> String {
    let code = js_sys::Reflect::get(e, &JsValue::from_str("code"))
        .ok()
        .and_then(|c| c.as_string());
    let message = js_sys::Reflect::get(e, &JsValue::from_str("message"))
        .ok()
        .and_then(|m| m.as_string());
    match (code, message) {
        (Some(code), Some(message)) => format!("[{code}] {message}"),
        (Some(code), None) => code,
        (None, Some(message)) => message,
        (None, None) => format!("{e:?}"),
    }
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

fn set_ref(obj: &js_sys::Object, key: &str, value: &JsValue) {
    if js_sys::Reflect::set(obj, &JsValue::from_str(key), value).is_err() {
        tracing::warn!("device-ai invoke: failed to set `{key}` (ref) on payload");
    }
}
