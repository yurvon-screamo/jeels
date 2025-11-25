pub mod card;
pub mod error;
pub mod review;
pub mod value_objects;

pub use card::Card;
pub use error::JeersError;
pub use review::Review;
pub use value_objects::Rating;

use crate::domain::value_objects::{Answer, Interval, MemoryState, Question, Stability};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: Ulid,
    username: String,
    new_cards_limit: usize,
    cards: HashMap<Ulid, Card>,
    native_language: NativeLanguage,
    current_japanese_level: JapaneseLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeLanguage {
    English,
    Russian,
}

impl User {
    pub fn new(
        username: String,
        current_japanese_level: JapaneseLevel,
        native_language: NativeLanguage,
        new_cards_limit: usize,
    ) -> Self {
        Self {
            id: Ulid::new(),
            username,
            cards: HashMap::new(),
            current_japanese_level,
            native_language,
            new_cards_limit,
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

    pub fn cards(&self) -> &HashMap<Ulid, Card> {
        &self.cards
    }

    pub fn new_cards_limit(&self) -> usize {
        self.new_cards_limit
    }

    pub fn find_synonyms(&self, card_id: Ulid) -> Result<Vec<Card>, JeersError> {
        const SIMILARITY_THRESHOLD: f32 = 0.85;

        let card = self
            .cards
            .get(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        let query_embedding = card.question().embedding();
        let synonyms = self
            .cards
            .iter()
            .filter(|(id, card)| {
                if *id == &card_id {
                    return false;
                }
                let card_embedding = card.question().embedding();
                let similarity = cosine_similarity(query_embedding, card_embedding);
                similarity >= SIMILARITY_THRESHOLD
            })
            .map(|(_, card)| card.clone())
            .collect();

        Ok(synonyms)
    }

    fn has_card_with_question(&self, question: &Question, exclude_card_id: Option<Ulid>) -> bool {
        let query_embedding = question.embedding();
        const SIMILARITY_THRESHOLD: f32 = 0.9999;

        self.cards.iter().any(|(id, card)| {
            if let Some(exclude_id) = exclude_card_id
                && *id == exclude_id
            {
                return false;
            }

            let card_embedding = card.question().embedding();
            let similarity = cosine_similarity(query_embedding, card_embedding);

            similarity >= SIMILARITY_THRESHOLD
        })
    }

    pub fn create_card(&mut self, question: Question, answer: Answer) -> Result<Card, JeersError> {
        if self.has_card_with_question(&question, None) {
            return Err(JeersError::DuplicateCard {
                question: question.text().to_string(),
            });
        }
        let card = Card::new(question, answer);
        self.cards.insert(card.id(), card.clone());
        Ok(card)
    }

    pub fn edit_card(
        &mut self,
        card_id: Ulid,
        new_question: Question,
        new_answer: Answer,
    ) -> Result<(), JeersError> {
        let card = self
            .cards
            .get(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        let current_question = card.question();
        let question_changed = current_question.text().trim().to_lowercase()
            != new_question.text().trim().to_lowercase();

        if question_changed && self.has_card_with_question(&new_question, Some(card_id)) {
            return Err(JeersError::DuplicateCard {
                question: new_question.text().to_string(),
            });
        }

        let card = self
            .cards
            .get_mut(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        card.edit(new_question, new_answer);

        Ok(())
    }

    pub fn delete_card(&mut self, card_id: Ulid) -> Result<Card, JeersError> {
        let card = self
            .cards
            .remove(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        Ok(card)
    }

    pub fn start_study_session(&self) -> Vec<Card> {
        let mut old_cards: Vec<Card> = self
            .cards
            .values()
            .filter(|card| card.is_due() && !card.is_new())
            .cloned()
            .collect();

        let mut new_cards: Vec<Card> = self
            .cards
            .values()
            .filter(|card| card.is_due() && card.is_new())
            .cloned()
            .collect();

        old_cards.sort_by_key(|a| a.next_review_date());
        new_cards.sort_by_key(|a| a.next_review_date());

        new_cards.truncate(self.new_cards_limit);
        old_cards.append(&mut new_cards);

        old_cards
    }

    pub fn rate_card(
        &mut self,
        card_id: Ulid,
        rating: Rating,
        interval: Interval,
        next_review_date: DateTime<Utc>,
        stability: Stability,
        memory_state: MemoryState,
    ) -> Result<(), JeersError> {
        let card = self
            .cards
            .get_mut(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        let review = Review::new(rating, interval);
        card.add_review(review);

        self.schedule_next_review(card_id, next_review_date, stability, memory_state)?;
        Ok(())
    }

    fn schedule_next_review(
        &mut self,
        card_id: Ulid,
        next_review_date: DateTime<Utc>,
        stability: Stability,
        memory_state: MemoryState,
    ) -> Result<(), JeersError> {
        let card = self
            .cards
            .get_mut(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        card.update_schedule(next_review_date, stability);
        card.update_memory_state(memory_state);

        Ok(())
    }

    pub fn get_card(&self, card_id: Ulid) -> Option<&Card> {
        self.cards.get(&card_id)
    }

    pub fn find_similar_cards(
        &self,
        card_id: Ulid,
        limit: usize,
    ) -> Result<Vec<(Card, f32)>, JeersError> {
        let query_card = self
            .cards
            .get(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        let query_embedding = query_card.question().embedding();

        let mut results: Vec<(Card, f32)> = self
            .cards
            .values()
            .filter(|card| card.id() != card_id)
            .map(|card| {
                let card_embedding = card.question().embedding();
                let similarity = cosine_similarity(query_embedding, card_embedding);
                (card.clone(), similarity)
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        Ok(results)
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod mod_test;
