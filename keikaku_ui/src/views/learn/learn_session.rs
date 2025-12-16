use keikaku::domain::{
    dictionary::{KanjiInfo, RadicalInfo},
    kanji_card::ExampleKanjiWord,
    value_objects::{ExamplePhrase, JapaneseLevel},
};

#[derive(Clone, PartialEq)]
pub enum SessionState {
    Loading,
    Active,
    Completed,
}

#[derive(Clone, PartialEq)]
pub enum LearnStep {
    Question,
    Answer,
    Completed, // Новое состояние после рейтинга
}

#[derive(Clone, PartialEq)]
pub enum CardType {
    Vocabulary,
    Kanji,
}

#[derive(Clone, PartialEq)]
pub struct SimilarCard {
    pub word: String,
    pub meaning: String,
}

#[derive(Clone, PartialEq)]
pub struct LearnCard {
    pub id: String,
    pub card_type: CardType,
    pub question: String,
    pub answer: String,
    // Для vocabulary:
    pub example_phrases: Vec<ExamplePhrase>,
    pub similarity: Vec<SimilarCard>,
    pub homonyms: Vec<SimilarCard>,
    pub kanji_info: Vec<KanjiInfo>,
    // Для kanji:
    pub example_words: Vec<ExampleKanjiWord>,
    pub radicals: Vec<RadicalInfo>,
    pub jlpt_level: JapaneseLevel,
}
