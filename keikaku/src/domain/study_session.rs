use crate::domain::{
    VocabularyCard,
    dictionary::{KanjiInfo, RadicalInfo},
    kanji_card::ExampleKanjiWord,
    value_objects::{ExamplePhrase, JapaneseLevel},
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StudySessionItem {
    Vocabulary(VocabularyStudySessionItem),
    Kanji(KanjiStudySessionItem),
}

impl StudySessionItem {
    pub fn card_id(&self) -> Ulid {
        match self {
            StudySessionItem::Vocabulary(card) => card.card_id(),
            StudySessionItem::Kanji(card) => card.card_id(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VocabularyStudySessionItem {
    card_id: Ulid,
    word: String,
    meaning: String,
    shuffle: bool,
    similarity: Vec<VocabularyStudySessionItem>,
    homonyms: Vec<VocabularyStudySessionItem>,
    example_phrases: Vec<ExamplePhrase>,
    kanji: Vec<KanjiInfo>,
    level: JapaneseLevel,
}

impl VocabularyStudySessionItem {
    pub fn new(
        card_id: Ulid,
        word: String,
        meaning: String,
        shuffle: bool,
        similarity: Vec<VocabularyCard>,
        homonyms: Vec<VocabularyCard>,
        example_phrases: Vec<ExamplePhrase>,
        kanji: Vec<KanjiInfo>,
        level: JapaneseLevel,
    ) -> Self {
        Self {
            card_id,
            word,
            meaning,
            shuffle,
            similarity: similarity
                .into_iter()
                .map(|card| Self::card_to_study_item(&card, &level))
                .collect(),
            homonyms: homonyms
                .into_iter()
                .map(|card| Self::card_to_study_item(&card, &level))
                .collect(),
            example_phrases,
            kanji,
            level,
        }
    }

    fn card_to_study_item(
        card: &VocabularyCard,
        level: &JapaneseLevel,
    ) -> VocabularyStudySessionItem {
        VocabularyStudySessionItem::new(
            card.id(),
            card.word().text().to_string(),
            card.meaning().text().to_string(),
            false,
            vec![],
            vec![],
            card.example_phrases().to_vec(),
            card.get_kanji_cards(level).into_iter().cloned().collect(),
            *level,
        )
    }

    pub fn card_id(&self) -> Ulid {
        self.card_id
    }

    pub fn meaning(&self) -> &str {
        &self.meaning
    }

    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn similarity(&self) -> &[VocabularyStudySessionItem] {
        &self.similarity
    }

    pub fn homonyms(&self) -> &[VocabularyStudySessionItem] {
        &self.homonyms
    }

    pub fn example_phrases(&self) -> &[ExamplePhrase] {
        &self.example_phrases
    }

    pub fn kanji(&self) -> &[KanjiInfo] {
        &self.kanji
    }

    pub fn level(&self) -> &JapaneseLevel {
        &self.level
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KanjiStudySessionItem {
    card_id: Ulid,
    kanji: char,
    description: String,
    example_words: Vec<ExampleKanjiWord>,
    radicals: Vec<RadicalInfo>,
    level: JapaneseLevel,
}

impl KanjiStudySessionItem {
    pub fn new(
        card_id: Ulid,
        kanji: char,
        description: String,
        example_words: Vec<ExampleKanjiWord>,
        radicals: Vec<RadicalInfo>,
        level: JapaneseLevel,
    ) -> Self {
        Self {
            card_id,
            kanji,
            description,
            example_words,
            radicals,
            level,
        }
    }

    pub fn card_id(&self) -> Ulid {
        self.card_id
    }

    pub fn kanji(&self) -> char {
        self.kanji
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn example_words(&self) -> &[ExampleKanjiWord] {
        &self.example_words
    }

    pub fn radicals(&self) -> &[RadicalInfo] {
        &self.radicals
    }

    pub fn level(&self) -> &JapaneseLevel {
        &self.level
    }
}
