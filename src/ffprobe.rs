use anyhow::Result;

use crate::audio_content::AudioContent;
use crate::{AppConfig, background::BackgroundVideo, pexels::PexelsVideo};

pub trait VideoContent {
    fn content(&self) -> &[u8];
}

impl VideoContent for PexelsVideo {
    fn content(&self) -> &[u8] {
        &self.content
    }
}

impl VideoContent for BackgroundVideo {
    fn content(&self) -> &[u8] {
        &self.content
    }
}

pub async fn concat_videos_and_audio<T: VideoContent>(
    video: T,
    audio: AudioContent,
    config: &AppConfig,
) -> Result<String> {
    tokio::fs::create_dir_all(&config.output_dir).await?;

    let t_id = ulid::Ulid::new();
    let audio_path = format!("audio_{}.wav", t_id);
    let output_path = format!("{}/{}.mp4", config.output_dir, t_id);

    tokio::fs::write(&audio_path, audio.content).await?;

    let video_path = format!("video_{}.mp4", t_id);
    tokio::fs::write(&video_path, video.content()).await?;

    let used_video_path: String;

    let video_duration = get_duration(&video_path).await?;
    if video_duration >= audio.duration {
        let trimmed = format!("trimmed_{}.mp4", t_id);
        trim_video(&video_path, &trimmed, audio.duration).await?;
        used_video_path = trimmed;
    } else {
        used_video_path = video_path.clone();
    }

    add_audio_to_video(&used_video_path, &audio_path, &output_path).await?;

    tokio::fs::remove_file(&audio_path).await?;
    tokio::fs::remove_file(&video_path).await?;
    if used_video_path.starts_with("trimmed_") {
        tokio::fs::remove_file(&used_video_path).await?;
    }

    Ok(output_path)
}

pub async fn get_duration(file: &str) -> Result<f64> {
    let output = tokio::process::Command::new("ffprobe")
        .args(&[
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            file,
        ])
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let duration_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    duration_str
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse duration: {}", e))
}

async fn trim_video(input: &str, output: &str, duration: f64) -> Result<()> {
    let output_cmd = tokio::process::Command::new("ffmpeg")
        .args(&[
            "-y",
            "-i",
            input,
            "-t",
            &duration.to_string(),
            "-c:v",
            "libx264",
            "-preset",
            "fast",
            "-crf",
            "23",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            "-ar",
            "44100",
            output,
        ])
        .output()
        .await?;

    if !output_cmd.status.success() {
        return Err(anyhow::anyhow!(
            "ffmpeg trim failed: {}",
            String::from_utf8_lossy(&output_cmd.stderr)
        ));
    }

    Ok(())
}

async fn add_audio_to_video(video: &str, audio: &str, output: &str) -> Result<()> {
    let output_cmd = tokio::process::Command::new("ffmpeg")
        .args(&[
            "-y",
            "-i",
            video,
            "-i",
            audio,
            "-c:v",
            "libx264",
            "-preset",
            "fast",
            "-crf",
            "23",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            "-ar",
            "44100",
            "-map",
            "0:v:0",
            "-map",
            "1:a:0",
            "-shortest",
            output,
        ])
        .output()
        .await?;

    if !output_cmd.status.success() {
        return Err(anyhow::anyhow!(
            "ffmpeg add audio failed: {}",
            String::from_utf8_lossy(&output_cmd.stderr)
        ));
    }

    Ok(())
}
