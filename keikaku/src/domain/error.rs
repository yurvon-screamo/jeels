use serde::{Deserialize, Serialize};
use std::fmt;
use ulid::Ulid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeikakuError {
    UserNotFound { user_id: Ulid },
    UserNotFoundByUsername { username: String },
    CardNotFound { card_id: Ulid },
    DuplicateCard { question: String },
    InvalidQuestion { reason: String },
    InvalidAnswer { reason: String },
    InvalidStability { reason: String },
    InvalidDifficulty { reason: String },
    InvalidMemoryState { reason: String },
    SrsCalculationFailed { reason: String },
    RepositoryError { reason: String },
    EmbeddingError { reason: String },
    LlmError { reason: String },
    SettingsError { reason: String },
    FuriganaError { reason: String },
    TranslationError { reason: String },
    KradfileError { reason: String },
    InvalidValues { reason: String },
    TokenizerError { reason: String },
    GrammarFormatError { reason: String },
}

impl fmt::Display for KeikakuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeikakuError::UserNotFound { user_id } => {
                write!(f, "User with id {} not found", user_id)
            }
            KeikakuError::UserNotFoundByUsername { username } => {
                write!(f, "User with username {} not found", username)
            }
            KeikakuError::CardNotFound { card_id } => {
                write!(f, "Card with id {} not found", card_id)
            }
            KeikakuError::DuplicateCard { question } => {
                write!(f, "Card with question '{}' already exists", question)
            }
            KeikakuError::InvalidQuestion { reason } => {
                write!(f, "Invalid question: {}", reason)
            }
            KeikakuError::InvalidAnswer { reason } => {
                write!(f, "Invalid answer: {}", reason)
            }
            KeikakuError::InvalidStability { reason } => {
                write!(f, "Invalid stability: {}", reason)
            }
            KeikakuError::InvalidDifficulty { reason } => {
                write!(f, "Invalid difficulty: {}", reason)
            }
            KeikakuError::InvalidMemoryState { reason } => {
                write!(f, "Invalid memory state: {}", reason)
            }
            KeikakuError::SrsCalculationFailed { reason } => {
                write!(f, "SRS calculation failed: {}", reason)
            }
            KeikakuError::RepositoryError { reason } => {
                write!(f, "Repository error: {}", reason)
            }
            KeikakuError::EmbeddingError { reason } => {
                write!(f, "Embedding error: {}", reason)
            }
            KeikakuError::LlmError { reason } => {
                write!(f, "LLM error: {}", reason)
            }
            KeikakuError::SettingsError { reason } => {
                write!(f, "Settings error: {}", reason)
            }
            KeikakuError::FuriganaError { reason } => {
                write!(f, "Furigana error: {}", reason)
            }
            KeikakuError::TranslationError { reason } => {
                write!(f, "Translation error: {}", reason)
            }
            KeikakuError::KradfileError { reason } => {
                write!(f, "Kradfile error: {}", reason)
            }
            KeikakuError::InvalidValues { reason } => {
                write!(f, "Invalid values: {}", reason)
            }
            KeikakuError::TokenizerError { reason } => {
                write!(f, "Tokenizer error: {}", reason)
            }
            KeikakuError::GrammarFormatError { reason } => {
                write!(f, "Grammar rule format error: {}", reason)
            }
        }
    }
}

impl std::error::Error for KeikakuError {}
