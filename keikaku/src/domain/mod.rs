pub mod daily_history;
pub mod dictionary;
pub mod error;
pub mod japanese;
pub mod kanji_card;
pub mod review;
pub mod settings;
pub mod study_session;
pub mod value_objects;
pub mod vocabulary_card;

use crate::domain::{
    daily_history::DailyHistoryItem,
    japanese::IsJapaneseText,
    kanji_card::KanjiCard,
    review::MemoryState,
    study_session::{KanjiStudySessionItem, StudySessionItem, VocabularyStudySessionItem},
    value_objects::{Answer, CardContent, ExamplePhrase, JapaneseLevel, NativeLanguage, Question},
};
use chrono::{DateTime, Duration, Utc};
pub use error::JeersError;
use rand::{Rng, seq::SliceRandom};
pub use review::Review;
use serde::{Deserialize, Serialize};
pub use settings::{EmbeddingSettings, LlmSettings, TranslationSettings, UserSettings};
use std::collections::HashMap;
use ulid::Ulid;
pub use value_objects::Rating;
pub use vocabulary_card::VocabularyCard;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: Ulid,
    username: String,
    new_cards_limit: usize,
    native_language: NativeLanguage,
    current_japanese_level: JapaneseLevel,
    lesson_history: Vec<DailyHistoryItem>,

    #[serde(default)]
    settings: UserSettings,

    vocabulary_cards: HashMap<Ulid, VocabularyCard>,
    kanji_cards: HashMap<Ulid, KanjiCard>,
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
            kanji_cards: HashMap::new(),
            current_japanese_level,
            native_language,
            new_cards_limit,
            lesson_history: Vec::new(),
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

    pub fn cards(&self) -> &HashMap<Ulid, VocabularyCard> {
        &self.vocabulary_cards
    }

    pub fn new_cards_limit(&self) -> usize {
        self.new_cards_limit
    }

    pub fn settings(&self) -> &UserSettings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut UserSettings {
        &mut self.settings
    }

    pub fn find_homonyms(&self, card_id: Ulid) -> Result<Vec<VocabularyCard>, JeersError> {
        let original_card = self
            .vocabulary_cards
            .get(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        let homonyms = self
            .vocabulary_cards
            .values()
            .filter(|card| card.id() != card_id)
            .filter(|card| {
                card.word()
                    .text()
                    .equals_by_reading(original_card.word().text())
            })
            .cloned()
            .collect::<Vec<_>>();

        Ok(homonyms)
    }

    pub fn find_similarity(&self, card_id: Ulid) -> Result<Vec<VocabularyCard>, JeersError> {
        let card = self
            .vocabulary_cards
            .get(&card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        let similarity = self
            .vocabulary_cards
            .iter()
            .filter(|(id, c)| {
                if *id == &card_id {
                    return false;
                }
                c.word().text() == card.word().text()
            })
            .map(|(_, card)| card.clone())
            .collect::<Vec<_>>();

        Ok(similarity)
    }

    pub fn create_card(
        &mut self,
        question: Question,
        content: CardContent,
    ) -> Result<VocabularyCard, JeersError> {
        if self
            .vocabulary_cards
            .values()
            .any(|card| card.word().text() == question.text())
        {
            return Err(JeersError::DuplicateCard {
                question: question.text().to_string(),
            });
        }
        let card = VocabularyCard::new(question, content);
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

        let current_question = card.word();
        let question_changed = current_question.text().trim().to_lowercase()
            != new_question.text().trim().to_lowercase();

        if question_changed
            && self
                .vocabulary_cards
                .values()
                .any(|card| card.word().text() == new_question.text())
        {
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

    pub fn start_low_stability_cards_session(&self, limit: Option<usize>) -> Vec<StudySessionItem> {
        self.collect_study_cards(limit)
            .into_iter()
            .filter(|card| card.is_low_stability)
            .map(|card| card.item)
            .collect()
    }

    pub fn start_study_session(
        &self,
        force_new_cards: bool,
        limit: Option<usize>,
    ) -> Vec<StudySessionItem> {
        let all_cards = self.collect_study_cards(limit);
        let mut due_cards: Vec<_> = all_cards
            .iter()
            .filter(|card| card.is_due && (card.is_in_progress || card.is_known))
            .cloned()
            .collect();
        let mut priority_cards: Vec<_> = all_cards
            .iter()
            .filter(|card| card.is_due && card.is_low_stability)
            .cloned()
            .collect();

        if force_new_cards || priority_cards.len() < self.new_cards_limit {
            let mut new_cards: Vec<_> = all_cards.into_iter().filter(|card| card.is_new).collect();
            new_cards.sort_by_key(|card| card.next_review_date);

            if !force_new_cards {
                let available = self.new_cards_limit.saturating_sub(priority_cards.len());
                new_cards.truncate(available);
            }

            priority_cards.extend(new_cards);
        }

        due_cards.sort_by_key(|card| card.next_review_date);
        priority_cards.sort_by(|a, b| {
            let reviews_cmp = b.reviews_len.cmp(&a.reviews_len);
            if reviews_cmp != std::cmp::Ordering::Equal {
                return reviews_cmp;
            }
            a.next_review_date.cmp(&b.next_review_date)
        });

        due_cards.append(&mut priority_cards);

        let mut study_session_items: Vec<_> = due_cards.into_iter().map(|card| card.item).collect();
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
        if let Some(card) = self.vocabulary_cards.get_mut(&card_id) {
            let review = Review::new(rating, interval);
            card.add_review(memory_state, review);
        } else if let Some(card) = self.kanji_cards.get_mut(&card_id) {
            let review = Review::new(rating, interval);
            card.add_review(memory_state, review);
        } else {
            return Err(JeersError::CardNotFound { card_id });
        }

        Ok(())
    }

    pub fn update_daily_history(&mut self, lesson_duration: Duration) {
        let stability_cards: Vec<_> = self
            .vocabulary_cards
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
            .vocabulary_cards
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

        let total_words = self.vocabulary_cards.len();
        let known_words = self
            .vocabulary_cards
            .values()
            .filter(|card| card.memory().is_known_card())
            .count();
        let new_words = self
            .vocabulary_cards
            .values()
            .filter(|card| card.memory().is_new())
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
                lesson_duration,
            );
        } else {
            let daily_history_item = DailyHistoryItem::new(
                now,
                avg_stability,
                avg_difficulty,
                total_words,
                known_words,
                new_words,
                lesson_duration,
            );

            self.lesson_history.push(daily_history_item);
        }
    }

    pub fn get_card(&self, card_id: Ulid) -> Option<&VocabularyCard> {
        self.vocabulary_cards.get(&card_id)
    }

    pub fn get_kanji_card(&self, card_id: Ulid) -> Option<&KanjiCard> {
        self.kanji_cards.get(&card_id)
    }

    pub fn lesson_history(&self) -> &[DailyHistoryItem] {
        &self.lesson_history
    }

    fn card_to_study_item(&self, card: &VocabularyCard) -> Result<StudySessionItem, JeersError> {
        let shuffle = rand::rng().random_bool(0.65);

        // TODO: Skip expensive similarity/homonyms calculation for performance
        // TODO: These fields are not currently used in the UI
        let similarity = vec![];
        let homonyms = vec![];

        Ok(StudySessionItem::Vocabulary(
            VocabularyStudySessionItem::new(
                card.id(),
                card.word().text().to_string(),
                card.meaning().text().to_string(),
                shuffle,
                similarity,
                homonyms,
                card.example_phrases().to_vec(),
                card.get_kanji_cards(&self.current_japanese_level)
                    .into_iter()
                    .cloned()
                    .collect(),
                self.current_japanese_level,
            ),
        ))
    }

    fn kanji_card_to_study_item(&self, card: &KanjiCard) -> Result<StudySessionItem, JeersError> {
        let radicals = card
            .radicals_info()?
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();

        Ok(StudySessionItem::Kanji(KanjiStudySessionItem::new(
            card.id(),
            card.kanji().text().chars().next().unwrap_or_default(),
            card.description().text().to_string(),
            card.example_words().to_vec(),
            radicals,
            card.jlpt(),
        )))
    }

    fn collect_study_cards(&self, limit: Option<usize>) -> Vec<StudyCard> {
        let vocabulary_cards = self
            .vocabulary_cards
            .values()
            .filter_map(|card| self.vocabulary_to_study_card(card).ok());
        let kanji_cards = self
            .kanji_cards
            .values()
            .filter_map(|card| self.kanji_to_study_card(card).ok());

        let mut cards: Vec<_> = vocabulary_cards.chain(kanji_cards).collect();
        cards.sort_by_key(|card| card.next_review_date);

        if let Some(limit) = limit {
            cards.truncate(limit);
        }

        cards
    }

    fn vocabulary_to_study_card(&self, card: &VocabularyCard) -> Result<StudyCard, JeersError> {
        let item = self.card_to_study_item(card)?;
        let memory = card.memory();
        Ok(StudyCard {
            item,
            next_review_date: memory.next_review_date().cloned(),
            reviews_len: memory.reviews().len(),
            is_due: memory.is_due(),
            is_low_stability: memory.is_low_stability(),
            is_in_progress: memory.is_in_progress(),
            is_known: memory.is_known_card(),
            is_new: memory.is_new(),
        })
    }

    fn kanji_to_study_card(&self, card: &KanjiCard) -> Result<StudyCard, JeersError> {
        let item = self.kanji_card_to_study_item(card)?;
        let memory = card.memory_history();
        Ok(StudyCard {
            item,
            next_review_date: memory.next_review_date().cloned(),
            reviews_len: memory.reviews().len(),
            is_due: memory.is_due(),
            is_low_stability: memory.is_low_stability(),
            is_in_progress: memory.is_in_progress(),
            is_known: memory.is_known_card(),
            is_new: memory.is_new(),
        })
    }
}

#[derive(Clone)]
struct StudyCard {
    item: StudySessionItem,
    next_review_date: Option<DateTime<Utc>>,
    reviews_len: usize,
    is_due: bool,
    is_low_stability: bool,
    is_in_progress: bool,
    is_known: bool,
    is_new: bool,
}
