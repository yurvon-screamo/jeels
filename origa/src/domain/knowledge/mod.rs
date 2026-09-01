mod card;
mod daily_history;
mod empty_diagnosis;
mod grammar;
mod kanji;
mod kanji_companions;
pub mod lesson;
mod lesson_builder;
mod phrase;
mod stats_tracker;
mod stats_updater;
#[cfg(test)]
mod tests;
pub mod vocabulary;

pub use card::{Card, CardType, StudyCard};
pub use daily_history::{DailyHistoryItem, estimate_completion_date};
pub use empty_diagnosis::{LessonEmptyDiagnosis, diagnose_empty_lesson};
pub use grammar::GrammarRuleCard;
pub use kanji::{ExampleKanjiWord, KanjiCard};
pub use lesson::{
    GrammarInfo, GrammarQuizCard, LessonCard, LessonCardView, LessonData, LessonViewGenerator,
    MultiQuizResult, QuizCard, QuizMode, QuizOption, YesNoCard,
};
pub use lesson_builder::{MAX_LESSON_SIZE, NewCardPolicy};
pub(crate) use lesson_builder::{distribute_new_cards, jlpt_sort_key};
pub use phrase::PhraseCard;
pub use stats_tracker::StatsTracker;
pub use vocabulary::VocabularyCard;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use ulid::Ulid;

use crate::dictionary::kanji::get_kanji_info;
use crate::domain::{
    DailyBudget, JapaneseLevel, JlptContent, NativeLanguage, OrigaError, RateMode, Rating,
    srs::rate_memory,
};

pub(crate) const MAX_COMPANION_WORDS: usize = 3;

/// Collects the surface form of every vocabulary card whose memory state
/// qualifies as known. `include_in_progress` widens the predicate to also
/// accept in-progress cards (soft filter used by tail-phrase eligibility and
/// phrase seeding); pass `false` for the strict known-only view.
pub(crate) fn collect_known_vocabulary_words<'a, I>(
    cards: I,
    include_in_progress: bool,
) -> HashSet<String>
where
    I: IntoIterator<Item = &'a StudyCard>,
{
    cards
        .into_iter()
        .filter_map(|sc| match sc.card() {
            Card::Vocabulary(vocab) => {
                let known = sc.memory().is_known_card()
                    || (include_in_progress && sc.memory().is_in_progress());
                known.then(|| vocab.word().text().to_string())
            },
            _ => None,
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeSet {
    #[serde(deserialize_with = "deserialize_study_cards")]
    study_cards: HashMap<Ulid, StudyCard>,
    #[serde(default)]
    deleted_cards: HashSet<Ulid>,
    // Words of deleted Vocabulary cards. Consulted ONLY by
    // `create_companion_vocab_cards` so a word the user dismissed is never
    // auto-(re)introduced as a kanji companion on the next migration run.
    // Manual `create_card` always succeeds (it does not consult this set) and
    // evicts the word, so a dismissed word can still be re-added by hand.
    #[serde(default)]
    deleted_companion_words: HashSet<String>,
    #[serde(flatten)]
    stats: StatsTracker,
    // Transient bulk-import dedup index (see `begin_bulk_import`): never
    // serialized, rebuilt from `study_cards` when a bulk import starts, and
    // always `None` outside an import batch.
    #[serde(skip)]
    import_dedup_index: Option<HashSet<(CardType, String)>>,
}

fn deserialize_study_cards<'de, D>(deserializer: D) -> Result<HashMap<Ulid, StudyCard>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StudyCardsVisitor;

    impl<'de> serde::de::Visitor<'de> for StudyCardsVisitor {
        type Value = HashMap<Ulid, StudyCard>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a map of study cards")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut result = HashMap::new();
            while let Some(key) = map.next_key::<Ulid>()? {
                match map.next_value::<StudyCard>() {
                    Ok(value) => {
                        result.insert(key, value);
                    },
                    Err(e) => {
                        tracing::warn!("Skipping study card {}: {}", key, e);
                    },
                }
            }
            Ok(result)
        }
    }

    deserializer.deserialize_map(StudyCardsVisitor)
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
            deleted_cards: HashSet::new(),
            deleted_companion_words: HashSet::new(),
            stats: StatsTracker::new(),
            import_dedup_index: None,
        }
    }

    pub fn merge(&mut self, new_values: &KnowledgeSet) {
        // Merge is a sync path, never a bulk import: drop any active dedup
        // index so `create_card` falls back to the linear uniqueness scan
        // (merge itself validates uniqueness per card below).
        self.import_dedup_index = None;

        for deleted_id in &new_values.deleted_cards {
            self.study_cards.remove(deleted_id);
            self.deleted_cards.insert(*deleted_id);
        }

        // Union the dismissed-companion blocklist so post-sync auto-creation is
        // suppressed consistently across devices. Existing study cards are NOT
        // evicted: a legitimately user-created card for a blocklisted word on
        // the remote must survive the merge.
        self.deleted_companion_words
            .extend(new_values.deleted_companion_words.iter().cloned());

        for (id, study_card) in &new_values.study_cards {
            if self.deleted_cards.contains(id) {
                continue;
            }

            if let Some(existing_card) = self.study_cards.get_mut(id) {
                existing_card.merge(study_card);
            } else if self.validate_unique_card(study_card.card()).is_ok() {
                self.study_cards.insert(*id, study_card.clone());
            }
        }

        self.stats.merge(&new_values.stats);
        self.recalculate_daily_stats();
    }

    pub fn get_card(&self, card_id: Ulid) -> Option<&StudyCard> {
        self.study_cards.get(&card_id)
    }

    pub fn study_cards(&self) -> &HashMap<Ulid, StudyCard> {
        &self.study_cards
    }

    pub fn lesson_history(&self) -> &[DailyHistoryItem] {
        self.stats.history()
    }

    pub fn new_cards_studied_today(&self) -> usize {
        self.stats.new_cards_studied_today()
    }

    pub fn phrase_cards_studied_today(&self) -> usize {
        self.stats.phrase_cards_studied_today()
    }

    pub fn get_known_kanji(&self) -> HashSet<char> {
        self.study_cards
            .values()
            .filter_map(|study_card| match study_card.card() {
                Card::Kanji(kanji_card) if study_card.memory().is_known_card() => {
                    kanji_card.kanji().text().chars().next()
                },
                _ => None,
            })
            .collect()
    }

    pub fn delete_card(&mut self, card_id: Ulid) -> Result<(), OrigaError> {
        let removed = self
            .study_cards
            .remove(&card_id)
            .ok_or(OrigaError::CardNotFound { card_id })?;
        if let Card::Vocabulary(vocab) = removed.card() {
            self.deleted_companion_words
                .insert(vocab.word().text().to_string());
        }
        self.deleted_cards.insert(card_id);
        if let Some(index) = &mut self.import_dedup_index {
            index.remove(&Self::import_dedup_key(removed.card()));
        } else {
            self.recalculate_daily_stats();
        }
        Ok(())
    }

    pub fn update_card_content(&mut self, card_id: Ulid, new_card: Card) -> Result<(), OrigaError> {
        // Content replacement can break the bulk index invariant (one card ↔
        // one key: a `new_card` whose key is already owned by another card
        // would alias keys). Drop the index so subsequent `create_card` calls
        // degrade to the linear uniqueness scan — semantics preserved, and
        // the invariant cannot be violated. (Mirrors the `merge` policy.)
        self.import_dedup_index = None;
        let study_card = self
            .study_cards
            .get_mut(&card_id)
            .ok_or(OrigaError::CardNotFound { card_id })?;
        study_card.replace_card(new_card);
        Ok(())
    }

    pub fn create_card(&mut self, card: Card) -> Result<StudyCard, OrigaError> {
        let study_card = StudyCard::new(card);
        let card_id = *study_card.card_id();
        let dedup_key = Self::import_dedup_key(study_card.card());

        // Bulk-import mode checks uniqueness against the dedup index (O(1));
        // outside it, the linear scan keeps the single-card path as-is.
        if let Some(index) = &self.import_dedup_index {
            if index.contains(&dedup_key) {
                return Err(OrigaError::DuplicateCard {
                    question: study_card.card().content_key(),
                });
            }
        } else {
            self.validate_unique_card(study_card.card())?;
        }

        if self
            .study_cards
            .insert(card_id, study_card.clone())
            .is_some()
        {
            return Err(OrigaError::DuplicateCard {
                question: study_card.card().content_key(),
            });
        }

        // A manually (re)created Vocabulary card clears the word from the
        // dismissed-companion blocklist: the user explicitly reintroduced it,
        // so future companion auto-creation should not be suppressed by a stale
        // dismissal. The companion path skips blocklisted words BEFORE reaching
        // `create_card`, so this eviction never fires for companion cards.
        if let Card::Vocabulary(vocab) = study_card.card() {
            self.deleted_companion_words.remove(vocab.word().text());
        }

        if let Some(index) = &mut self.import_dedup_index {
            index.insert(dedup_key);
        } else {
            self.recalculate_daily_stats();
        }
        Ok(study_card)
    }

    /// Uniqueness identity of a card: card type + content key. Cross-type
    /// collisions are legitimate (a vocabulary word and a kanji may share the
    /// same surface text), so the type discriminates the key.
    fn import_dedup_key(card: &Card) -> (CardType, String) {
        (CardType::from(card), card.content_key())
    }

    /// Enters bulk-import mode: builds a dedup index over the current cards
    /// so `create_card` uniqueness checks are O(1) instead of a full scan,
    /// and daily-stats recalculation is deferred. Onboarding/Anki imports
    /// create thousands of cards; per-card full scans made those imports
    /// quadratic (the "N2 onboarding import is unusably slow" bug).
    pub(crate) fn begin_bulk_import(&mut self) {
        self.import_dedup_index = Some(
            self.study_cards
                .values()
                .map(|sc| Self::import_dedup_key(sc.card()))
                .collect(),
        );
    }

    /// Leaves bulk-import mode: drops the dedup index and recalculates the
    /// daily stats once for the whole batch. The recalc is a pure function
    /// of the final card set plus the day's preserved counters, so it lands
    /// on the same today-item the per-card recalcs would have (the only
    /// divergence is a batch spanning UTC midnight, where the per-card path
    /// would additionally snapshot the previous day).
    pub(crate) fn end_bulk_import(&mut self) {
        self.import_dedup_index = None;
        self.recalculate_daily_stats();
    }

    pub fn deleted_cards(&self) -> &HashSet<Ulid> {
        &self.deleted_cards
    }

    #[cfg(test)]
    pub fn deleted_companion_words_for_test(&self) -> &HashSet<String> {
        &self.deleted_companion_words
    }

    fn build_cards_by_type(&self) -> HashMap<CardType, Vec<Card>> {
        let mut map: HashMap<CardType, Vec<Card>> = HashMap::new();
        for study_card in self.study_cards.values() {
            let card_type = CardType::from(study_card.card());
            map.entry(card_type)
                .or_default()
                .push(study_card.card().clone());
        }
        map
    }

    fn validate_unique_card(&self, card: &Card) -> Result<(), OrigaError> {
        if self.study_cards.values().any(|c| match (card, c.card()) {
            (Card::Vocabulary(vocabulary_card), Card::Vocabulary(existing_vocabulary_card)) => {
                vocabulary_card.word() == existing_vocabulary_card.word()
            },
            (Card::Kanji(kanji_card), Card::Kanji(existing_kanji_card)) => {
                kanji_card.kanji() == existing_kanji_card.kanji()
            },
            (Card::Grammar(grammar_rule_card), Card::Grammar(existing_grammar_rule_card)) => {
                grammar_rule_card.rule_id() == existing_grammar_rule_card.rule_id()
            },
            (Card::Phrase(phrase_card), Card::Phrase(existing_phrase_card)) => {
                phrase_card.phrase_id() == existing_phrase_card.phrase_id()
            },

            _ => false,
        }) {
            return Err(OrigaError::DuplicateCard {
                question: card.content_key(),
            });
        }

        Ok(())
    }

    pub fn cards_to_lesson(
        &self,
        budget: DailyBudget,
        jlpt_content: &JlptContent,
        user_level: JapaneseLevel,
        native_language: NativeLanguage,
    ) -> LessonData {
        self.cards_to_lesson_with_policy(
            budget,
            jlpt_content,
            user_level,
            native_language,
            NewCardPolicy::Inject,
        )
    }

    /// Вариант с явной политикой новых карт (docs/acquaintance-mode.md §9.3
    /// S3): `Exclude` держит незнакомые карты вне урока независимо от пути
    /// попадания (впрыск/избранное/padding/companions); фразы освобождены.
    pub fn cards_to_lesson_with_policy(
        &self,
        budget: DailyBudget,
        jlpt_content: &JlptContent,
        user_level: JapaneseLevel,
        native_language: NativeLanguage,
        new_card_policy: NewCardPolicy,
    ) -> LessonData {
        let (core, primary_card_ids) = lesson_builder::build_lesson_core(
            self,
            budget.new_cards_per_day(),
            jlpt_content,
            native_language,
            new_card_policy,
        );
        let with_companions =
            kanji_companions::add_kanji_companions(core, self, user_level, native_language);
        let with_companions = match new_card_policy {
            NewCardPolicy::Inject => with_companions,
            NewCardPolicy::Exclude => {
                lesson_builder::drop_new_cards(with_companions, self, new_card_policy)
            },
        };
        let interleaved = lesson_builder::interleave_core_by_type(with_companions);
        // NEW anchored phrases are capped per LESSON (not per day): every
        // lesson of the day receives the full allowance, so an evening
        // lesson is no longer starved by a morning one (see DailyBudget).
        let mut phrase_new_budget = budget.new_phrases_per_lesson();
        let with_phrases =
            lesson_builder::add_phrases(interleaved, self, native_language, &mut phrase_new_budget);
        let expanded = lesson_builder::expand_repeated_views(
            with_phrases,
            self,
            native_language,
            &primary_card_ids,
        );
        lesson_builder::redistribute_core_for_spacing(expanded)
    }

    pub(crate) fn rate_card(
        &mut self,
        card_id: Ulid,
        rating: Rating,
        mode: RateMode,
    ) -> Result<(), OrigaError> {
        if let Some(card) = self.study_cards.get_mut(&card_id) {
            let was_new = card.memory().is_new();
            let is_phrase = matches!(card.card(), Card::Phrase(_));
            let effective_mode = match mode {
                RateMode::ShortTerm | RateMode::OnboardingScoring => mode,
                _ => match card.card() {
                    Card::Phrase(_) => RateMode::PhraseReview,
                    Card::Grammar(_) => RateMode::GrammarReview,
                    Card::Kanji(_) => RateMode::KanjiReview,
                    Card::Vocabulary(_) => mode,
                },
            };

            let memory_state = rate_memory(effective_mode, rating, card.memory())?;
            card.apply_review(memory_state, rating);
            card.handle_favorite_rating(rating);
            self.update_history(rating, was_new, is_phrase, mode);
            Ok(())
        } else {
            Err(OrigaError::CardNotFound { card_id })
        }
    }

    pub(crate) fn toggle_favorite(&mut self, card_id: Ulid) -> Result<(), OrigaError> {
        self.study_cards
            .get_mut(&card_id)
            .map(|card| card.toggle_favorite())
            .ok_or(OrigaError::CardNotFound { card_id })
    }

    fn update_history(&mut self, rating: Rating, was_new: bool, is_phrase: bool, mode: RateMode) {
        self.stats
            .update(&self.study_cards, rating, was_new, is_phrase, mode);
    }

    pub fn create_companion_vocab_cards(
        &mut self,
        kanji_char: &str,
        native_language: &NativeLanguage,
    ) -> Vec<StudyCard> {
        let kanji_info = match get_kanji_info(kanji_char) {
            Ok(info) => info,
            Err(_) => {
                tracing::debug!(kanji = %kanji_char, "Kanji not found, skipping companion creation");
                return Vec::new();
            },
        };

        let mut created = Vec::new();
        for word in kanji_info.popular_words().iter().take(MAX_COMPANION_WORDS) {
            if self.deleted_companion_words.contains(word.as_str()) {
                tracing::debug!(
                    kanji = %kanji_char,
                    word = %word,
                    "Companion word previously dismissed by user, skipping"
                );
                continue;
            }
            match VocabularyCard::from_known_word(word, native_language) {
                Ok(vocab_card) => match self.create_card(Card::Vocabulary(vocab_card)) {
                    Ok(study_card) => {
                        tracing::debug!(kanji = %kanji_char, word = %word, "Companion vocab card created");
                        created.push(study_card);
                    },
                    Err(OrigaError::DuplicateCard { .. }) => {
                        tracing::debug!(kanji = %kanji_char, word = %word, "Companion already exists, skipping");
                    },
                    Err(e) => {
                        tracing::warn!(kanji = %kanji_char, word = %word, error = %e, "Failed to create companion card");
                    },
                },
                Err(_) => {
                    tracing::debug!(kanji = %kanji_char, word = %word, "No translation for companion word, skipping");
                },
            }
        }
        created
    }

    fn recalculate_daily_stats(&mut self) {
        self.stats.recalculate(&self.study_cards);
    }

    pub fn mark_card_as_known(&mut self, card_id: Ulid) -> Result<(), OrigaError> {
        use crate::domain::memory::{
            Difficulty, KNOWN_CARD_STABILITY_THRESHOLD, MemoryState, Rating, Stability,
        };
        use chrono::{Duration, Utc};

        if let Some(card) = self.study_cards.get_mut(&card_id) {
            let stability = KNOWN_CARD_STABILITY_THRESHOLD + 1.0;
            let memory = MemoryState::new(
                Stability::new(stability).unwrap(),
                Difficulty::new(3.0).unwrap(),
                Utc::now() - Duration::days(1),
            );
            card.apply_review(memory, Rating::Easy);
            card.handle_favorite_rating(Rating::Easy);
            Ok(())
        } else {
            Err(OrigaError::CardNotFound { card_id })
        }
    }

    /// Закрытие руки знакомства (docs/acquaintance-mode.md): каждой ещё
    /// новой карте руки сидируется состояние памяти с первым ревью в
    /// `first_due`; уже незнакомость потерявшие карты пропускаются
    /// (идемпотентность как у `mark_card_as_known`). Дневной лимит
    /// списывается одной операцией за фактически сидированные карты.
    pub fn complete_acquaintance_hand(
        &mut self,
        card_ids: &[Ulid],
        first_due: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), OrigaError> {
        use crate::domain::acquaintance::build_seeded_memory_state;

        let mut seeded = 0usize;
        for card_id in card_ids {
            let Some(card) = self.study_cards.get_mut(card_id) else {
                return Err(OrigaError::CardNotFound { card_id: *card_id });
            };
            if !card.memory().is_new() {
                continue;
            }
            let memory_state = build_seeded_memory_state(first_due)?;
            card.seed_first_review(memory_state);
            seeded += 1;
        }

        if seeded > 0 {
            self.stats
                .register_acquaintance_completions(&self.study_cards, seeded);
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn study_cards_mut_for_test(&mut self) -> &mut HashMap<Ulid, StudyCard> {
        &mut self.study_cards
    }
}
