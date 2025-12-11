use serde::{Deserialize, Serialize};
use std::fmt;
use ulid::Ulid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JeersError {
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
}

impl fmt::Display for JeersError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JeersError::UserNotFound { user_id } => {
                write!(f, "User with id {} not found", user_id)
            }
            JeersError::UserNotFoundByUsername { username } => {
                write!(f, "User with username {} not found", username)
            }
            JeersError::CardNotFound { card_id } => {
                write!(f, "Card with id {} not found", card_id)
            }
            JeersError::DuplicateCard { question } => {
                write!(f, "Card with question '{}' already exists", question)
            }
            JeersError::InvalidQuestion { reason } => {
                write!(f, "Invalid question: {}", reason)
            }
            JeersError::InvalidAnswer { reason } => {
                write!(f, "Invalid answer: {}", reason)
            }
            JeersError::InvalidStability { reason } => {
                write!(f, "Invalid stability: {}", reason)
            }
            JeersError::InvalidDifficulty { reason } => {
                write!(f, "Invalid difficulty: {}", reason)
            }
            JeersError::InvalidMemoryState { reason } => {
                write!(f, "Invalid memory state: {}", reason)
            }
            JeersError::SrsCalculationFailed { reason } => {
                write!(f, "SRS calculation failed: {}", reason)
            }
            JeersError::RepositoryError { reason } => {
                write!(f, "Repository error: {}", reason)
            }
            JeersError::EmbeddingError { reason } => {
                write!(f, "Embedding error: {}", reason)
            }
            JeersError::LlmError { reason } => {
                write!(f, "LLM error: {}", reason)
            }
            JeersError::SettingsError { reason } => {
                write!(f, "Settings error: {}", reason)
            }
            JeersError::FuriganaError { reason } => {
                write!(f, "Furigana error: {}", reason)
            }
            JeersError::TranslationError { reason } => {
                write!(f, "Translation error: {}", reason)
            }
            JeersError::KradfileError { reason } => {
                write!(f, "Kradfile error: {}", reason)
            }
            JeersError::InvalidValues { reason } => {
                write!(f, "Invalid values: {}", reason)
            }
        }
    }
}

impl std::error::Error for JeersError {}
