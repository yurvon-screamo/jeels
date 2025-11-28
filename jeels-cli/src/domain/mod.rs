pub mod card;
pub mod error;
pub mod review;
pub mod value_objects;

pub use card::Card;
pub use error::JeersError;
use rand::{Rng, seq::SliceRandom};
pub use review::Review;
pub use value_objects::Rating;

use crate::domain::value_objects::{Answer, MemoryState, Question, StudySessionItem};
use chrono::{DateTime, Duration, Utc};
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

    #[serde(default)]
    lesson_history: Vec<LessonHistoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonHistoryItem {
    timestamp: DateTime<Utc>,
    avg_stability: f64,
    avg_difficulty: f64,
}

impl LessonHistoryItem {
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn avg_stability(&self) -> f64 {
        self.avg_stability
    }

    pub fn avg_difficulty(&self) -> f64 {
        self.avg_difficulty
    }
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
            lesson_history: Vec::new(),
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

    pub fn find_similarity(&self, card_id: Ulid) -> Result<Vec<Card>, JeersError> {
        const SIMILARITY_THRESHOLD: f32 = 0.8;

        let card = self
            .cards
            .get(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        let query_embedding = card.question().embedding();
        let similarity = self
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

        Ok(similarity)
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

    pub fn start_study_session(&self, force_new_cards: bool) -> Vec<StudySessionItem> {
        let mut old_cards: Vec<_> = self
            .cards
            .values()
            .filter(|card| card.is_due() && !card.is_new())
            .collect();

        let mut new_cards: Vec<_> = self
            .cards
            .values()
            .filter(|card| card.is_due() && card.is_new())
            .collect();

        old_cards.sort_by_key(|a| a.next_review_date());
        new_cards.sort_by(|a, b| {
            let reviews_cmp = b.reviews().len().cmp(&a.reviews().len());
            if reviews_cmp != std::cmp::Ordering::Equal {
                reviews_cmp
            } else {
                a.next_review_date().cmp(&b.next_review_date())
            }
        });

        if !force_new_cards {
            new_cards.truncate(self.new_cards_limit);
        }

        old_cards.append(&mut new_cards);

        let mut study_session_items: Vec<_> = old_cards
            .into_iter()
            .filter_map(|card| {
                let shuffle = rand::rng().random_bool(0.5);
                let (answer, question) = if card.is_known_card() && shuffle {
                    (
                        clear_japanese_characters(card.question().text()),
                        card.answer().text().to_string(),
                    )
                } else {
                    (
                        card.answer().text().to_string(),
                        card.question().text().to_string(),
                    )
                };

                let mut item =
                    StudySessionItem::new(card.id(), answer.to_string(), question.to_string());

                let similarity = self.find_similarity(card.id());

                if let Ok(similarity) = similarity {
                    item.set_similarity(&similarity);
                    Some(item)
                } else {
                    None
                }
            })
            .collect();

        study_session_items.shuffle(&mut rand::rng());

        study_session_items
    }

    pub fn rate_card(
        &mut self,
        card_id: Ulid,
        rating: Rating,
        interval: Duration,
        memory_state: MemoryState,
    ) -> Result<(), JeersError> {
        let card = self
            .cards
            .get_mut(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        let review = Review::new(rating, interval);
        card.add_review(review);

        let next_review_date = Utc::now() + interval;
        self.schedule_next_review(card_id, next_review_date, memory_state)?;

        Ok(())
    }

    fn schedule_next_review(
        &mut self,
        card_id: Ulid,
        next_review_date: DateTime<Utc>,
        memory_state: MemoryState,
    ) -> Result<(), JeersError> {
        let card = self
            .cards
            .get_mut(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        card.update_schedule(next_review_date, memory_state);

        Ok(())
    }

    pub fn create_lesson_history_item(&mut self) {
        let avg_stability = self
            .cards
            .values()
            .filter_map(|card| card.stability())
            .map(|stability| stability.value())
            .sum::<f64>()
            / self
                .cards
                .values()
                .filter_map(|card| card.stability())
                .count() as f64;

        let avg_difficulty = self
            .cards
            .values()
            .filter_map(|card| card.difficulty())
            .map(|difficulty| difficulty.value())
            .sum::<f64>()
            / self
                .cards
                .values()
                .filter_map(|card| card.stability())
                .count() as f64;

        let lesson_history_item = LessonHistoryItem {
            timestamp: Utc::now(),
            avg_stability,
            avg_difficulty,
        };

        self.lesson_history.push(lesson_history_item);
    }

    pub fn get_card(&self, card_id: Ulid) -> Option<&Card> {
        self.cards.get(&card_id)
    }

    pub fn lesson_history(&self) -> &[LessonHistoryItem] {
        &self.lesson_history
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

fn clear_japanese_characters(text: &str) -> String {
    text.chars()
        .filter(|c| !is_japanese_character(*c))
        .collect::<String>()
        .replace("''", "")
        .replace("\"\"", "")
        .replace("““", "")
        .replace("””", "")
        .replace("«»", "")
}

fn is_japanese_character(c: char) -> bool {
    let u = c as u32;
    matches!(u,
        // Hiragana
        0x3040..=0x309F |
        // Katakana
        0x30A0..=0x30FF |
        // Kanji (CJK Unified Ideographs)
        0x4E00..=0x9FAF |
        // Kanji Extension A
        0x3400..=0x4DBF |
        // CJK Symbols and Punctuation
        0x3000..=0x303F |
        // Half-width and Full-width Forms
        0xFF00..=0xFFEF |
        // Kanji Extension B
        0x20000..=0x2A6DF
    )
}

#[cfg(test)]
mod mod_test;
