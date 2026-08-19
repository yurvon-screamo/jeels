use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tokenizer")]
#[command(about = "Unified CLI for Japanese tokenization and OCR tools", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Tokenize Japanese text and extract vocabulary words
    Tokenize {
        /// Text to tokenize or path to file
        text: String,

        /// Read text from file
        #[arg(short, long)]
        file: bool,
    },

    /// Japanese OCR using NDLOCR-Lite models
    Ndlocr {
        /// Input image path
        #[arg(short, long)]
        input: PathBuf,

        /// Detector model path
        #[arg(long, default_value = "../ndlocr-lite/src/model/deim-s-1024x1024.onnx")]
        detector: PathBuf,

        /// Parseq 30 model path
        #[arg(
            long,
            default_value = "../ndlocr-lite/src/model/parseq-ndl-16x256-30-tiny-192epoch-tegaki3.onnx"
        )]
        rec30: PathBuf,

        /// Parseq 50 model path
        #[arg(
            long,
            default_value = "../ndlocr-lite/src/model/parseq-ndl-16x384-50-tiny-146epoch-tegaki2.onnx"
        )]
        rec50: PathBuf,

        /// Parseq 100 model path
        #[arg(
            long,
            default_value = "../ndlocr-lite/src/model/parseq-ndl-16x768-100-tiny-165epoch-tegaki2.onnx"
        )]
        rec100: PathBuf,

        /// Vocabulary file path
        #[arg(long, default_value = "config/NDLmoji.txt")]
        vocab: PathBuf,
    },

    /// Batch process JSON files in well_known_set format
    TokenizeWellKnown {
        /// Path to directory or JSON file
        path: PathBuf,
    },

    /// Find vocabulary words missing from dictionary and generate translations
    FindMissing {
        /// Output path for the markdown report (default: missing_vocabulary.md in project root)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Auto-generate missing words with translations
        #[arg(short, long)]
        generate: bool,

        /// OpenAI API base URL
        #[arg(
            long,
            env = "ORIGA_API_URL",
            default_value = "http://localhost:8001/v1"
        )]
        api_base: String,

        /// OpenAI API key
        #[arg(long, default_value = "none")]
        api_key: String,

        /// Model name for the OpenAI-compatible endpoint
        #[arg(long, default_value = "llm")]
        model: String,

        /// Number of concurrent translation requests
        #[arg(short = 'w', long, default_value = "32")]
        workers: usize,

        /// Chunk size for processing
        #[arg(long, default_value = "512")]
        chunk_size: usize,

        /// Only translate to Russian
        #[arg(long)]
        russian_only: bool,

        /// Only translate to English
        #[arg(long)]
        english_only: bool,
    },

    /// Build phrase dataset from transcriptions
    BuildPhraseDataset {
        /// Input JSON file with transcriptions
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory for results
        #[arg(short, long)]
        output: PathBuf,

        /// Minimum vocabulary tokens per phrase
        #[arg(long, default_value = "2")]
        min_tokens: usize,
    },

    /// Validate vocabulary dictionary translations using LLM
    ValidateDictionary {
        /// OpenRouter API key (required, or set OPENROUTER_API_KEY env var)
        #[arg(long, env = "OPENROUTER_API_KEY")]
        api_key: String,

        /// OpenAI-compatible API base URL
        #[arg(long, default_value = "https://openrouter.ai/api/v1")]
        api_base: String,

        /// LLM model to use
        #[arg(long, default_value = "google/gemini-2.0-flash-001")]
        model: String,

        /// Number of concurrent validation requests
        #[arg(short = 'w', long, default_value = "8")]
        workers: usize,

        /// Output path for JSONL progress file (default: invalid_vocabulary.jsonl in project root)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show what would be done without making API calls
        #[arg(long)]
        dry_run: bool,

        /// Limit number of words to validate
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Print a grammar prompt to stdout without calling the LLM
    GenerateGrammarPrompt {
        /// Grammar pattern title (e.g. ～ます, ～てください)
        #[arg(short, long)]
        title: String,

        /// JLPT level (N5, N4, N3, N2, N1)
        #[arg(short, long, default_value = "N5")]
        level: String,

        /// Optional rule name from grammar index for additional context
        #[arg(long)]
        rule_name_from_index: Option<String>,
    },

    /// Generate grammar rule descriptions using LLM
    GenerateGrammar {
        /// Rule ID to generate (omit with --all for batch mode)
        rule_id: Option<String>,

        /// Generate descriptions for all rules
        #[arg(long)]
        all: bool,

        /// Rule indices to regenerate (comma-separated, e.g. "153,176,202,193")
        #[arg(long)]
        indices: Option<String>,

        /// Filter by JLPT level (N5, N4, N3) — use with --all
        #[arg(long)]
        level: Option<String>,

        /// OpenAI-compatible API base URL
        #[arg(
            long,
            env = "ORIGA_API_URL",
            default_value = "http://localhost:8001/v1"
        )]
        api_base: String,

        /// API key
        #[arg(long, default_value = "none")]
        api_key: String,

        /// LLM model to use (e.g. "minimax/minimax-m2.5:free")
        #[arg(long)]
        model: Option<String>,

        /// Enable reasoning with high effort
        #[arg(long)]
        reasoning: bool,

        /// Number of concurrent workers (0 = sequential)
        #[arg(short = 'w', long, default_value = "1")]
        workers: usize,

        /// Show what would be done without making changes
        #[arg(long)]
        dry_run: bool,

        /// Custom path to grammar.json
        #[arg(long)]
        grammar_path: Option<PathBuf>,
    },

    /// Remove duplicate readings from kanji dictionary
    DedupKanjiReadings {
        /// Path to kanji.json
        #[arg(short, long, default_value = "cdn/dictionary/kanji.json")]
        input: PathBuf,

        /// Show what would be changed without writing
        #[arg(long)]
        dry_run: bool,
    },

    /// Apply targeted reading removal patches to kanji dictionary
    PatchKanjiReadings {
        /// Path to kanji.json
        #[arg(short, long, default_value = "cdn/dictionary/kanji.json")]
        input: PathBuf,

        /// Path to patches JSON file
        #[arg(short, long, default_value = "cdn/dictionary/kanji_patches.json")]
        patches: PathBuf,

        /// Show what would be changed without writing
        #[arg(long)]
        dry_run: bool,
    },

    /// Regenerate translations for invalid vocabulary words
    RegenerateInvalid {
        /// Path to JSONL progress file from validate-dictionary
        #[arg(short, long)]
        input: PathBuf,

        /// OpenAI-compatible API base URL
        #[arg(
            long,
            env = "ORIGA_API_URL",
            default_value = "http://localhost:8001/v1"
        )]
        api_base: String,

        /// API key
        #[arg(long, default_value = "none")]
        api_key: String,

        /// Number of concurrent translation requests
        #[arg(short = 'w', long, default_value = "8")]
        workers: usize,

        /// Show what would be done without making changes
        #[arg(long)]
        dry_run: bool,

        /// Only translate to Russian
        #[arg(long)]
        russian_only: bool,

        /// Only translate to English
        #[arg(long)]
        english_only: bool,
    },

    /// Re-tokenize phrase_index.json tokens with the current tokenizer
    RetokenizePhrases {
        /// Path to phrase_index.json
        #[arg(long)]
        index: PathBuf,

        /// Directory containing data_bundle_*.json (phrase texts)
        #[arg(long)]
        data_dir: PathBuf,
    },

    /// Enrich phrase index with grammar rule detection
    EnrichPhrasesWithGrammar {
        /// Input phrase_index.json path
        #[arg(short, long)]
        input: PathBuf,

        /// Directory containing chunk data files (p*.json)
        #[arg(long)]
        chunks_dir: PathBuf,

        /// Path to grammar.json
        #[arg(short, long)]
        grammar: PathBuf,

        /// Output enriched phrase_index.json path
        #[arg(short, long)]
        output: PathBuf,

        /// Dictionary directory for tokenizer (optional, uses default path)
        #[arg(long)]
        dictionary_dir: Option<PathBuf>,
    },
}
