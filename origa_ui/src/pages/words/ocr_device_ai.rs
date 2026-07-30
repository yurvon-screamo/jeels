//! device-ai native OCR path.
//!
//! When the native text-recognition capability is available, OCR runs through
//! the platform's on-device API (no model download) and returns immediately.
//! Any failure (capability unavailable, invoke error, timeout) yields `None`
//! so the caller transparently falls back to the WASM NDLOCR pipeline.

use crate::core::device_ai::{self, Feature};
use crate::ui_components::{OcrLoadingStage, OcrLoadingState};
use leptos::prelude::*;
use tracing::warn;

/// Attempts native OCR on a base64-encoded image. Returns `Some(text)` on
/// success, or `None` when native OCR is unavailable or fails — signalling
/// the caller to use the WASM fallback.
pub(super) async fn recognize_via_device_ai(
    base64_data: &str,
    loading_state: &OcrLoadingState,
) -> Option<String> {
    if !device_ai::available(Feature::TextRecognition).await {
        return None;
    }

    // No model download or initialization stage for native OCR — jump straight
    // to recognition so the UI reflects the actual work.
    loading_state.stage.set(OcrLoadingStage::Recognizing);

    match device_ai::recognize_text(base64_data).await {
        Ok(result) => Some(result.text),
        Err(e) => {
            warn!("device-ai OCR failed, falling back to WASM: {e}");
            None
        },
    }
}
