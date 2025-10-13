use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use serde::Deserialize;

mod video_content;

#[derive(Clone, Deserialize)]
pub struct AppConfig {
    pub openai_api_key: String,
    pub openai_model: String,
    pub openai_api_base: String,
}

impl AppConfig {
    fn load() -> Result<Self> {
        config::Config::builder()
            .add_source(config::File::with_name("config.toml"))
            .build()?
            .try_deserialize()
            .context("Failed to parse config.toml")
    }
}

#[derive(Parser, Debug)]
#[command(version, about = "🎬 Jeels - Генератор обучающих видео", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// Generate video for learning new words
    Words { words: Vec<String> },
}

#[derive(Debug, Clone)]
struct PreparedContent {
    topic: String,
    content: String,
}

async fn prepare_content(command: &Commands, config: &AppConfig) -> Result<PreparedContent> {
    match command {
        Commands::Words { words } => Ok(PreparedContent {
            topic: words.join(", "),
            content: video_content::generate_video_content(&words.join(", "), config).await?,
        }),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!(
        "{}",
        "🎬 Jeels - Generate learning lesson for japanese language"
            .bright_cyan()
            .bold()
    );
    println!("{}", "=".repeat(50).bright_blue());

    let config = AppConfig::load()?;
    let args = Args::parse();

    let prepared = prepare_content(&args.command, &config).await?;
    println!(
        "{}. {}",
        prepared.topic.bright_blue().bold(),
        prepared.content.bright_green().bold()
    );

    Ok(())
}
