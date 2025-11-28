use crate::domain::{Card, error::JeersError};
use serde::{Deserialize, Serialize};
use std::fmt;
use ulid::Ulid;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudySessionItem {
    card_id: Ulid,
    answer: String,
    question: String,
    furigana: Option<String>,
    similarity: Vec<StudySessionItem>,
}

impl StudySessionItem {
    pub fn new(card_id: Ulid, answer: String, question: String) -> Self {
        Self {
            card_id,
            answer,
            question,
            furigana: None,
            similarity: vec![],
        }
    }

    pub fn set_furigana(&mut self, furigana: String) {
        self.furigana = Some(furigana);
    }

    pub fn set_similarity(&mut self, similarity: &[Card]) {
        self.similarity = similarity
            .iter()
            .map(|card| {
                StudySessionItem::new(
                    card.id(),
                    card.answer().text().to_string(),
                    card.question().text().to_string(),
                )
            })
            .collect();
    }

    pub fn card_id(&self) -> Ulid {
        self.card_id
    }

    pub fn answer(&self) -> &str {
        &self.answer
    }

    pub fn question(&self) -> &str {
        &self.question
    }

    pub fn furigana(&self) -> Option<&str> {
        self.furigana.as_deref()
    }

    pub fn similarity(&self) -> &Vec<StudySessionItem> {
        &self.similarity
    }

    pub fn set_similarity_furigana(&mut self, card_id: Ulid, furigana: String) {
        for similarity in self.similarity.iter_mut() {
            if similarity.card_id() == card_id {
                similarity.set_furigana(furigana);
                break;
            }
        }
    }
}
