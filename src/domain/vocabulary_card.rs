use crate::domain::dictionary::{KANJI_DB, KanjiInfo};
use crate::domain::japanese::IsJapanese;
use crate::domain::review::Review;
use crate::domain::value_objects::{
    Answer, CardContent, Difficulty, ExamplePhrase, JapaneseLevel, MemoryState, Question, Stability,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use ulid::Ulid;

const LOW_STABILITY_THRESHOLD: f64 = 1.5;
const KNOWN_CARD_STABILITY_THRESHOLD: f64 = 4.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyCard {
    id: Ulid,

    answer: Answer,
    question: Question,

    reviews: VecDeque<Review>,

    next_review_date: DateTime<Utc>,
    memory_state: Option<MemoryState>,

    example_phrases: Vec<ExamplePhrase>,
}

impl VocabularyCard {
    pub fn new(question: Question, content: CardContent) -> Self {
        Self {
            id: Ulid::new(),
            question,
            reviews: VecDeque::new(),
            next_review_date: Utc::now(),
            memory_state: None,
            answer: content.answer().clone(),
            example_phrases: content.example_phrases().to_vec(),
        }
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn question(&self) -> &Question {
        &self.question
    }

    pub fn answer(&self) -> &Answer {
        &self.answer
    }

    pub fn example_phrases(&self) -> &[ExamplePhrase] {
        &self.example_phrases
    }

    pub fn reviews(&self) -> &VecDeque<Review> {
        &self.reviews
    }

    pub fn next_review_date(&self) -> DateTime<Utc> {
        self.next_review_date
    }

    pub fn memory_state(&self) -> Option<MemoryState> {
        self.memory_state
    }

    pub fn stability(&self) -> Option<Stability> {
        self.memory_state.map(|ms| ms.stability())
    }

    pub fn difficulty(&self) -> Option<Difficulty> {
        self.memory_state.map(|ms| ms.difficulty())
    }

    pub(crate) fn edit(
        &mut self,
        new_question: Question,
        new_answer: Answer,
        new_example_phrases: Vec<ExamplePhrase>,
    ) {
        self.question = new_question;
        self.answer = new_answer;
        self.example_phrases = new_example_phrases;
    }

    pub(crate) fn add_review(&mut self, review: Review) {
        self.reviews.push_back(review);
    }

    pub(crate) fn update_schedule(
        &mut self,
        next_review_date: DateTime<Utc>,
        memory_state: MemoryState,
    ) {
        self.next_review_date = next_review_date;
        self.memory_state = Some(memory_state);
    }

    /// Карта которая требует повторения
    pub fn is_due(&self) -> bool {
        !self.is_new() && self.next_review_date <= Utc::now()
    }

    /// Карта изучение которой еще не началось
    pub fn is_new(&self) -> bool {
        self.memory_state.is_none()
    }

    /// Карта которая имеет низкую стабильность
    pub fn is_low_stability(&self) -> bool {
        self.stability()
            .map(|stability| stability.value() < LOW_STABILITY_THRESHOLD)
            .unwrap_or(false)
    }

    /// Карта которая еще не была изучена до стабильного уровня, но уже начала изучаться
    pub fn is_in_progress(&self) -> bool {
        self.stability()
            .map(|stability| {
                stability.value() < KNOWN_CARD_STABILITY_THRESHOLD
                    && stability.value() >= LOW_STABILITY_THRESHOLD
            })
            .unwrap_or(false)
    }

    /// Карта которая уже изучена до стабильного уровня
    pub fn is_known_card(&self) -> bool {
        self.stability()
            .map(|stability| stability.value() > KNOWN_CARD_STABILITY_THRESHOLD)
            .unwrap_or(false)
    }

    pub fn last_review_date(&self) -> Option<DateTime<Utc>> {
        self.reviews.back().map(|review| review.timestamp())
    }

    pub fn get_kanji_cards(&self, current_level: &JapaneseLevel) -> Vec<&KanjiInfo> {
        self.question
            .text()
            .chars()
            .filter(|c| c.is_kanji())
            .filter_map(|c| KANJI_DB.get_kanji_info(&c).ok())
            .filter(|k| k.jlpt() <= current_level)
            .collect::<Vec<_>>()
    }
}
