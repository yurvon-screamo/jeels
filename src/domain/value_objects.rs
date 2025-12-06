use crate::domain::error::JeersError;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Embedding(pub Vec<f32>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Question {
    text: String,
    embedding: Vec<f32>,
}

impl Question {
    pub fn new(text: String, embedding: Embedding) -> Result<Self, JeersError> {
        if text.trim().is_empty() {
            return Err(JeersError::InvalidQuestion {
                reason: "Question text cannot be empty".to_string(),
            });
        }
        Ok(Self {
            text,
            embedding: embedding.0,
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn embedding(&self) -> &Vec<f32> {
        &self.embedding
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Answer {
    text: String,
}

impl Answer {
    pub fn new(text: String) -> Result<Self, JeersError> {
        if text.trim().is_empty() {
            return Err(JeersError::InvalidAnswer {
                reason: "Answer text cannot be empty".to_string(),
            });
        }
        Ok(Self { text })
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rating {
    Easy,
    Good,
    Hard,
    Again,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Stability {
    value: f64,
}

impl Stability {
    pub fn new(value: f64) -> Result<Self, JeersError> {
        if value < 0.0 {
            return Err(JeersError::InvalidStability {
                reason: "Stability cannot be negative".to_string(),
            });
        }
        Ok(Self { value })
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

impl fmt::Display for Stability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Difficulty {
    value: f64,
}

impl Difficulty {
    pub fn new(value: f64) -> Result<Self, JeersError> {
        if value < 0.0 {
            return Err(JeersError::InvalidDifficulty {
                reason: "Difficulty cannot be negative".to_string(),
            });
        }
        Ok(Self { value })
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MemoryState {
    stability: Stability,
    difficulty: Difficulty,
}

impl MemoryState {
    pub fn new(stability: Stability, difficulty: Difficulty) -> Self {
        Self {
            stability,
            difficulty,
        }
    }

    pub fn stability(&self) -> Stability {
        self.stability
    }

    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }
}

impl fmt::Display for MemoryState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Stability: {}, Difficulty: {}",
            self.stability, self.difficulty
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JapaneseLevel {
    N5,
    N4,
    N3,
    N2,
    N1,
}

impl JapaneseLevel {
    pub fn as_number(&self) -> u8 {
        match self {
            JapaneseLevel::N5 => 5,
            JapaneseLevel::N4 => 4,
            JapaneseLevel::N3 => 3,
            JapaneseLevel::N2 => 2,
            JapaneseLevel::N1 => 1,
        }
    }
}

impl fmt::Display for JapaneseLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_number())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeLanguage {
    English,
    Russian,
}

impl NativeLanguage {
    pub fn as_str(&self) -> &str {
        match self {
            NativeLanguage::English => "English",
            NativeLanguage::Russian => "Russian",
        }
    }
}

impl fmt::Display for NativeLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExamplePhrase {
    text: String,
    translation: String,
}

impl ExamplePhrase {
    pub fn new(text: String, translation: String) -> Self {
        Self { text, translation }
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn translation(&self) -> &String {
        &self.translation
    }
}
