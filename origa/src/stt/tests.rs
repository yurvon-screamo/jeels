use super::common::strip_trailing_repeats;

#[test]
fn strip_trailing_repeats_no_repeat() {
    assert_eq!(strip_trailing_repeats("こんにちは"), "こんにちは");
}

#[test]
fn strip_trailing_repeats_double_char() {
    assert_eq!(strip_trailing_repeats("hellooo"), "hell");
}

#[test]
fn strip_trailing_repeats_triple_char() {
    assert_eq!(strip_trailing_repeats("aaaa"), "");
}

#[test]
fn strip_trailing_repeats_short() {
    assert_eq!(strip_trailing_repeats("ab"), "ab");
    assert_eq!(strip_trailing_repeats("a"), "a");
    assert_eq!(strip_trailing_repeats(""), "");
}

#[test]
fn strip_trailing_repeats_different_last_chars() {
    assert_eq!(strip_trailing_repeats("abc"), "abc");
}

#[test]
fn strip_trailing_repeats_multiple_same() {
    assert_eq!(strip_trailing_repeats("ですですです"), "ですですです");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn whisper_transcriber_loads_models() {
    let model_dir = std::path::Path::new("tauri/resources/whisper");
    if !model_dir.join("encoder_model.onnx").exists() {
        println!("Skipping: whisper models not found at {:?}", model_dir);
        return;
    }

    let transcriber = super::whisper::WhisperTranscriber::new(model_dir);
    assert!(
        transcriber.is_ok(),
        "Failed to create WhisperTranscriber: {:?}",
        transcriber.err()
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn whisper_transcribe_audio() {
    let model_dir = std::path::Path::new("tauri/resources/whisper");
    let wav_path = std::path::Path::new("standard_sample1.wav");

    if !model_dir.join("encoder_model.onnx").exists() || !wav_path.exists() {
        println!("Skipping: whisper models or test audio not found");
        return;
    }

    let transcriber = super::whisper::WhisperTranscriber::new(model_dir).unwrap();
    let text = transcriber.transcribe(wav_path);
    assert!(text.is_ok(), "Transcription failed: {:?}", text.err());

    let result = text.unwrap();
    assert!(!result.is_empty(), "Transcription should not be empty");
    println!("Transcription result: {}", result);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn audio_load_wav_sample() {
    let wav_path = std::path::Path::new("standard_sample1.wav");
    if !wav_path.exists() {
        println!("Skipping: test WAV not found at {:?}", wav_path);
        return;
    }

    let samples = super::audio::load_wav(wav_path);
    assert!(samples.is_ok(), "Failed to load WAV: {:?}", samples.err());

    let result = samples.unwrap();
    assert_eq!(
        result.len(),
        480_000,
        "Should be padded/trimmed to 30s at 16kHz"
    );
}
