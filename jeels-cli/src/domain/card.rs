use crate::domain::Rating;
use crate::domain::review::Review;
use crate::domain::value_objects::{Answer, MemoryState, Question, Stability};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    id: Ulid,
    answer: Answer,
    question: Question,
    stability: Stability,
    reviews: VecDeque<Review>,
    next_review_date: DateTime<Utc>,
    memory_state: Option<MemoryState>,
}

impl Card {
    pub fn new(question: Question, answer: Answer) -> Self {
        Self {
            id: Ulid::new(),
            question,
            answer,
            reviews: VecDeque::new(),
            next_review_date: Utc::now(),
            stability: Stability::new(0.0).unwrap(),
            memory_state: None,
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

    pub fn reviews(&self) -> &VecDeque<Review> {
        &self.reviews
    }

    pub fn next_review_date(&self) -> DateTime<Utc> {
        self.next_review_date
    }

    pub fn stability(&self) -> Stability {
        self.stability
    }

    pub fn memory_state(&self) -> Option<MemoryState> {
        self.memory_state
    }

    pub(crate) fn edit(&mut self, new_question: Question, new_answer: Answer) {
        self.question = new_question;
        self.answer = new_answer;
    }

    pub(crate) fn add_review(&mut self, review: Review) {
        self.reviews.push_back(review);
    }

    pub(crate) fn update_schedule(
        &mut self,
        next_review_date: DateTime<Utc>,
        stability: Stability,
    ) {
        self.next_review_date = next_review_date;
        self.stability = stability;
    }

    pub(crate) fn update_memory_state(&mut self, memory_state: MemoryState) {
        self.memory_state = Some(memory_state);
    }

    pub fn is_due(&self) -> bool {
        self.next_review_date <= Utc::now()
    }

    pub fn is_new(&self) -> bool {
        self.reviews
            .iter()
            .any(|review| review.rating() != Rating::Again)
    }

    pub fn last_review_date(&self) -> Option<DateTime<Utc>> {
        self.reviews.back().map(|review| review.timestamp())
    }
}
