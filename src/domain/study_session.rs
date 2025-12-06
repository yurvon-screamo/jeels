use crate::domain::{
    VocabularyCard,
    kanji::KanjiCard,
    value_objects::{ExamplePhrase, JapaneseLevel},
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudySessionItem {
    answer: String,
    question: String,

    card_id: Ulid,
    shuffled: bool,

    similarity: Vec<StudySessionItem>,
    homonyms: Vec<StudySessionItem>,

    example_phrases: Vec<ExamplePhrase>,
    kanji: Vec<KanjiCard>,

    level: JapaneseLevel,
}

impl StudySessionItem {
    pub fn new(
        card_id: Ulid,
        answer: String,
        question: String,
        shuffled: bool,
        example_phrases: Vec<ExamplePhrase>,
        kanji: Vec<KanjiCard>,
        level: JapaneseLevel,
    ) -> Self {
        Self {
            card_id,
            answer,
            question,
            similarity: vec![],
            homonyms: vec![],
            shuffled,
            example_phrases,
            kanji,
            level,
        }
    }

    pub fn set_similarity(&mut self, similarity: &[VocabularyCard]) {
        self.similarity = similarity
            .iter()
            .map(|card| self.card_to_study_item(card))
            .collect();
    }

    pub fn set_homonyms(&mut self, homonyms: &[VocabularyCard]) {
        self.homonyms = homonyms
            .iter()
            .map(|card| self.card_to_study_item(card))
            .collect();
    }

    fn card_to_study_item(&self, card: &VocabularyCard) -> StudySessionItem {
        StudySessionItem::new(
            card.id(),
            card.answer().text().to_string(),
            card.question().text().to_string(),
            false,
            card.example_phrases().to_vec(),
            card.get_kanji_cards(&self.level),
            self.level.clone(),
        )
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

    pub fn similarity(&self) -> &Vec<StudySessionItem> {
        &self.similarity
    }

    pub fn homonyms(&self) -> &Vec<StudySessionItem> {
        &self.homonyms
    }

    pub fn example_phrases(&self) -> &[ExamplePhrase] {
        &self.example_phrases
    }

    pub fn kanji(&self) -> &[KanjiCard] {
        &self.kanji
    }
}
