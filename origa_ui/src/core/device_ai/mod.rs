//! High-level device-ai provider: capabilities-first routing decisions plus
//! cached access to the plugin's native ASR/OCR/TTS.
//!
//! Routing contract:
//! - Web (no Tauri) → the plugin is unreachable; [`available`] returns `false`
//!   for every feature and callers use the WASM/`speechSynthesis` fallback.
//! - Tauri on Windows/Linux → the plugin is not compiled in; invoking it
//!   fails, capabilities stay unavailable, fallback is used.
//! - Tauri on macOS/iOS/Android → capabilities are queried once, cached, and
//!   each feature routes to device-ai when reported available, otherwise to
//!   the fallback stack.
//!
//! All plugin access goes through [`invoke`], which enforces a per-call
//! timeout so a hung native call falls back instead of freezing the UI.

pub mod contracts;
mod invoke;

use std::cell::RefCell;

use contracts::{Capabilities, RecognitionResult, TextRecognitionResult, Voice};

use crate::core::tauri;

/// Feature identifiers used by routing decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    SpeechRecognition,
    SpeechSynthesis,
    TextRecognition,
}

impl Feature {
    fn status(self, caps: &Capabilities) -> bool {
        match self {
            Self::SpeechRecognition => caps.speech_recognition.available,
            Self::SpeechSynthesis => caps.speech_synthesis.available,
            Self::TextRecognition => caps.text_recognition.available,
        }
    }
}

thread_local! {
    /// Capabilities cache. `None` = not yet queried; `Some(caps)` = queried
    /// (possibly all-unavailable). Queried once per session to avoid repeated
    /// plugin round-trips on every TTS/OCR/ASR call.
    static CACHED_CAPABILITIES: RefCell<Option<Capabilities>> = const { RefCell::new(None) };
    static CAPABILITIES_LOADING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Returns `true` if the device-ai plugin could be present at all.
///
/// Cheap, synchronous, allocation-free — the first check before any async
/// plugin round-trip. This is a runtime check, not a compile-time `cfg`:
/// the frontend is one WASM binary shared across every Tauri host, so
/// `target_os` is always `unknown` here. The actual per-platform presence
/// (plugin registered on macOS/iOS/Android, absent on Windows/Linux) is
/// resolved by the capabilities query — an unregistered plugin rejects the
/// `get_capabilities` invoke and collapses to "unavailable".
pub fn plugin_compiled_in() -> bool {
    tauri::is_tauri()
}

/// Reports whether `feature` is available via device-ai on this platform.
///
/// Triggers a one-shot capabilities query on first use, then reads the cache.
/// Any failure (plugin absent, invoke error, timeout) collapses to
/// "unavailable" so callers transparently use the fallback stack.
pub async fn available(feature: Feature) -> bool {
    if !plugin_compiled_in() {
        return false;
    }
    let caps = match cached_or_query().await {
        Ok(c) => c,
        Err(_) => return false,
    };
    feature.status(&caps)
}

/// Resolves capabilities from the cache, querying the plugin on first access.
/// Guards against concurrent queries with a loading flag.
///
/// Only a *successful* query is cached — including an honest "all features
/// unavailable". A transient failure (plugin not yet ready, timeout, decode
/// error) is returned as `all_unavailable` for the current call but is NOT
/// cached, so the next call retries instead of permanently disabling device-ai
/// for the whole session.
async fn cached_or_query() -> Result<Capabilities, String> {
    if let Some(caps) = CACHED_CAPABILITIES.with(|c| c.borrow().clone()) {
        return Ok(caps);
    }

    if CAPABILITIES_LOADING.with(|l| l.get()) {
        // Another caller is querying; treat as unavailable this turn — the
        // caller falls back, and subsequent calls hit the now-warm cache.
        return Ok(Capabilities::all_unavailable());
    }

    CAPABILITIES_LOADING.with(|l| l.set(true));
    let result = invoke::get_capabilities().await;
    CAPABILITIES_LOADING.with(|l| l.set(false));

    match result {
        Ok(caps) => {
            CACHED_CAPABILITIES.with(|c| *c.borrow_mut() = Some(caps.clone()));
            Ok(caps)
        },
        Err(e) => {
            tracing::debug!("device-ai capabilities query failed, will retry: {e}");
            Ok(Capabilities::all_unavailable())
        },
    }
}

/// Recognize text in a base64-encoded image via native OCR. Returns `Err` if
/// the plugin is unavailable or the call fails — callers fall back to WASM.
pub async fn recognize_text(base64: &str) -> Result<TextRecognitionResult, String> {
    invoke::recognize_text(base64).await
}

/// Synthesize and play `text` via native TTS. Resolves when playback finishes.
pub async fn synthesize(text: &str, voice_id: Option<&str>, rate: f32) -> Result<(), String> {
    invoke::synthesize(text, voice_id, rate).await
}

/// List native synthesis voices.
pub async fn get_voices() -> Result<Vec<Voice>, String> {
    invoke::get_voices().await
}

/// One-shot live-microphone recognition. Returns `Err` if unavailable —
/// callers fall back to recording + Whisper WASM.
pub async fn recognize_live(language: &str) -> Result<RecognitionResult, String> {
    invoke::recognize_live(language).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_status_reads_available_flag() {
        let mut caps = Capabilities::all_unavailable();
        caps.speech_synthesis.available = true;

        assert!(!Feature::SpeechRecognition.status(&caps));
        assert!(Feature::SpeechSynthesis.status(&caps));
        assert!(!Feature::TextRecognition.status(&caps));
    }

    #[test]
    fn all_unavailable_reports_false_for_every_feature() {
        let caps = Capabilities::all_unavailable();

        assert!(!Feature::SpeechRecognition.status(&caps));
        assert!(!Feature::SpeechSynthesis.status(&caps));
        assert!(!Feature::TextRecognition.status(&caps));
    }
}
