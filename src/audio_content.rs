use anyhow::Result;

use crate::{AppConfig, ffprobe::get_duration};

pub struct AudioContent {
    pub content: Vec<u8>,
    pub duration: f64,
}

pub async fn generate_audio(video_content: &str, config: &AppConfig) -> Result<AudioContent> {
    // let video_content = video_content.replace("\n", "　");

    let t_id = ulid::Ulid::new();
    let tokens_path = format!("tokens_{}.npy", t_id);
    let output_path = format!("output_{}.wav", t_id);

    let llama_output = tokio::process::Command::new("./llama_generate")
        .arg("--text")
        .arg(video_content)
        .arg("--out-path")
        .arg(&tokens_path)
        .arg("--checkpoint")
        .arg(&config.fish_speech_checkpoint)
        .output()
        .await?;

    llama_output
        .status
        .success()
        .then_some(t_id)
        .ok_or(anyhow::anyhow!(
            "Llama generate failed {}",
            String::from_utf8_lossy(&llama_output.stderr)
        ))?;

    let vocoder_output = tokio::process::Command::new("./vocoder")
        .arg("-i")
        .arg(&tokens_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--checkpoint")
        .arg(&config.fish_speech_checkpoint)
        .output()
        .await?;

    vocoder_output
        .status
        .success()
        .then_some(t_id)
        .ok_or(anyhow::anyhow!(
            "Vocoder failed {}",
            String::from_utf8_lossy(&vocoder_output.stderr)
        ))?;

    let audio = tokio::fs::read(&output_path).await?;

    tokio::fs::remove_file(&tokens_path).await?;
    tokio::fs::remove_file(&output_path).await?;

    Ok(AudioContent {
        duration: get_audio_duration(&audio).await?,
        content: audio,
    })
}

async fn get_audio_duration(audio: &[u8]) -> Result<f64> {
    let t_id = ulid::Ulid::new();
    let temp_audio_path = format!("temp_audio_{}.wav", t_id);

    tokio::fs::write(&temp_audio_path, audio).await?;
    let duration = get_duration(&temp_audio_path).await?;
    tokio::fs::remove_file(&temp_audio_path).await?;

    Ok(duration)
}
