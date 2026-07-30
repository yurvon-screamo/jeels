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

/// A synthesizable voice installed on the system.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Voice {
    pub id: String,
    pub name: String,
    pub language: String,
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

/// Selects the best Japanese voice for synthesis.
///
/// Preference order: an enhanced Kyoko voice (matches the macOS system voice
/// historically selected), any Kyoko voice, any enhanced Japanese voice, any
/// Japanese voice. Pure function — unit-testable without the plugin.
pub fn pick_japanese_voice(voices: &[Voice]) -> Option<&Voice> {
    let ja: Vec<&Voice> = voices
        .iter()
        .filter(|v| v.language.starts_with("ja"))
        .collect();
    let name_lower = |v: &&Voice| v.name.to_lowercase();

    ja.iter()
        .find(|v| name_lower(v).contains("kyoko") && name_lower(v).contains("enhanced"))
        .copied()
        .or_else(|| ja.iter().find(|v| name_lower(v).contains("kyoko")).copied())
        .or_else(|| {
            ja.iter()
                .find(|v| name_lower(v).contains("enhanced"))
                .copied()
        })
        .or_else(|| ja.first().copied())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn voice(id: &str, name: &str, lang: &str) -> Voice {
        Voice {
            id: id.to_string(),
            name: name.to_string(),
            language: lang.to_string(),
        }
    }

    #[test]
    fn pick_japanese_voice_prefers_enhanced_kyoko() {
        let voices = vec![
            voice("v1", "Otoya", "ja-JP"),
            voice("v2", "Kyoko", "ja-JP"),
            voice("v3", "Kyoko Enhanced", "ja-JP"),
        ];

        let picked = pick_japanese_voice(&voices);
        assert_eq!(picked.map(|v| v.id.as_str()), Some("v3"));
    }

    #[test]
    fn pick_japanese_voice_falls_back_to_any_japanese() {
        let voices = vec![
            voice("v1", "Samantha", "en-US"),
            voice("v2", "Otoya", "ja-JP"),
        ];

        let picked = pick_japanese_voice(&voices);
        assert_eq!(picked.map(|v| v.id.as_str()), Some("v2"));
    }

    #[test]
    fn pick_japanese_voice_returns_none_without_japanese() {
        let voices = vec![voice("v1", "Samantha", "en-US")];

        assert!(pick_japanese_voice(&voices).is_none());
    }

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
        assert!(caps.speech_synthesis.available);
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
