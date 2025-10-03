use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use serde::Deserialize;

use crate::{
    audio_content::generate_audio, background::pick_video_from_background,
    ffprobe::concat_videos_and_audio, video_content::generate_video_content,
};

mod audio_content;
mod background;
mod ffprobe;
mod pexels;
mod video_content;

struct ProgressTracker {
    main_pb: ProgressBar,
}

impl ProgressTracker {
    fn new() -> Self {
        let main_pb = ProgressBar::new(4);
        main_pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );

        Self { main_pb }
    }

    fn step(&self, step: u64, message: &str) {
        self.main_pb.set_position(step);
        self.main_pb.set_message(message.to_owned());
    }

    fn finish(&self, message: &str) {
        self.main_pb.finish_with_message(message.to_owned());
    }
}

fn create_output_structure(user_input: &str, config: &AppConfig) -> Result<PathBuf> {
    let now = chrono::Utc::now();
    let datetime_str = now.format("%Y%m%d_%H%M%S").to_string();

    let sanitized_input = user_input
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .replace(' ', "_");

    let folder_name = format!("{}_{}", datetime_str, sanitized_input);
    let output_path = PathBuf::from(&config.output_dir)
        .join("words")
        .join(folder_name);

    std::fs::create_dir_all(&output_path)?;

    Ok(output_path)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!(
        "{}",
        "🎬 Jeels - Генератор обучающих видео".bright_cyan().bold()
    );
    println!("{}", "=".repeat(50).bright_blue());

    let config = AppConfig::new()?;
    let args = Args::parse();

    let topic = match &args.command {
        Commands::Words { words } => {
            format!("{}", words.join(", "))
        }
        Commands::Grammar { .. } => todo!(),
    };

    let output_dir = create_output_structure(&topic, &config)?;

    let handles: Vec<_> = (0..config.concurrent_videos)
        .map(|i| {
            let config = config.clone();
            let topic = topic.clone();
            let output_dir = output_dir.clone();
            tokio::spawn(async move {
                let progress = ProgressTracker::new();
                generate(i + 1, topic, config, progress, output_dir).await
            })
        })
        .collect();

    let mut successful_videos = Vec::new();
    for (i, handle) in handles.into_iter().enumerate() {
        match handle.await? {
            Ok(output_path) => {
                println!(
                    "{}",
                    format!("🎬 Видео {} готово: {}", i + 1, output_path)
                        .bright_green()
                        .bold()
                );
                successful_videos.push(output_path);
            }
            Err(e) => {
                println!(
                    "{}",
                    format!("❌ Ошибка при генерации видео {}: {}", i + 1, e)
                        .bright_red()
                        .bold()
                );
            }
        }
    }

    println!("\n{}", "=".repeat(50).bright_blue());
    println!(
        "{}",
        format!(
            "🎉 Успешно создано {} видео из {}",
            successful_videos.len(),
            config.concurrent_videos
        )
        .bright_green()
        .bold()
    );

    Ok(())
}

async fn generate(
    video_number: usize,
    topic: String,
    config: AppConfig,
    progress: ProgressTracker,
    output_dir: PathBuf,
) -> Result<String> {
    // Step 1: Generate content plan
    progress.step(
        0,
        &format!(
            "\n🤖 Генерируем план контента для видео {}...",
            video_number
        ),
    );
    let video_content = generate_video_content(&topic, &config).await?;
    println!(
        "\n{}",
        format!("📝 Сгенерированный контент для видео {}:", video_number)
            .bright_yellow()
            .bold()
    );
    println!("{}", video_content.bright_white());

    // Step 2: Generate audio
    progress.step(
        1,
        &format!("\n🎵 Генерируем аудио для видео {}...", video_number),
    );
    let audio = generate_audio(&video_content, &config).await?;
    println!(
        "\n{}",
        format!("✅ Аудио сгенерировано для видео {}", video_number).bright_green()
    );

    // Step 3: Pick videos
    progress.step(
        2,
        &format!(
            "\n🎥 Ищем видео в папке background для видео {}...",
            video_number
        ),
    );
    let videos = pick_video_from_background(&config).await?;
    println!(
        "\n{}",
        format!("✅ Найдено видео для видео {}", video_number).bright_green()
    );

    // Step 4: Concatenate videos and audio
    progress.step(
        3,
        &format!("\n🎬 Создаем финальное видео {}...", video_number),
    );
    let output_path = concat_videos_and_audio(videos, audio, &output_dir).await?;

    create_readme(&output_dir, &topic, &video_content).await?;

    progress.finish(&format!("\n🎉 Видео {} готово!", video_number));

    Ok(output_path)
}

async fn create_readme(output_dir: &std::path::Path, topic: &str, content: &str) -> Result<()> {
    let readme_content = format!(
        "# Обучение японскому языку: {}\n\n{}\n\n## Содержание урока\n\n{}\n\n## Файлы\n\n- `video.mp4` - Финальное видео\n- `audio.wav` - Аудио файл\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        topic,
        content,
    );

    let readme_path = output_dir.join("README.md");
    tokio::fs::write(&readme_path, readme_content).await?;

    Ok(())
}

#[derive(Clone, Deserialize)]
struct FishSpeechPrompt {
    tokens: String,
    text: String,
}

#[derive(Clone, Deserialize)]
struct AppConfig {
    openai_api_key: String,
    openai_model: String,
    openai_api_base: String,

    pexels_api_key: String,
    pexels_keywords: Vec<String>,
    pexels_per_page: usize,
    pexels_total: usize,

    fish_speech_checkpoint: String,
    fish_speech_prompts: Vec<FishSpeechPrompt>,

    concurrent_videos: usize,
    output_dir: String,
    background_dir: String,
}

impl AppConfig {
    fn new() -> Result<Self> {
        let config_path = "config.toml";

        let config = config::Config::builder()
            .add_source(config::File::new(config_path, config::FileFormat::Toml))
            .build()?
            .try_deserialize::<Self>()?;

        Ok(config)
    }
}

/// Jeels - Reels-like video generator for learning japanese.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate video for learning a new words
    Words {
        /// The words to learn
        words: Vec<String>,
    },
    /// Generate video for learning grammar
    Grammar {
        /// The grammar topic to learn
        grammar: String,
    },
}
