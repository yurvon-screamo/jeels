mod daily_history;
mod grammar;
mod kanji;
mod vocabulary;

pub use daily_history::DailyHistoryItem;
pub use grammar::GrammarRuleCard;
pub use kanji::{ExampleKanjiWord, KanjiCard};
pub use vocabulary::VocabularyCard;

use std::collections::HashMap;

use crate::domain::{
    KeikakuError, Rating, Review,
    review::{MemoryHistory, MemoryState},
    value_objects::{Answer, Question},
};
use chrono::{Duration, Utc};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

const NEW_CARDS_LIMIT: usize = 7;
const HARD_CARDS_LIMIT: usize = 15;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeSet {
    study_cards: HashMap<Ulid, StudyCard>,
    lesson_history: Vec<DailyHistoryItem>,
}

impl Default for KnowledgeSet {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeSet {
    pub fn new() -> Self {
        Self {
            study_cards: HashMap::new(),
            lesson_history: Vec::new(),
        }
    }

    pub fn get_card(&self, card_id: Ulid) -> Option<&StudyCard> {
        self.study_cards.get(&card_id)
    }

    pub fn study_cards(&self) -> &HashMap<Ulid, StudyCard> {
        &self.study_cards
    }

    pub fn lesson_history(&self) -> &[DailyHistoryItem] {
        &self.lesson_history
    }

    pub fn delete_card(&mut self, card_id: Ulid) -> Result<(), KeikakuError> {
        if self.study_cards.remove(&card_id).is_none() {
            return Err(KeikakuError::CardNotFound { card_id });
        }
        Ok(())
    }

    pub fn create_card(&mut self, card: Card) -> Result<StudyCard, KeikakuError> {
        let study_card = StudyCard::new(card);
        let card_id = *study_card.card_id();

        self.validate_unique_card(study_card.card())?;

        if self
            .study_cards
            .insert(card_id, study_card.clone())
            .is_some()
        {
            return Err(KeikakuError::DuplicateCard {
                question: study_card.card().question().text().to_string(),
            });
        }
        Ok(study_card)
    }

    fn validate_unique_card(&self, card: &Card) -> Result<(), KeikakuError> {
        if self
            .study_cards
            .values()
            .any(|c| c.card().question() == card.question())
        {
            return Err(KeikakuError::DuplicateCard {
                question: card.question().text().to_string(),
            });
        }

        Ok(())
    }

    pub fn cards_to_fixation(&self) -> Vec<Card> {
        let mut cards = self
            .study_cards
            .iter()
            .filter(|(_, card)| {
                card.memory().is_low_stability() || card.memory().is_high_difficulty()
            })
            .collect::<Vec<_>>();

        cards.sort_by_key(|(_, card)| card.memory().next_review_date());
        cards.reverse();

        cards.truncate(HARD_CARDS_LIMIT);

        cards.iter().map(|(_, card)| card.card().clone()).collect()
    }

    pub fn cards_to_lesson(&self) -> HashMap<Ulid, Card> {
        let mut all_cards = self.study_cards.iter().collect::<Vec<_>>();
        all_cards.sort_by_key(|(_, card)| card.memory().next_review_date());

        let mut due_cards: Vec<_> = all_cards
            .iter()
            .filter(|(_, card)| {
                card.memory().is_due()
                    && (card.memory().is_in_progress() || card.memory().is_known_card())
            })
            .collect();

        let mut priority_cards: Vec<_> = all_cards
            .iter()
            .filter(|(_, card)| card.memory().is_due() && card.memory().is_low_stability())
            .collect();

        if priority_cards.len() < NEW_CARDS_LIMIT {
            let mut new_cards: Vec<_> = all_cards
                .iter()
                .filter(|(_, card)| card.memory().is_new())
                .collect();

            let available = NEW_CARDS_LIMIT.saturating_sub(priority_cards.len());
            new_cards.truncate(available);

            priority_cards.extend(new_cards);
        }

        due_cards.sort_by_key(|(_, card)| card.memory().next_review_date());
        priority_cards.sort_by(|(_, a_card), (_, b_card)| {
            let reviews_cmp = b_card
                .memory()
                .reviews()
                .len()
                .cmp(&a_card.memory().reviews().len());
            if reviews_cmp != std::cmp::Ordering::Equal {
                return reviews_cmp;
            }

            a_card
                .memory()
                .next_review_date()
                .cmp(&b_card.memory().next_review_date())
        });

        due_cards.append(&mut priority_cards);
        due_cards.shuffle(&mut rand::rng());

        due_cards
            .iter()
            .map(|(card_id, card)| (**card_id, card.card().clone()))
            .collect()
    }

    pub(crate) fn rate_card(
        &mut self,
        card_id: Ulid,
        rating: Rating,
        interval: Duration,
        memory_state: MemoryState,
    ) -> Result<(), KeikakuError> {
        if let Some(card) = self.study_cards.get_mut(&card_id) {
            let review = Review::new(rating, interval);
            card.memory_history.add_review(memory_state, review);
            self.update_history();
            Ok(())
        } else {
            Err(KeikakuError::CardNotFound { card_id })
        }
    }

    pub(crate) fn add_lesson_duration(&mut self, lesson_duration: Duration) {
        self.lesson_history
            .last_mut()
            .unwrap()
            .add_lesson_duration(lesson_duration);
    }

    fn update_history(&mut self) {
        let stability_cards: Vec<_> = self
            .study_cards
            .values()
            .filter_map(|card| card.memory().stability())
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
            .study_cards
            .values()
            .filter_map(|card| card.memory().difficulty())
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

        let total_words = self.study_cards.len();
        let known_words = self
            .study_cards
            .values()
            .filter(|card| card.memory().is_known_card())
            .count();
        let new_words = self
            .study_cards
            .values()
            .filter(|card| card.memory().is_new())
            .count();
        let in_progress_words = self
            .study_cards
            .values()
            .filter(|card| card.memory().is_in_progress())
            .count();
        let low_stability_words = self
            .study_cards
            .values()
            .filter(|card| card.memory().is_low_stability())
            .count();
        let high_difficulty_words = self
            .study_cards
            .values()
            .filter(|card| card.memory().is_high_difficulty())
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
                in_progress_words,
                low_stability_words,
                high_difficulty_words,
            );
        } else {
            let mut item = DailyHistoryItem::new();
            item.update(
                avg_stability,
                avg_difficulty,
                total_words,
                known_words,
                new_words,
                in_progress_words,
                low_stability_words,
                high_difficulty_words,
            );
            self.lesson_history.push(item);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudyCard {
    card_id: Ulid,
    card: Card,
    memory_history: MemoryHistory,
}

impl StudyCard {
    pub fn new(card: Card) -> Self {
        Self {
            card_id: Ulid::new(),
            card,
            memory_history: MemoryHistory::default(),
        }
    }

    pub fn card_id(&self) -> &Ulid {
        &self.card_id
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

    pub fn memory(&self) -> &MemoryHistory {
        &self.memory_history
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Card {
    Vocabulary(VocabularyCard),
    Kanji(KanjiCard),
    Grammar(GrammarRuleCard),
}

impl Card {
    fn question(&self) -> &Question {
        match self {
            Card::Vocabulary(card) => card.word(),
            Card::Kanji(card) => card.kanji(),
            Card::Grammar(card) => card.title(),
        }
    }

    fn answer(&self) -> &Answer {
        match self {
            Card::Vocabulary(card) => card.meaning(),
            Card::Kanji(card) => card.description(),
            Card::Grammar(card) => card.description(),
        }
    }
}
