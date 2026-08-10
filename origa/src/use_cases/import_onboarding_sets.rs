use std::collections::HashSet;

use tracing::{debug, info, warn};

use crate::dictionary::grammar::get_rules_by_level;
use crate::dictionary::kanji::get_kanji_list;
use crate::domain::resolve_set_path;
use crate::domain::{
    Card, GrammarRuleCard, JapaneseLevel, KanjiCard, OrigaError, StudyCard, User, VocabularyCard,
    WellKnownSet,
};
use crate::traits::{CdnProvider, UserRepository};

pub struct ImportOnboardingResult {
    pub imported_set_ids: Vec<String>,
    pub created_vocabulary: usize,
    pub created_kanji: usize,
    pub created_grammar: usize,
    pub skipped_duplicates: usize,
    pub skipped_no_translation: usize,
}

#[derive(Clone)]
pub struct ImportOnboardingSetsUseCase<'a, R: UserRepository, C: CdnProvider> {
    repository: &'a R,
    cdn: &'a C,
}

impl<'a, R: UserRepository, C: CdnProvider> ImportOnboardingSetsUseCase<'a, R, C> {
    pub fn new(repository: &'a R, cdn: &'a C) -> Self {
        Self { repository, cdn }
    }

    pub async fn execute(
        &self,
        mut user: User,
        set_ids: Vec<String>,
        target_level: JapaneseLevel,
    ) -> Result<ImportOnboardingResult, OrigaError> {
        debug!(user_id = %user.id(), set_count = set_ids.len(), ?target_level, "Starting onboarding sets import");

        let native_language = *user.native_language();

        let sets = self.load_sets_via_cdn(&set_ids).await?;

        let mut result = ImportOnboardingResult {
            imported_set_ids: Vec::new(),
            created_vocabulary: 0,
            created_kanji: 0,
            created_grammar: 0,
            skipped_duplicates: 0,
            skipped_no_translation: 0,
        };

        let mut created_kanji_chars: HashSet<String> = HashSet::new();
        let mut grammar_levels: HashSet<JapaneseLevel> = HashSet::new();

        for (set_id, set) in sets {
            debug!(set_id = %set_id, words_count = set.words().len(), "Processing set");

            let set_level = *set.level();
            // Grammar is keyed by JLPT level, so any set contributes its own
            // level — previously gated to "Jlpt"-prefixed ids only, which left
            // Minna / Irodori / Duolingo without grammar.
            grammar_levels.insert(set_level);

            let words_result = VocabularyCard::from_text(&set.words().join(" "), &native_language);

            result.skipped_no_translation += words_result.skipped_no_translation.len();

            for vocab_card in words_result.cards {
                if let Ok(_study_card) = self.create_vocabulary_card(
                    &mut user,
                    vocab_card,
                    &mut result.skipped_duplicates,
                ) {
                    result.created_vocabulary += 1;
                }
            }

            result.imported_set_ids.push(set_id);
        }

        // Pull in every level at or below the target so a user picking N3 also
        // gets N5/N4/N2/... grammar rules and kanji they should already know,
        // regardless of which sets they imported.
        for level in JapaneseLevel::ALL {
            if level <= target_level {
                grammar_levels.insert(level);
            }
        }

        debug!(levels = ?grammar_levels, "Importing grammar rules for onboarding levels");

        for level in &grammar_levels {
            let grammar_rules = get_rules_by_level(level);
            for rule in grammar_rules {
                if let Ok(grammar_card) = GrammarRuleCard::new(*rule.rule_id()) {
                    let card = Card::Grammar(grammar_card);
                    match user.create_card(card) {
                        Ok(_) => {
                            result.created_grammar += 1;
                        },
                        Err(OrigaError::DuplicateCard { .. }) => {
                            result.skipped_duplicates += 1;
                        },
                        Err(e) => {
                            warn!(error = ?e, "Failed to create grammar card");
                        },
                    }
                }
            }
        }

        // Import kanji directly from the kanji dictionary for every level ≤
        // target — symmetric with grammar above. Previously kanji were
        // extracted FROM vocabulary words, which produced wrong results: a
        // word at N4 might contain an N3 kanji, so the user got kanji they
        // never asked for. Kanji→vocab companion cards are still created for
        // each kanji (create_companion_vocab_cards).
        debug!(target_level = ?target_level, "Importing kanji from dictionary");
        for level in JapaneseLevel::ALL {
            if level > target_level {
                continue;
            }
            for info in get_kanji_list(&level) {
                let kanji_char = info.kanji().to_string();
                if created_kanji_chars.contains(&kanji_char) {
                    continue;
                }
                if self
                    .create_kanji_card(&mut user, &kanji_char, &mut result)
                    .is_ok()
                {
                    result.created_kanji += 1;
                    created_kanji_chars.insert(kanji_char);
                }
            }
        }

        user.mark_sets_as_imported(set_ids);
        self.repository.save_sync(&user).await?;

        info!(
            user_id = %user.id(),
            vocabulary = result.created_vocabulary,
            kanji = result.created_kanji,
            grammar = result.created_grammar,
            duplicates = result.skipped_duplicates,
            "Onboarding sets import completed"
        );

        Ok(result)
    }

    async fn load_sets_via_cdn(
        &self,
        set_ids: &[String],
    ) -> Result<Vec<(String, WellKnownSet)>, OrigaError> {
        #[derive(serde::Deserialize)]
        struct SetData {
            level: JapaneseLevel,
            words: Vec<String>,
        }

        let mut results = Vec::with_capacity(set_ids.len());
        for id in set_ids {
            let path = resolve_set_path(id);
            let json = self.cdn.fetch_text(&path).await?;

            let data: SetData =
                serde_json::from_str(&json).map_err(|e| OrigaError::WellKnownSetParseError {
                    reason: format!("Error parsing {}: {}", id, e),
                })?;

            results.push((id.clone(), WellKnownSet::new(data.level, data.words)));
        }
        Ok(results)
    }

    fn create_vocabulary_card(
        &self,
        user: &mut crate::domain::User,
        vocab_card: VocabularyCard,
        skipped_duplicates: &mut usize,
    ) -> Result<StudyCard, OrigaError> {
        let card = Card::Vocabulary(vocab_card);
        match user.create_card(card) {
            Ok(study_card) => {
                debug!(word = ?study_card.card().question(&crate::domain::NativeLanguage::Russian), "Vocabulary card created");
                Ok(study_card)
            },
            Err(OrigaError::DuplicateCard { question }) => {
                warn!(word = %question, "Duplicate vocabulary card, skipping");
                *skipped_duplicates += 1;
                Err(OrigaError::DuplicateCard { question })
            },
            Err(e) => Err(e),
        }
    }

    fn create_kanji_card(
        &self,
        user: &mut crate::domain::User,
        kanji_char: &str,
        result: &mut ImportOnboardingResult,
    ) -> Result<StudyCard, OrigaError> {
        match KanjiCard::new(kanji_char.to_string()) {
            Ok(kanji_card) => {
                let card = Card::Kanji(kanji_card);
                match user.create_card(card) {
                    Ok(study_card) => {
                        debug!(kanji = %kanji_char, "Kanji card created");

                        let companions = user.create_companion_vocab_cards(kanji_char);
                        result.created_vocabulary += companions.len();
                        if !companions.is_empty() {
                            debug!(kanji = %kanji_char, companions = companions.len(), "Companion vocab cards created during onboarding");
                        }

                        Ok(study_card)
                    },
                    Err(OrigaError::DuplicateCard { question }) => {
                        warn!(kanji = %question, "Duplicate kanji card, skipping");
                        result.skipped_duplicates += 1;
                        Err(OrigaError::DuplicateCard { question })
                    },
                    Err(e) => Err(e),
                }
            },
            Err(e) => {
                warn!(kanji = %kanji_char, error = ?e, "Failed to create kanji card");
                Err(e)
            },
        }
    }
}
