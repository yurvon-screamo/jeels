use crate::domain::error::KeikakuError;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Question {
    text: String,
}

impl Question {
    pub fn new(text: String) -> Result<Self, KeikakuError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(KeikakuError::InvalidQuestion {
                reason: "Question text cannot be empty".to_string(),
            });
        }

        Ok(Self {
            text: text.to_string(),
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Answer {
    text: String,
}

impl Answer {
    pub fn new(text: String) -> Result<Self, KeikakuError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(KeikakuError::InvalidAnswer {
                reason: "Answer text cannot be empty".to_string(),
            });
        }

        Ok(Self {
            text: text.to_string(),
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
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
    pub fn new(value: f64) -> Result<Self, KeikakuError> {
        if value < 0.0 {
            return Err(KeikakuError::InvalidStability {
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
    pub fn new(value: f64) -> Result<Self, KeikakuError> {
        if value < 0.0 {
            return Err(KeikakuError::InvalidDifficulty {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
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

    pub fn code(&self) -> &'static str {
        match self {
            JapaneseLevel::N5 => "N5",
            JapaneseLevel::N4 => "N4",
            JapaneseLevel::N3 => "N3",
            JapaneseLevel::N2 => "N2",
            JapaneseLevel::N1 => "N1",
        }
    }
}

impl fmt::Display for JapaneseLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_number())
    }
}

impl FromStr for JapaneseLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "N5" => Ok(JapaneseLevel::N5),
            "N4" => Ok(JapaneseLevel::N4),
            "N3" => Ok(JapaneseLevel::N3),
            "N2" => Ok(JapaneseLevel::N2),
            "N1" => Ok(JapaneseLevel::N1),
            other => Err(format!("Unknown Japanese level: {}", other)),
        }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Pronoun,
    Preposition,
    Conjunction,
    Interjection,
    Particle,
    Other,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JlptVocabularyEntry {
    level: JapaneseLevel,
    russian_translation: String,
    english_translation: String,
    russian_examples: Vec<ExamplePhrase>,
    english_examples: Vec<ExamplePhrase>,
    part_of_speech: PartOfSpeech,
}

impl JlptVocabularyEntry {
    pub fn new(
        level: JapaneseLevel,
        russian_translation: String,
        english_translation: String,
        russian_examples: Vec<ExamplePhrase>,
        english_examples: Vec<ExamplePhrase>,
        part_of_speech: PartOfSpeech,
    ) -> Self {
        Self {
            level,
            russian_translation,
            english_translation,
            russian_examples,
            english_examples,
            part_of_speech,
        }
    }

    pub fn level(&self) -> &JapaneseLevel {
        &self.level
    }

    pub fn russian_translation(&self) -> &str {
        &self.russian_translation
    }

    pub fn english_translation(&self) -> &str {
        &self.english_translation
    }

    pub fn russian_examples(&self) -> &[ExamplePhrase] {
        &self.russian_examples
    }

    pub fn english_examples(&self) -> &[ExamplePhrase] {
        &self.english_examples
    }

    pub fn part_of_speech(&self) -> &PartOfSpeech {
        &self.part_of_speech
    }
}
