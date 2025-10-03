use anyhow::Result;

use crate::{AppConfig, ffprobe::get_duration};

pub struct AudioContent {
    pub content: Vec<u8>,
    pub duration: f64,
}

pub async fn generate_audio(video_content: &str, config: &AppConfig) -> Result<AudioContent> {
    let sentences: Vec<&str> = video_content
        .split("---")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if sentences.is_empty() {
        return Ok(AudioContent {
            content: Vec::new(),
            duration: 0.0,
        });
    }

    let mut all_audio_chunks = Vec::new();
    let mut total_duration = 0.0;

    for (i, sentence) in sentences.iter().enumerate() {
        println!(
            "Processing sentence {}/{}: {}",
            i + 1,
            sentences.len(),
            sentence
        );

        let chunk_audio = generate_audio_chunk(sentence, config).await?;
        all_audio_chunks.push(chunk_audio.content);
        total_duration += chunk_audio.duration;
    }

    let merged_audio = merge_audio_chunks(all_audio_chunks).await?;

    Ok(AudioContent {
        content: merged_audio,
        duration: total_duration,
    })
}

async fn merge_audio_chunks(audio_chunks: Vec<Vec<u8>>) -> Result<Vec<u8>> {
    if audio_chunks.is_empty() {
        return Ok(Vec::new());
    }

    if audio_chunks.len() == 1 {
        return Ok(audio_chunks[0].clone());
    }

    let t_id = ulid::Ulid::new();
    let concat_list_path = format!("concat_list_{}.txt", t_id);
    let output_path = format!("merged_audio_{}.wav", t_id);

    let mut concat_list = String::new();
    for (i, chunk) in audio_chunks.iter().enumerate() {
        let chunk_path = format!("chunk_{}_{}.wav", t_id, i);
        tokio::fs::write(&chunk_path, chunk).await?;
        concat_list.push_str(&format!("file '{}'\n", chunk_path));
    }

    tokio::fs::write(&concat_list_path, concat_list).await?;

    let mut ffmpeg_process = tokio::process::Command::new("ffmpeg")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(&concat_list_path)
        .arg("-c")
        .arg("copy")
        .arg(&output_path)
        .arg("-y")
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()?;

    let ffmpeg_status = ffmpeg_process.wait().await?;

    if !ffmpeg_status.success() {
        return Err(anyhow::anyhow!(
            "FFmpeg concatenation failed with exit code: {:?}",
            ffmpeg_status.code()
        ));
    }

    let merged_audio = tokio::fs::read(&output_path).await?;

    tokio::fs::remove_file(&concat_list_path).await?;
    tokio::fs::remove_file(&output_path).await?;

    for i in 0..audio_chunks.len() {
        tokio::fs::remove_file(&format!("chunk_{}_{}.wav", t_id, i)).await?;
    }

    Ok(merged_audio)
}

async fn generate_audio_chunk(text: &str, config: &AppConfig) -> Result<AudioContent> {
    let t_id = ulid::Ulid::new();
    let tokens_path = format!("tokens_{}.npy", t_id);
    let output_path = format!("output_{}.wav", t_id);

    let prompt = if config.fish_speech_prompts.is_empty() {
        return Err(anyhow::anyhow!("No fish speech prompts configured"));
    } else {
        let index = fastrand::usize(..config.fish_speech_prompts.len());
        &config.fish_speech_prompts[index]
    };

    let mut llama_process = tokio::process::Command::new("./llama_generate")
        .arg("--text")
        .arg(text)
        .arg("--out-path")
        .arg(&tokens_path)
        .arg("--checkpoint")
        .arg(&config.fish_speech_checkpoint)
        .arg("--max-new-tokens")
        .arg("4096")
        .arg("--prompt-tokens")
        .arg(&prompt.tokens)
        .arg("--prompt-text")
        .arg(&prompt.text)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()?;

    let llama_status = llama_process.wait().await?;

    if !llama_status.success() {
        return Err(anyhow::anyhow!(
            "Llama generate failed with exit code: {:?}",
            llama_status.code()
        ));
    }

    let mut vocoder_process = tokio::process::Command::new("./vocoder")
        .arg("-i")
        .arg(&tokens_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--checkpoint")
        .arg(&config.fish_speech_checkpoint)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()?;

    let vocoder_status = vocoder_process.wait().await?;

    if !vocoder_status.success() {
        return Err(anyhow::anyhow!(
            "Vocoder failed with exit code: {:?}",
            vocoder_status.code()
        ));
    }

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
