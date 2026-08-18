use tracing::{info, warn};

use crate::dictionary::vocabulary::get_translation;
use crate::domain::{Card, NativeLanguage, OrigaError, Question, tokenize_text};
use crate::traits::UserRepository;

pub struct LemmaMigrationResult {
    pub vocab_count: usize,
    pub candidates: usize,
    pub migrated: usize,
}

/// Migrates vocabulary cards whose word is no longer a single lemma of the
/// current tokenizer (the SudachiDict migration changed lemma spellings:
/// 信ずる → 信じる, 現われる → 現れる, アイディア → アイデア …).
///
/// For each vocabulary card:
/// 1. Skip if the word still tokenizes to exactly one token equal to itself
///    (fast path — the overwhelming majority).
/// 2. Re-tokenize; if the text collapses to exactly ONE vocabulary lemma that
///    has a translation in the dictionary, rename the card to that lemma.
///    FSRS memory state is preserved — only the word text changes.
/// 3. Anything else (multi-token splits like 二十 → 二|十, words that vanished
///    from the dictionary) is left untouched: the card keeps working, it just
///    stops matching new well-known sets.
#[derive(Clone)]
pub struct MigrateVocabularyLemmasUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> MigrateVocabularyLemmasUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<LemmaMigrationResult, OrigaError> {
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        // The translation dictionary may not be loaded yet (this runs at app
        // start); translations are required for the rename check, so bail out
        // early — the next launch retries.
        if !crate::dictionary::vocabulary::is_vocabulary_loaded() {
            return Ok(LemmaMigrationResult {
                vocab_count: 0,
                candidates: 0,
                migrated: 0,
            });
        }

        let vocab: Vec<(ulid::Ulid, String)> = user
            .knowledge_set()
            .study_cards()
            .iter()
            .filter_map(|(id, sc)| match sc.card() {
                Card::Vocabulary(v) => Some((*id, v.word().text().to_string())),
                _ => None,
            })
            .collect();

        let vocab_count = vocab.len();
        let mut candidates = 0usize;
        let mut migrated = 0usize;

        for (card_id, word_text) in vocab {
            match new_lemma(&word_text) {
                Ok(Some(new_word)) if new_word != word_text => {
                    candidates += 1;
                    let Some(sc) = user.knowledge_set().get_card(card_id) else {
                        continue;
                    };
                    let Card::Vocabulary(vocab_card) = sc.card() else {
                        continue;
                    };
                    let updated =
                        vocab_card
                            .clone()
                            .with_word(Question::new(new_word.clone()).map_err(|e| {
                                OrigaError::InvalidQuestion {
                                    reason: e.to_string(),
                                }
                            })?);
                    match user.update_card_content(card_id, Card::Vocabulary(updated)) {
                        Ok(()) => {
                            migrated += 1;
                            info!(
                                card_id = %card_id,
                                from = %word_text,
                                to = %new_word,
                                "Migrated vocabulary lemma"
                            );
                        },
                        Err(e) => warn!(
                            card_id = %card_id,
                            from = %word_text,
                            to = %new_word,
                            error = ?e,
                            "Failed to migrate vocabulary lemma"
                        ),
                    }
                },
                Ok(_) => {},
                Err(e) => warn!(
                    card_id = %card_id,
                    word = %word_text,
                    error = ?e,
                    "Tokenization failed during lemma migration"
                ),
            }
        }

        if migrated > 0 {
            self.repository.save(&user).await?;
        }

        info!(
            vocab_count,
            candidates, migrated, "Vocabulary lemma migration complete"
        );

        Ok(LemmaMigrationResult {
            vocab_count,
            candidates,
            migrated,
        })
    }
}

/// Returns the single new lemma for `word`, if the word needs renaming.
fn new_lemma(word: &str) -> Result<Option<String>, OrigaError> {
    let tokens = tokenize_text(word)?;
    if tokens.len() == 1
        && tokens[0].orthographic_base_form() == word
        && tokens[0].orthographic_surface_form() == word
    {
        // Still a dictionary lemma — nothing to do.
        return Ok(None);
    }
    let lemma = tokens
        .iter()
        .filter(|t| t.part_of_speech().is_vocabulary_word())
        .map(|t| t.orthographic_base_form().to_string())
        .collect::<Vec<_>>();
    if lemma.len() != 1 {
        // Multi-token split (二十 → 二|十) or nothing salvageable —
        // leave the card as is.
        return Ok(None);
    }
    let new_word = lemma[0].clone();
    // The renamed card must keep having a translation, otherwise the card
    // becomes unanswerable.
    if get_translation(&new_word, &NativeLanguage::Russian).is_none()
        && get_translation(&new_word, &NativeLanguage::English).is_none()
    {
        return Ok(None);
    }
    Ok(Some(new_word))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_lemma_word_returns_none() {
        crate::use_cases::tests::fixtures::init_real_dictionaries();
        assert_eq!(new_lemma("自動車").unwrap(), None);
    }

    #[test]
    fn particle_only_word_returns_none() {
        crate::use_cases::tests::fixtures::init_real_dictionaries();
        // は alone is not a vocabulary word — nothing to rename to.
        assert_eq!(new_lemma("は").unwrap(), None);
    }

    #[test]
    fn zuuru_verb_migrates_to_jiru_lemma() {
        crate::use_cases::tests::fixtures::init_real_dictionaries();
        // Old UniDic lemma spelling; SudachiDict collapses it to 信じる.
        assert_eq!(new_lemma("信ずる").unwrap().as_deref(), Some("信じる"));
    }

    #[test]
    fn numeral_composite_is_left_alone() {
        crate::use_cases::tests::fixtures::init_real_dictionaries();
        // 二十 splits into 二|十 — a multi-token split must not be renamed.
        assert_eq!(new_lemma("二十").unwrap(), None);
    }

    #[tokio::test]
    async fn migrates_card_word_in_place() {
        crate::use_cases::tests::fixtures::init_real_dictionaries();

        use crate::domain::{Card, VocabularyCard};

        let mut user = crate::domain::User::new(
            "test@example.com".to_string(),
            crate::domain::NativeLanguage::Russian,
            None,
        );
        let vocab =
            VocabularyCard::from_known_word("信ずる", &crate::domain::NativeLanguage::Russian)
                .expect("card with translation must be constructible");
        user.create_card(Card::Vocabulary(vocab)).expect("card must be created");
        let created_id = *user
            .knowledge_set()
            .study_cards()
            .keys()
            .next()
            .expect("one card");

        let repo = crate::use_cases::tests::fixtures::InMemoryUserRepository::with_user(user);
        let uc = MigrateVocabularyLemmasUseCase::new(&repo);
        let result = uc.execute().await.expect("migration must succeed");

        assert_eq!(result.migrated, 1);
        let saved = repo.get_current_user().await.unwrap().unwrap();
        let sc = saved.knowledge_set().get_card(created_id).unwrap();
        match sc.card() {
            Card::Vocabulary(v) => assert_eq!(v.word().text(), "信じる"),
            other => panic!("expected vocabulary card, got {other:?}"),
        }
    }
}
