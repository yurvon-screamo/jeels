pub mod daily_history;
pub mod error;
pub mod japanese;
pub mod kanji;
pub mod review;
pub mod study_session;
pub mod value_objects;
pub mod vocabulary_card;

pub use error::JeersError;
use rand::{Rng, seq::SliceRandom};
pub use review::Review;
pub use value_objects::Rating;
pub use vocabulary_card::VocabularyCard;

use crate::domain::{
    daily_history::DailyHistoryItem,
    japanese::IsJapaneseText,
    study_session::StudySessionItem,
    value_objects::{Answer, ExamplePhrase, JapaneseLevel, MemoryState, NativeLanguage, Question},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: Ulid,
    username: String,
    new_cards_limit: usize,
    native_language: NativeLanguage,
    current_japanese_level: JapaneseLevel,
    lesson_history: Vec<DailyHistoryItem>,
    duolingo_jwt_token: Option<String>,
    vocabulary_cards: HashMap<Ulid, VocabularyCard>,
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
            vocabulary_cards: HashMap::new(),
            current_japanese_level,
            native_language,
            new_cards_limit,
            lesson_history: Vec::new(),
            duolingo_jwt_token: None,
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

    pub fn cards(&self) -> &HashMap<Ulid, VocabularyCard> {
        &self.vocabulary_cards
    }

    pub fn new_cards_limit(&self) -> usize {
        self.new_cards_limit
    }

    pub fn duolingo_jwt_token(&self) -> Option<&str> {
        self.duolingo_jwt_token.as_deref()
    }

    pub fn set_duolingo_jwt_token(&mut self, token: Option<String>) {
        self.duolingo_jwt_token = token;
    }

    pub fn find_homonyms(&self, card_id: Ulid) -> Result<Vec<VocabularyCard>, JeersError> {
        let card = self
            .vocabulary_cards
            .get(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        let hiragana = card.question().text().as_hiragana();

        let homonyms = self
            .vocabulary_cards
            .values()
            .filter(|card| card.id() != card_id)
            .filter(|card| card.question().text().as_hiragana() == hiragana)
            .cloned()
            .collect::<Vec<_>>();

        Ok(homonyms)
    }

    pub fn find_similarity(&self, card_id: Ulid) -> Result<Vec<VocabularyCard>, JeersError> {
        const SIMILARITY_THRESHOLD: f32 = 0.8;

        let card = self
            .vocabulary_cards
            .get(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        let query_embedding = card.question().embedding();
        let similarity = self
            .vocabulary_cards
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

        self.vocabulary_cards.iter().any(|(id, card)| {
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

    pub fn create_card(
        &mut self,
        question: Question,
        answer: Answer,
        example_phrases: Vec<ExamplePhrase>,
    ) -> Result<VocabularyCard, JeersError> {
        if self.has_card_with_question(&question, None) {
            return Err(JeersError::DuplicateCard {
                question: question.text().to_string(),
            });
        }
        let card = VocabularyCard::new(question, answer, example_phrases);
        self.vocabulary_cards.insert(card.id(), card.clone());
        Ok(card)
    }

    pub fn edit_card(
        &mut self,
        card_id: Ulid,
        new_question: Question,
        new_answer: Answer,
        new_example_phrases: Vec<ExamplePhrase>,
    ) -> Result<(), JeersError> {
        let card = self
            .vocabulary_cards
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
            .vocabulary_cards
            .get_mut(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        card.edit(new_question, new_answer, new_example_phrases);

        Ok(())
    }

    pub fn delete_card(&mut self, card_id: Ulid) -> Result<VocabularyCard, JeersError> {
        let card = self
            .vocabulary_cards
            .remove(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        Ok(card)
    }

    pub fn start_low_stability_cards_session(&self) -> Vec<StudySessionItem> {
        self.vocabulary_cards
            .values()
            .filter(|card| card.is_low_stability())
            .filter_map(|card| self.card_to_study_item(card))
            .collect()
    }

    pub fn start_study_session(&self, force_new_cards: bool) -> Vec<StudySessionItem> {
        let mut old_cards: Vec<_> = self
            .vocabulary_cards
            .values()
            .filter(|card| card.is_due() && (card.is_in_progress() || card.is_known_card()))
            .collect();

        let mut low_stability_cards: Vec<_> = self
            .vocabulary_cards
            .values()
            .filter(|card| card.is_due() && card.is_low_stability())
            .collect();

        if force_new_cards || low_stability_cards.len() < self.new_cards_limit {
            let new_cards = self.vocabulary_cards.values().filter(|card| card.is_new());

            if !force_new_cards {
                let new_count = self.new_cards_limit - low_stability_cards.len();

                let mut truncated_new_cards: Vec<_> = new_cards.collect();
                truncated_new_cards.sort_by_key(|a| a.next_review_date());
                truncated_new_cards.truncate(new_count);

                low_stability_cards.extend(truncated_new_cards);
            } else {
                low_stability_cards.extend(new_cards);
            }
        }

        old_cards.sort_by_key(|a| a.next_review_date());
        low_stability_cards.sort_by(|a, b| {
            let reviews_cmp = b.reviews().len().cmp(&a.reviews().len());
            if reviews_cmp != std::cmp::Ordering::Equal {
                reviews_cmp
            } else {
                a.next_review_date().cmp(&b.next_review_date())
            }
        });

        old_cards.append(&mut low_stability_cards);

        let mut study_session_items: Vec<_> = old_cards
            .into_iter()
            .filter_map(|card| self.card_to_study_item(card))
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
            .vocabulary_cards
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
            .vocabulary_cards
            .get_mut(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        card.update_schedule(next_review_date, memory_state);

        Ok(())
    }

    pub fn update_daily_history(&mut self) {
        let stability_cards: Vec<_> = self
            .vocabulary_cards
            .values()
            .filter_map(|card| card.stability())
            .collect();
        let avg_stability = if stability_cards.is_empty() {
            None
        } else {
            Some(
                stability_cards
                    .iter()
                    .map(|stability| stability.value())
                    .sum::<f64>()
                    / stability_cards.len() as f64,
            )
        };

        let difficulty_cards: Vec<_> = self
            .vocabulary_cards
            .values()
            .filter_map(|card| card.difficulty())
            .collect();
        let avg_difficulty = if difficulty_cards.is_empty() {
            None
        } else {
            Some(
                difficulty_cards
                    .iter()
                    .map(|difficulty| difficulty.value())
                    .sum::<f64>()
                    / difficulty_cards.len() as f64,
            )
        };

        let total_words = self.vocabulary_cards.len();
        let known_words = self
            .vocabulary_cards
            .values()
            .filter(|card| card.is_known_card())
            .count();
        let new_words = self
            .vocabulary_cards
            .values()
            .filter(|card| card.is_new())
            .count();

        let now = Utc::now();
        let today = now.date_naive();

        if let Some(existing_item) = self
            .lesson_history
            .iter_mut()
            .find(|item| item.timestamp().date_naive() == today)
        {
            existing_item.update(
                avg_stability,
                avg_difficulty,
                total_words,
                known_words,
                new_words,
            );
        } else {
            let daily_history_item = DailyHistoryItem::new(
                now,
                avg_stability,
                avg_difficulty,
                total_words,
                known_words,
                new_words,
            );

            self.lesson_history.push(daily_history_item);
        }
    }

    pub fn get_card(&self, card_id: Ulid) -> Option<&VocabularyCard> {
        self.vocabulary_cards.get(&card_id)
    }

    pub fn lesson_history(&self) -> &[DailyHistoryItem] {
        &self.lesson_history
    }

    pub fn find_similar_cards(
        &self,
        card_id: Ulid,
        limit: usize,
    ) -> Result<Vec<(VocabularyCard, f32)>, JeersError> {
        let query_card = self
            .vocabulary_cards
            .get(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        let query_embedding = query_card.question().embedding();

        let mut results: Vec<(VocabularyCard, f32)> = self
            .vocabulary_cards
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

    fn card_to_study_item(&self, card: &VocabularyCard) -> Option<StudySessionItem> {
        let mut shuffle = rand::rng().random_bool(0.65);
        let (answer, question) = if card.is_known_card() && shuffle {
            (card.question().text(), card.answer().text())
        } else {
            shuffle = false;
            (card.answer().text(), card.question().text())
        };

        let mut item = StudySessionItem::new(
            card.id(),
            answer.to_string(),
            question.to_string(),
            shuffle,
            card.example_phrases().to_vec(),
            card.get_kanji_cards(&self.current_japanese_level),
            self.current_japanese_level.clone(),
        );

        let similarity = self.find_similarity(card.id());
        let homonyms = self.find_homonyms(card.id());

        if let Ok(similarity) = similarity {
            item.set_similarity(&similarity);
        }

        if let Ok(homonyms) = homonyms {
            item.set_homonyms(&homonyms);
        }

        Some(item)
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
