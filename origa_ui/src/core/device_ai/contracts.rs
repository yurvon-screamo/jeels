//! Wire-format contracts for the `device-ai-apis` Tauri plugin commands.
//!
//! These mirror the TypeScript types in the plugin's `guest-js/types.ts`. The
//! plugin serialises with camelCase, so every struct carries
//! `#[serde(rename_all = "camelCase")]` — without it, fields silently fall
//! back to defaults (e.g. `available = false`) and the capabilities-first
//! routing would permanently route to the fallback stack.

use serde::Deserialize;

/// Availability of a single native AI feature on the current platform.
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FeatureStatus {
    #[serde(default)]
    pub available: bool,
}

/// Native AI capabilities reported by `get_capabilities`.
///
/// Only the features Origa routes through device-ai are decoded; the remaining
/// plugin capabilities (barcode/face/image-classification/llm) are ignored.
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    #[serde(default)]
    pub speech_recognition: FeatureStatus,
    #[serde(default)]
    pub speech_synthesis: FeatureStatus,
    #[serde(default)]
    pub text_recognition: FeatureStatus,
}

impl Capabilities {
    /// All features reported as unavailable — the value used when the plugin is
    /// absent (Windows/Linux) or the capability query itself failed.
    pub fn all_unavailable() -> Self {
        Self::default()
    }
}

/// Result of one-shot speech recognition (live microphone or audio file).
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecognitionResult {
    #[serde(default)]
    pub text: String,
}

/// Result of OCR text recognition. Only the full text is consumed by Origa;
/// per-block geometry is decoded away by serde (unknown fields are ignored).
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TextRecognitionResult {
    #[serde(default)]
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_deserialize_from_plugin_camel_case_payload() {
        let json = r#"{
            "speechRecognition": { "available": true, "onDevice": true, "requiresPermission": true },
            "speechSynthesis": { "available": true, "onDevice": true, "requiresPermission": false },
            "textRecognition": { "available": true, "onDevice": true, "requiresPermission": false },
            "barcodeDetection": { "available": false, "onDevice": false, "requiresPermission": false }
        }"#;

        let caps: Capabilities = serde_json::from_str(json).unwrap();

        assert!(caps.speech_recognition.available);
        assert!(caps.text_recognition.available);
    }

    #[test]
    fn capabilities_default_to_unavailable_for_partial_payload() {
        // A payload missing feature keys must not panic — it yields unavailable.
        let json = r#"{}"#;

        let caps: Capabilities = serde_json::from_str(json).unwrap();

        assert!(!caps.speech_recognition.available);
        assert_eq!(caps, Capabilities::all_unavailable());
    }
}
