use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::dictionary::vocabulary::get_translation;
use crate::domain::{
    Card, CardType, DailyLoad, JapaneseLevel, JlptContent, JlptProgress, KnowledgeSet,
    NativeLanguage, OrigaError, RateMode, Rating, ScoreContentResult, StudyCard, score_content,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordKnowledge {
    pub is_known: bool,
    pub meaning: Option<String>,
}

/// Marker stored inside `imported_sets` to signal that the user has finished
/// onboarding scoring. The companion `ONBOARDING_SKIPPED_KEY` marker (written
/// by the skip path) is treated equivalently by [`User::is_onboarding_completed`].
/// Reusing `imported_sets` avoids a schema migration: the field is already
/// `#[serde(default)]` and synced via `save_sync`.
pub const ONBOARDING_COMPLETED_KEY: &str = "__onboarding_completed__";
pub const ONBOARDING_SKIPPED_KEY: &str = "__onboarding_skipped__";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: Ulid,
    email: String,
    username: String,
    native_language: NativeLanguage,
    jlpt_progress: JlptProgress,
    #[serde(
        default,
        serialize_with = "crate::domain::serde_utils::option_u64_as_string::serialize",
        deserialize_with = "crate::domain::serde_utils::option_u64_as_string::deserialize"
    )]
    telegram_user_id: Option<u64>,
    knowledge_set: KnowledgeSet,

    #[serde(default)]
    updated_at: DateTime<Utc>,

    #[serde(default)]
    imported_sets: HashSet<String>,

    #[serde(default)]
    daily_load: DailyLoad,

    #[serde(default)]
    known_vocab_hash: u32,

    /// Cards the user marked "don't know" during onboarding scoring. Persisted
    /// per-click via `save_sync` so the scoring step can resume after an app
    /// restart and skip already-evaluated cards. Cleared by
    /// [`crate::use_cases::CompleteOnboardingScoringUseCase`] when onboarding
    /// finishes.
    #[serde(default)]
    onboarding_scoring_skipped: HashSet<Ulid>,
}

impl User {
    pub fn new(
        email: String,
        native_language: NativeLanguage,
        telegram_user_id: Option<u64>,
    ) -> Self {
        Self {
            id: Ulid::new(),
            username: email.split('@').next().unwrap_or(&email).to_string(),
            email,
            knowledge_set: KnowledgeSet::new(),
            jlpt_progress: JlptProgress::new(),
            native_language,
            telegram_user_id,
            updated_at: Utc::now(),
            imported_sets: HashSet::new(),
            daily_load: DailyLoad::default(),
            known_vocab_hash: 0,
            onboarding_scoring_skipped: HashSet::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_row(
        id: Ulid,
        email: String,
        username: String,
        jlpt_progress: JlptProgress,
        native_language: NativeLanguage,
        telegram_user_id: Option<u64>,
        knowledge_set: KnowledgeSet,
        updated_at: DateTime<Utc>,
        imported_sets: HashSet<String>,
        daily_load: DailyLoad,
        known_vocab_hash: u32,
    ) -> Self {
        Self {
            id,
            email,
            username,
            jlpt_progress,
            native_language,
            telegram_user_id,
            knowledge_set,
            updated_at,
            imported_sets,
            daily_load,
            known_vocab_hash,
            onboarding_scoring_skipped: HashSet::new(),
        }
    }

    pub fn merge(&mut self, another_user: &User) {
        // Remote is the source of truth for identity: a local record must not
        // override the canonical user id, otherwise saves on the same browser
        // get attributed to different ids and break cross-device sync.
        //
        // Defensive guard: a nil remote id means the remote trailbase_id failed
        // to decode. Propagating nil here would re-introduce the bug this merge
        // change is meant to prevent, so when the remote id is nil the local id
        // is left untouched (non-identity fields are still merged). Callers that
        // must never observe a nil remote should reject it upstream — see
        // `TrailBaseUserRepository::find_current`, which returns an error before
        // reaching merge when the decoded id is nil.
        if another_user.id != Ulid::nil() {
            self.id = another_user.id;
        }
        self.email = another_user.email.clone();
        self.username = another_user.username.clone();
        self.native_language = another_user.native_language;
        self.telegram_user_id = another_user.telegram_user_id;
        self.daily_load = another_user.daily_load;

        self.knowledge_set.merge(&another_user.knowledge_set);

        for set_id in &another_user.imported_sets {
            self.imported_sets.insert(set_id.clone());
        }

        for card_id in &another_user.onboarding_scoring_skipped {
            self.onboarding_scoring_skipped.insert(*card_id);
        }

        self.touch();
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn set_username(&mut self, username: String) {
        self.username = username;
    }

    pub fn current_japanese_level(&self) -> JapaneseLevel {
        self.jlpt_progress.current_level()
    }

    pub fn jlpt_progress(&self) -> &JlptProgress {
        &self.jlpt_progress
    }

    pub fn native_language(&self) -> &NativeLanguage {
        &self.native_language
    }

    pub fn set_native_language(&mut self, native_language: NativeLanguage) {
        self.native_language = native_language
    }

    pub fn knowledge_set(&self) -> &KnowledgeSet {
        &self.knowledge_set
    }

    #[cfg(test)]
    pub fn knowledge_set_mut(&mut self) -> &mut KnowledgeSet {
        &mut self.knowledge_set
    }

    pub fn telegram_user_id(&self) -> Option<&u64> {
        self.telegram_user_id.as_ref()
    }

    pub fn set_telegram_user_id(&mut self, telegram_user_id: Option<u64>) {
        self.telegram_user_id = telegram_user_id;
    }

    pub fn daily_load(&self) -> &DailyLoad {
        &self.daily_load
    }

    pub fn set_daily_load(&mut self, daily_load: DailyLoad) {
        self.daily_load = daily_load;
        self.touch();
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn mark_set_as_imported(&mut self, set_id: String) {
        self.imported_sets.insert(set_id);
        self.touch();
    }

    pub fn mark_sets_as_imported(&mut self, set_ids: impl IntoIterator<Item = String>) {
        for set_id in set_ids {
            self.imported_sets.insert(set_id);
        }
        self.touch();
    }

    pub fn is_set_imported(&self, set_id: &str) -> bool {
        self.imported_sets.contains(set_id)
    }

    pub fn imported_sets(&self) -> &HashSet<String> {
        &self.imported_sets
    }

    pub fn known_vocab_hash(&self) -> u32 {
        self.known_vocab_hash
    }

    pub fn set_known_vocab_hash(&mut self, hash: u32) {
        self.known_vocab_hash = hash;
        self.touch();
    }

    /// Marks the onboarding scoring as finished. The user is no longer
    /// redirected to `/onboarding` on the next app start. Stored as a sentinel
    /// inside `imported_sets` so no schema migration is required.
    pub fn mark_onboarding_completed(&mut self) {
        self.imported_sets
            .insert(ONBOARDING_COMPLETED_KEY.to_string());
        self.touch();
    }

    /// Returns `true` when onboarding has reached a terminal state (either the
    /// user finished scoring or skipped it). Used as the single routing guard
    /// across `/home` and `/onboarding`.
    pub fn is_onboarding_completed(&self) -> bool {
        self.imported_sets.contains(ONBOARDING_COMPLETED_KEY)
            || self.imported_sets.contains(ONBOARDING_SKIPPED_KEY)
    }

    /// Records a card the user marked "don't know" during onboarding scoring.
    /// Persisted through the regular `save` path so the scoring step can resume
    /// after an app restart and skip cards already evaluated.
    pub fn mark_card_skipped_in_onboarding(&mut self, card_id: Ulid) {
        self.onboarding_scoring_skipped.insert(card_id);
        self.touch();
    }

    pub fn is_card_skipped_in_onboarding(&self, card_id: &Ulid) -> bool {
        self.onboarding_scoring_skipped.contains(card_id)
    }

    pub fn onboarding_scoring_skipped(&self) -> &HashSet<Ulid> {
        &self.onboarding_scoring_skipped
    }

    /// Clears every "don't know" record collected during onboarding scoring.
    /// Called by [`crate::use_cases::CompleteOnboardingScoringUseCase`] when
    /// onboarding reaches its terminal state.
    pub fn clear_onboarding_scoring_skipped(&mut self) {
        if !self.onboarding_scoring_skipped.is_empty() {
            self.onboarding_scoring_skipped.clear();
            self.touch();
        }
    }

    pub fn is_word_known(&self, word: &str) -> WordKnowledge {
        let meaning = get_translation(word, self.native_language());

        for study_card in self.knowledge_set().study_cards().values() {
            if let Card::Vocabulary(vocab_card) = study_card.card()
                && vocab_card.word().text() == word
            {
                return WordKnowledge {
                    is_known: true,
                    meaning,
                };
            }
        }

        WordKnowledge {
            is_known: false,
            meaning,
        }
    }

    pub fn rate_card(
        &mut self,
        card_id: Ulid,
        rating: Rating,
        mode: RateMode,
    ) -> Result<(), OrigaError> {
        self.knowledge_set.rate_card(card_id, rating, mode)
    }

    pub fn mark_card_as_known(&mut self, card_id: Ulid) -> Result<(), OrigaError> {
        self.knowledge_set.mark_card_as_known(card_id)
    }

    pub fn delete_card(&mut self, card_id: Ulid) -> Result<(), OrigaError> {
        self.knowledge_set.delete_card(card_id)
    }

    pub fn create_card(&mut self, card: Card) -> Result<StudyCard, OrigaError> {
        self.knowledge_set.create_card(card)
    }

    pub fn update_card_content(&mut self, card_id: Ulid, new_card: Card) -> Result<(), OrigaError> {
        self.knowledge_set.update_card_content(card_id, new_card)
    }

    pub fn create_companion_vocab_cards(&mut self, kanji_char: &str) -> Vec<StudyCard> {
        self.knowledge_set
            .create_companion_vocab_cards(kanji_char, &self.native_language)
    }

    pub fn score_content(&self, content: &str) -> Result<ScoreContentResult, OrigaError> {
        score_content::score_content(content, self.knowledge_set().study_cards())
    }

    pub fn toggle_favorite(&mut self, card_id: Ulid) -> Result<(), OrigaError> {
        self.knowledge_set.toggle_favorite(card_id)
    }

    pub fn recalculate_jlpt_progress(&mut self, content: &JlptContent) {
        use crate::domain::jlpt_progress::{CategoryCounts, ProgressUpdate};

        let mut learned = CategoryCounts::default();
        let mut projected = CategoryCounts::default();

        for study_card in self.knowledge_set.study_cards().values() {
            let memory = study_card.memory();
            let card = study_card.card();
            let Some(level) = content.find_level(&card.content_key(), CardType::from(card)) else {
                continue;
            };

            // `learned` counts cards stabilized past the known-card threshold.
            // `projected` counts cards in flight — `hard` or `in-progress`, but
            // explicitly NOT `learned` (otherwise `projected_percentage` would
            // double-count them and exceed 100%).
            let is_learned = memory.is_known_card();
            let is_in_flight = !memory.is_new() && !is_learned;

            if !is_learned && !is_in_flight {
                continue;
            }

            // Pick the (learned, projected) bucket pair for this card type.
            // Phrase cards roll up under `words` — see domain/jlpt_content.rs.
            let (learned_map, projected_map) = match card {
                Card::Kanji(_) => (&mut learned.kanji, &mut projected.kanji),
                Card::Vocabulary(_) | Card::Phrase(_) => (&mut learned.words, &mut projected.words),
                Card::Grammar(_) => (&mut learned.grammar, &mut projected.grammar),
            };

            if is_learned {
                *learned_map.entry(level).or_insert(0) += 1;
            } else {
                *projected_map.entry(level).or_insert(0) += 1;
            }
        }

        let total = CategoryCounts {
            kanji: Self::build_totals(&content.kanji_by_level),
            words: Self::build_totals(&content.words_by_level),
            grammar: Self::build_totals(&content.grammar_by_level),
        };

        self.jlpt_progress.recalculate(ProgressUpdate {
            learned,
            projected,
            total,
        });
    }

    fn build_totals(
        content: &HashMap<JapaneseLevel, std::collections::HashSet<String>>,
    ) -> HashMap<JapaneseLevel, usize> {
        content
            .iter()
            .map(|(level, set)| (*level, set.len()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::Question;
    use crate::domain::{RateMode, VocabularyCard};

    fn create_test_vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    fn create_test_content_with_words(words: &[(&str, JapaneseLevel)]) -> JlptContent {
        let mut content = JlptContent::new();
        for (word, level) in words {
            content
                .words_by_level
                .entry(*level)
                .or_default()
                .insert(word.to_string());
        }
        content
    }

    #[test]
    fn user_new_creates_default_jlpt_progress() {
        let user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        assert_eq!(user.current_japanese_level(), JapaneseLevel::N5);
        assert_eq!(user.email(), "test@example.com");
        assert_eq!(user.username(), "test");
    }

    #[test]
    fn user_current_japanese_level_returns_from_progress() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let complete = crate::domain::jlpt_progress::LevelProgressDetail {
            kanji: crate::domain::jlpt_progress::CategoryProgress {
                learned: 100,
                projected: 0,
                total: 100,
            },
            words: crate::domain::jlpt_progress::CategoryProgress {
                learned: 100,
                projected: 0,
                total: 100,
            },
            grammar: crate::domain::jlpt_progress::CategoryProgress {
                learned: 100,
                projected: 0,
                total: 100,
            },
        };

        user.jlpt_progress.update_level(JapaneseLevel::N5, complete);
        assert_eq!(user.current_japanese_level(), JapaneseLevel::N4);
    }

    #[test]
    fn user_recalculate_jlpt_progress_counts_learned_cards() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let card1 = create_test_vocab_card("猫");
        let card2 = create_test_vocab_card("犬");
        let study_card1 = user.create_card(card1).unwrap();
        let _study_card2 = user.create_card(card2).unwrap();

        user.rate_card(
            *study_card1.card_id(),
            Rating::Easy,
            RateMode::StandardLesson,
        )
        .unwrap();
        user.knowledge_set_mut()
            .mark_card_as_known(*study_card1.card_id())
            .unwrap();

        let content =
            create_test_content_with_words(&[("猫", JapaneseLevel::N5), ("犬", JapaneseLevel::N5)]);

        user.recalculate_jlpt_progress(&content);

        let n5_progress = user
            .jlpt_progress()
            .level_progress(JapaneseLevel::N5)
            .unwrap();
        assert_eq!(n5_progress.words.learned, 1);
        assert_eq!(n5_progress.words.total, 2);
    }

    #[test]
    fn user_recalculate_jlpt_progress_ignores_unknown_cards() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let card = create_test_vocab_card("猫");
        user.create_card(card).unwrap();

        let content = create_test_content_with_words(&[("猫", JapaneseLevel::N5)]);

        user.recalculate_jlpt_progress(&content);

        let n5_progress = user
            .jlpt_progress()
            .level_progress(JapaneseLevel::N5)
            .unwrap();
        assert_eq!(n5_progress.words.learned, 0);
    }

    #[test]
    fn user_recalculate_jlpt_progress_handles_empty_content() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let card = create_test_vocab_card("猫");
        let study_card = user.create_card(card).unwrap();
        user.rate_card(
            *study_card.card_id(),
            Rating::Easy,
            RateMode::StandardLesson,
        )
        .unwrap();

        let content = JlptContent::new();
        user.recalculate_jlpt_progress(&content);

        let n5_progress = user
            .jlpt_progress()
            .level_progress(JapaneseLevel::N5)
            .unwrap();
        assert_eq!(n5_progress.words.learned, 0);
        assert_eq!(n5_progress.words.total, 0);
    }

    #[test]
    fn user_recalculate_jlpt_progress_separates_learned_from_in_flight() {
        // Arrange — three N5 word cards: one untouched (new), one marked known,
        // one rated once (in-progress). projected must contain ONLY the
        // in-progress card; learned must NOT be double-counted into projected.
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let new_card = create_test_vocab_card("本");
        let known_card = create_test_vocab_card("猫");
        let in_progress_card = create_test_vocab_card("犬");

        // `new_card` is intentionally left untouched — it is the control for
        // "brand-new card contributes to neither learned nor projected".
        let _ = user.create_card(new_card).unwrap();
        let known_id = user.create_card(known_card).unwrap();
        let in_progress_id = user.create_card(in_progress_card).unwrap();

        user.knowledge_set_mut()
            .mark_card_as_known(*known_id.card_id())
            .unwrap();
        user.rate_card(
            *in_progress_id.card_id(),
            Rating::Good,
            RateMode::StandardLesson,
        )
        .unwrap();

        let content = create_test_content_with_words(&[
            ("本", JapaneseLevel::N5),
            ("猫", JapaneseLevel::N5),
            ("犬", JapaneseLevel::N5),
        ]);

        // Act
        user.recalculate_jlpt_progress(&content);

        // Assert — learned = 1, projected = 1 (in-progress only, no double count)
        let n5_progress = user
            .jlpt_progress()
            .level_progress(JapaneseLevel::N5)
            .unwrap();
        assert_eq!(n5_progress.words.learned, 1, "only 猫 is known");
        assert_eq!(n5_progress.words.projected, 1, "only 犬 is in-flight");
        assert_eq!(n5_progress.words.total, 3);
        // projected_percentage = (learned + projected) / total = 2/3
        assert!(
            (n5_progress.words.projected_percentage() - 66.6667).abs() < 0.01,
            "projected_percentage must be (learned + projected) / total, got {}",
            n5_progress.words.projected_percentage()
        );
        assert!(n5_progress.words.projected_percentage() <= 100.0);
    }

    #[test]
    fn user_mark_set_as_imported_adds_to_set() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        assert!(!user.is_set_imported("set-123"));

        user.mark_set_as_imported("set-123".to_string());

        assert!(user.is_set_imported("set-123"));
        assert!(user.imported_sets().contains("set-123"));
    }

    #[test]
    fn user_is_set_imported_returns_true_for_imported() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        user.mark_set_as_imported("set-abc".to_string());

        assert!(user.is_set_imported("set-abc"));
    }

    #[test]
    fn user_is_set_imported_returns_false_for_not_imported() {
        let user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        assert!(!user.is_set_imported("set-xyz"));
        assert!(!user.is_set_imported("nonexistent-set"));
    }

    #[test]
    fn user_merge_merges_imported_sets() {
        let mut user1 = User::new(
            "user1@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        user1.mark_set_as_imported("set-1".to_string());

        let mut user2 = User::new(
            "user2@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        user2.mark_set_as_imported("set-2".to_string());
        user2.mark_set_as_imported("set-3".to_string());

        user1.merge(&user2);

        assert!(user1.is_set_imported("set-1"));
        assert!(user1.is_set_imported("set-2"));
        assert!(user1.is_set_imported("set-3"));
        assert_eq!(user1.imported_sets().len(), 3);
    }

    #[test]
    fn user_merge_takes_identity_from_remote() {
        // Remote is the source of truth for identity. A local user created with
        // a random ULID must adopt the remote id on merge so that subsequent
        // saves are attributed to the correct canonical user across devices.
        let mut local = User::new(
            "local@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let local_random_id = local.id();
        assert_ne!(local_random_id, Ulid::nil());

        let remote_id = Ulid::from_bytes([
            0x01, 0x67, 0x4f, 0x3a, 0x4e, 0x32, 0xdf, 0x41, 0x8c, 0x6b, 0x27, 0x4e, 0x73, 0x91,
            0x5f, 0xce,
        ]);
        let remote = User::from_row(
            remote_id,
            "remote@example.com".to_string(),
            "remote".to_string(),
            JlptProgress::new(),
            NativeLanguage::Russian,
            None,
            KnowledgeSet::new(),
            Utc::now(),
            HashSet::new(),
            DailyLoad::default(),
            0,
        );

        local.merge(&remote);

        assert_eq!(local.id(), remote_id);
        assert_ne!(local.id(), local_random_id);
        assert_ne!(local.id(), Ulid::nil());
    }

    #[test]
    fn user_merge_does_not_adopt_nil_remote_id() {
        // Regression guard for the nil-identity bug: a remote row whose id
        // failed to decode (nil) must not poison the local id. Otherwise the
        // "remote is source of truth" rule would re-introduce the cross-device
        // sync corruption that merge is meant to fix.
        let mut local = User::new(
            "local@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let local_id_before = local.id();
        assert_ne!(local_id_before, Ulid::nil());

        let nil_remote = User::from_row(
            Ulid::nil(),
            "remote@example.com".to_string(),
            "remote".to_string(),
            JlptProgress::new(),
            NativeLanguage::Russian,
            None,
            KnowledgeSet::new(),
            Utc::now(),
            HashSet::new(),
            DailyLoad::default(),
            0,
        );

        local.merge(&nil_remote);

        assert_eq!(local.id(), local_id_before);
        assert_ne!(local.id(), Ulid::nil());
        // Non-identity fields are still merged.
        assert_eq!(local.email(), "remote@example.com");
    }

    #[test]
    fn user_merge_preserves_jlpt_progress() {
        // Arrange
        let mut user1 = User::new(
            "user1@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let mut user2 = User::new(
            "user2@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let complete = crate::domain::jlpt_progress::LevelProgressDetail {
            kanji: crate::domain::jlpt_progress::CategoryProgress {
                learned: 100,
                projected: 0,
                total: 100,
            },
            words: crate::domain::jlpt_progress::CategoryProgress {
                learned: 100,
                projected: 0,
                total: 100,
            },
            grammar: crate::domain::jlpt_progress::CategoryProgress {
                learned: 100,
                projected: 0,
                total: 100,
            },
        };
        user2
            .jlpt_progress
            .update_level(JapaneseLevel::N5, complete);

        let progress_before = user1.jlpt_progress().clone();

        // Act
        user1.merge(&user2);

        // Assert
        assert_eq!(user1.jlpt_progress(), &progress_before);
        assert_eq!(user1.current_japanese_level(), JapaneseLevel::N5);
    }

    #[test]
    fn user_is_word_known_returns_correct_knowledge() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let word = "猫";
        let knowledge_before = user.is_word_known(word);
        assert!(!knowledge_before.is_known);

        let card = create_test_vocab_card(word);
        user.create_card(card).unwrap();

        let knowledge_after = user.is_word_known(word);
        assert!(knowledge_after.is_known);
    }

    #[test]
    fn user_rate_card_nonexistent_returns_card_not_found() {
        // Arrange
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let nonexistent_id = Ulid::new();

        // Act
        let result = user.rate_card(nonexistent_id, Rating::Good, RateMode::StandardLesson);

        // Assert
        assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
        if let Err(OrigaError::CardNotFound { card_id }) = result {
            assert_eq!(card_id, nonexistent_id);
        }
    }

    #[test]
    fn user_delete_card_nonexistent_returns_card_not_found() {
        // Arrange
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let nonexistent_id = Ulid::new();

        // Act
        let result = user.delete_card(nonexistent_id);

        // Assert
        assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
        if let Err(OrigaError::CardNotFound { card_id }) = result {
            assert_eq!(card_id, nonexistent_id);
        }
    }

    #[test]
    fn user_create_card_duplicate_returns_duplicate_card() {
        // Arrange
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let word = "猫";
        let card1 = create_test_vocab_card(word);
        let card2 = create_test_vocab_card(word);

        // Act
        let result1 = user.create_card(card1);
        let result2 = user.create_card(card2);

        // Assert
        assert!(result1.is_ok());
        assert!(matches!(result2, Err(OrigaError::DuplicateCard { .. })));
        if let Err(OrigaError::DuplicateCard { question }) = result2 {
            assert_eq!(question, word);
        }
    }

    #[test]
    fn user_toggle_favorite_nonexistent_returns_card_not_found() {
        // Arrange
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let nonexistent_id = Ulid::new();

        // Act
        let result = user.toggle_favorite(nonexistent_id);

        // Assert
        assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
        if let Err(OrigaError::CardNotFound { card_id }) = result {
            assert_eq!(card_id, nonexistent_id);
        }
    }

    #[test]
    fn user_rate_card_after_delete_returns_card_not_found() {
        // Arrange
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let card = create_test_vocab_card("猫");
        let study_card = user.create_card(card).unwrap();
        let card_id = *study_card.card_id();

        // Act
        user.delete_card(card_id).unwrap();
        let result = user.rate_card(card_id, Rating::Good, RateMode::StandardLesson);

        // Assert
        assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
    }

    #[test]
    fn user_jlpt_progress_recalculates_after_knowledge_changes() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        let card = create_test_vocab_card("猫");
        let study_card = user.create_card(card).unwrap();
        let card_id = *study_card.card_id();

        let content = create_test_content_with_words(&[("猫", JapaneseLevel::N5)]);

        // Before learning: progress should be 0
        user.recalculate_jlpt_progress(&content);
        let n5 = user
            .jlpt_progress()
            .level_progress(JapaneseLevel::N5)
            .unwrap();
        assert_eq!(n5.words.learned, 0);
        assert_eq!(n5.words.total, 1);

        // Rate and mark as known
        user.rate_card(card_id, Rating::Easy, RateMode::StandardLesson)
            .unwrap();
        user.knowledge_set_mut()
            .mark_card_as_known(card_id)
            .unwrap();

        // After learning: progress should reflect the known card
        user.recalculate_jlpt_progress(&content);
        let n5 = user
            .jlpt_progress()
            .level_progress(JapaneseLevel::N5)
            .unwrap();
        assert_eq!(n5.words.learned, 1);
        assert_eq!(n5.words.total, 1);
        assert!(n5.words.percentage() > 0.0);
    }

    #[test]
    fn new_user_has_empty_onboarding_scoring_skipped() {
        let user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        assert!(user.onboarding_scoring_skipped().is_empty());
        assert!(!user.is_onboarding_completed());
    }

    #[test]
    fn mark_card_skipped_in_onboarding_adds_to_set() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let card_id = Ulid::new();

        user.mark_card_skipped_in_onboarding(card_id);

        assert!(user.is_card_skipped_in_onboarding(&card_id));
        assert_eq!(user.onboarding_scoring_skipped().len(), 1);
    }

    #[test]
    fn mark_card_skipped_in_onboarding_is_idempotent() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let card_id = Ulid::new();

        user.mark_card_skipped_in_onboarding(card_id);
        user.mark_card_skipped_in_onboarding(card_id);

        assert_eq!(user.onboarding_scoring_skipped().len(), 1);
    }

    #[test]
    fn is_card_skipped_in_onboarding_returns_false_for_unknown_id() {
        let user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        assert!(!user.is_card_skipped_in_onboarding(&Ulid::new()));
    }

    #[test]
    fn clear_onboarding_scoring_skipped_empties_set() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        user.mark_card_skipped_in_onboarding(Ulid::new());
        user.mark_card_skipped_in_onboarding(Ulid::new());

        user.clear_onboarding_scoring_skipped();

        assert!(user.onboarding_scoring_skipped().is_empty());
    }

    #[test]
    fn mark_onboarding_completed_makes_is_completed_true() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );

        user.mark_onboarding_completed();

        assert!(user.is_onboarding_completed());
    }

    #[test]
    fn is_onboarding_completed_returns_true_for_legacy_skipped_marker() {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        user.mark_set_as_imported(ONBOARDING_SKIPPED_KEY.to_string());

        assert!(user.is_onboarding_completed());
    }

    #[test]
    fn user_deserializes_old_json_without_skipped_field() {
        // Arrange — simulate a legacy User payload that predates the
        // onboarding_scoring_skipped field by serializing a current user and
        // then stripping the field from the JSON.
        let user = User::new(
            "legacy@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let mut json = serde_json::to_value(&user).expect("serialize user");
        let obj = json
            .as_object_mut()
            .expect("User must serialize to a JSON object");
        assert!(
            obj.remove("onboarding_scoring_skipped").is_some(),
            "precondition: field exists in current serialization"
        );

        // Act
        let deserialized: User = serde_json::from_value(json)
            .expect("legacy JSON without skipped field must deserialize");

        // Assert
        assert_eq!(deserialized.email(), "legacy@example.com");
        assert!(deserialized.onboarding_scoring_skipped().is_empty());
    }
}
