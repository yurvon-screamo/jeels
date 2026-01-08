pub mod dictionary;
pub mod error;
pub mod furiganizer;
pub mod grammar;
pub mod japanese;
pub mod knowledge;
pub mod review;
pub mod settings;
pub mod tokenizer;
pub mod value_objects;

use crate::domain::{
    knowledge::{Card, KnowledgeSet, StudyCard},
    review::MemoryState,
    value_objects::{JapaneseLevel, NativeLanguage},
};
use chrono::Duration;
pub use error::KeikakuError;
pub use review::Review;
use serde::{Deserialize, Serialize};
pub use settings::{LlmSettings, UserSettings};
use ulid::Ulid;
pub use value_objects::Rating;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: Ulid,
    username: String,
    native_language: NativeLanguage,
    current_japanese_level: JapaneseLevel,
    settings: UserSettings,
    knowledge_set: KnowledgeSet,
}

impl User {
    pub fn new(
        username: String,
        current_japanese_level: JapaneseLevel,
        native_language: NativeLanguage,
    ) -> Self {
        Self {
            id: Ulid::new(),
            username,
            knowledge_set: KnowledgeSet::new(),
            current_japanese_level,
            native_language,
            settings: UserSettings::empty(),
        }
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn current_japanese_level(&self) -> &JapaneseLevel {
        &self.current_japanese_level
    }

    pub fn native_language(&self) -> &NativeLanguage {
        &self.native_language
    }

    pub fn knowledge_set(&self) -> &KnowledgeSet {
        &self.knowledge_set
    }

    pub fn settings(&self) -> &UserSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut UserSettings {
        &mut self.settings
    }

    pub fn rate_card(
        &mut self,
        card_id: Ulid,
        rating: Rating,
        interval: Duration,
        memory_state: MemoryState,
    ) -> Result<(), KeikakuError> {
        self.knowledge_set
            .rate_card(card_id, rating, interval, memory_state)?;
        Ok(())
    }

    pub fn add_lesson_duration(&mut self, lesson_duration: Duration) {
        self.knowledge_set.add_lesson_duration(lesson_duration);
    }

    pub fn delete_card(&mut self, card_id: Ulid) -> Result<(), KeikakuError> {
        self.knowledge_set.delete_card(card_id)
    }

    pub fn create_card(&mut self, card: Card) -> Result<StudyCard, KeikakuError> {
        self.knowledge_set.create_card(card)
    }
}
