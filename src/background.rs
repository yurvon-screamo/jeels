use anyhow::Result;
use rand::seq::SliceRandom;
use std::fs;
use std::path::Path;

use crate::AppConfig;

pub struct BackgroundVideo {
    pub content: Vec<u8>,
}

pub async fn pick_video_from_background(config: &AppConfig) -> Result<BackgroundVideo> {
    let background_dir = Path::new(&config.background_dir);

    if !background_dir.exists() {
        return Err(anyhow::anyhow!("Background directory not found"));
    }

    let mut video_files = Vec::new();

    let entries = fs::read_dir(background_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if matches!(
                    extension.to_str(),
                    Some("webm") | Some("mp4") | Some("avi") | Some("mov")
                ) {
                    video_files.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    if video_files.is_empty() {
        return Err(anyhow::anyhow!(
            "No video files found in background directory"
        ));
    }

    video_files.shuffle(&mut rand::rng());

    for video_path in video_files {
        let content = fs::read(&video_path)?;
        return Ok(BackgroundVideo { content });
    }

    Err(anyhow::anyhow!("No suitable videos found"))
}
