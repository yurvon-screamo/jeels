use anyhow::Result;

use crate::audio_content::AudioContent;
use crate::{AppConfig, pexels::PexelsVideo};

pub async fn concat_videos_and_audio(
    videos: Vec<PexelsVideo>,
    audio: AudioContent,
    video_content: &str,
    config: &AppConfig,
) -> Result<String> {
    tokio::fs::create_dir_all(&config.output_dir).await?;

    let t_id = ulid::Ulid::new();
    let audio_path = format!("audio_{}.wav", t_id);
    let output_path = format!("{}/{}.mp4", config.output_dir, t_id);

    tokio::fs::write(&audio_path, audio.content).await?;

    let mut video_paths = Vec::new();
    for (i, video) in videos.iter().enumerate() {
        let video_path = format!("video_{}_{}.mp4", t_id, i);
        tokio::fs::write(&video_path, &video.content).await?;
        video_paths.push(video_path);
    }

    let mut total_duration = 0.0;
    let mut used_videos = Vec::new();

    for (i, video_path) in video_paths.iter().enumerate() {
        let video_duration = get_duration(video_path).await?;
        if total_duration + video_duration >= audio.duration {
            let needed = audio.duration - total_duration;
            let trimmed = format!("trimmed_{}_{}.mp4", t_id, i);
            trim_video(video_path, &trimmed, needed).await?;
            used_videos.push(trimmed);
            break;
        } else {
            total_duration += video_duration;
            used_videos.push(video_path.clone());
        }
    }

    let list_path = format!("video_list_{}.txt", t_id);
    let mut list_content = String::new();
    for video in &used_videos {
        list_content.push_str(&format!("file '{}'\n", video));
    }
    tokio::fs::write(&list_path, list_content).await?;

    let concatenated_video = format!("concatenated_{}.mp4", t_id);
    concat_videos(&list_path, &concatenated_video).await?;

    add_audio_and_subtitles_to_video(
        &concatenated_video,
        &audio_path,
        &output_path,
        &video_content,
        audio.duration,
    )
    .await?;

    tokio::fs::remove_file(&audio_path).await?;
    for video_path in &video_paths {
        tokio::fs::remove_file(video_path).await?;
    }
    for video in &used_videos {
        if video.starts_with("trimmed_") {
            tokio::fs::remove_file(video).await?;
        }
    }
    tokio::fs::remove_file(&list_path).await?;
    tokio::fs::remove_file(&concatenated_video).await?;

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

async fn concat_videos(list_file: &str, output: &str) -> Result<()> {
    let output_cmd = tokio::process::Command::new("ffmpeg")
        .args(&[
            "-y", "-f", "concat", "-safe", "0", "-i", list_file, "-c:v", "libx264", "-preset",
            "fast", "-crf", "23", "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "128k", "-ar",
            "44100", output,
        ])
        .output()
        .await?;

    if !output_cmd.status.success() {
        return Err(anyhow::anyhow!(
            "ffmpeg concat failed: {}",
            String::from_utf8_lossy(&output_cmd.stderr)
        ));
    }

    Ok(())
}

fn format_duration_ass(seconds: f64) -> String {
    let secs = seconds.floor() as u64;
    let centisecs = ((seconds - secs as f64) * 100.0).round() as u32;

    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    format!("{}:{:02}:{:02}.{:02}", hours, minutes, seconds, centisecs)
}

async fn create_ass_file_timed(text: &str, total_duration_sec: f64) -> Result<String> {
    let phrases: Vec<String> = text
        .split('\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    let phrases = if phrases.is_empty() {
        vec![text.to_string()]
    } else {
        phrases
    };

    let phrase_count = phrases.len();
    let segment_duration = total_duration_sec / phrase_count as f64;

    let mut ass_content = String::new();

    ass_content.push_str("[Script Info]\n");
    ass_content.push_str("Title: Generated Subtitles\n");
    ass_content.push_str("ScriptType: v4.00+\n");
    ass_content.push_str("WrapStyle: 1\n");
    ass_content.push_str("ScaledBorderAndShadow: yes\n");
    ass_content.push_str("YCbCr Matrix: TV.601\n");
    ass_content.push_str("\n");

    ass_content.push_str("[V4+ Styles]\n");
    ass_content.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
    ass_content.push_str("Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,1,1,2,10,10,10,1\n");
    ass_content.push_str("\n");

    ass_content.push_str("[Events]\n");
    ass_content.push_str(
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
    );

    for (i, phrase) in phrases.into_iter().enumerate() {
        let start_sec = i as f64 * segment_duration;
        let end_sec = (i + 1) as f64 * segment_duration;

        let start = format_duration_ass(start_sec);
        let end = format_duration_ass(end_sec.min(total_duration_sec));

        ass_content.push_str(&format!(
            "Dialogue: 0,{},{},Default,,0,0,0,,{}\n",
            start,
            end,
            phrase.replace("\n", " ")
        ));
    }

    let ass_path = format!("subtitles_{}.ass", ulid::Ulid::new());
    tokio::fs::write(&ass_path, ass_content).await?;
    Ok(ass_path)
}

async fn add_audio_and_subtitles_to_video(
    video: &str,
    audio: &str,
    output: &str,
    subtitle_text: &str,
    audio_duration: f64,
) -> Result<()> {
    let ass_path = create_ass_file_timed(subtitle_text, audio_duration).await?;

    let output_cmd = tokio::process::Command::new("ffmpeg")
        .args(&[
            "-y",
            "-i",
            video,
            "-i",
            audio,
            "-vf",
            &format!("ass={}", ass_path),
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
            "ffmpeg add audio + subtitles failed: {}",
            String::from_utf8_lossy(&output_cmd.stderr)
        ));
    }

    // tokio::fs::remove_file(&ass_path).await?;
    Ok(())
}
