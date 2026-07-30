//! Native file-based speech recognition via the device-ai crate.
//!
//! macOS-only: the device-ai Rust backend implements speech recognition on
//! macOS (and Windows, which this workspace excludes — see tauri/Cargo.toml).
//! The JS plugin surface only accepts live-microphone input, so file-based
//! recognition requires calling the Rust crate directly through this command.
//! On platforms where the command is absent, the frontend falls back to the
//! Whisper WASM transcriber.

use base64::{Engine, engine::general_purpose::STANDARD};
use device_ai::{AudioSource, DeviceAi, RecognitionOptions};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RecognizeFileArgs {
    /// Standard base64-encoded audio bytes (no data-URL prefix).
    pub base64: String,
    pub language: String,
}

#[derive(Serialize)]
pub struct RecognizeFileResult {
    pub text: String,
}

/// Recognize speech in an in-memory audio buffer using native macOS APIs.
///
/// The native `recognize` call is synchronous and may take seconds, so it runs
/// on a blocking thread to avoid starving Tauri's async command workers.
#[tauri::command]
pub async fn device_ai_recognize_file(
    args: RecognizeFileArgs,
) -> Result<RecognizeFileResult, String> {
    let bytes = STANDARD
        .decode(&args.base64)
        .map_err(|e| format!("device-ai recognize: base64 decode failed: {e:?}"))?;

    let options = RecognitionOptions::new()
        .with_language(args.language)
        .with_audio_source(AudioSource::from_bytes(bytes));

    let result =
        tauri::async_runtime::spawn_blocking(move || DeviceAi::new().speech().recognize(options))
            .await
            .map_err(|e| format!("device-ai recognize join failed: {e:?}"))?
            .map_err(|e| format!("device-ai recognize failed: {e:?}"))?;

    Ok(RecognizeFileResult { text: result.text })
}
