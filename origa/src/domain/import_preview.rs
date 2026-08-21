//! Prediction of per-word import outcomes for the set-import preview.
//!
//! `CreateCardsFromAnalysisUseCase` classifies words through
//! [`VocabularyCard::from_text`] (tokenize → dictionary lemma → dictionary
//! entry) and then rejects duplicates by lemma. The preview must classify
//! through the same path: an earlier version compared raw set strings
//! against existing cards ([`User::is_word_known`]), which mispredicted
//! inflected forms and words without dictionary entries, so the summary
//! line never matched the import toast. The journey test in
//! `use_cases::tests::journeys/import_preview.rs` pins prediction ==
//! actual import outcome.
//!
//! Convergence guarantee (scope!): with the DEFAULT selection — every
//! importable word checked — `New` predicts the created bucket and
//! `AlreadyExists` + `DuplicateInSelection` predict the skipped bucket
//! exactly. A manual selection change can invalidate a relative
//! `DuplicateInSelection` label (unchecking the `New` occurrence of a
//! lemma makes its duplicate the word that will actually be created);
//! labels are not recomputed on toggle.

use std::collections::{HashMap, HashSet};

use crate::dictionary::vocabulary::get_translation;
use crate::domain::{User, VocabularyCard};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordImportOutcome {
    /// The import would create a card for this word's dictionary lemma.
    New,
    /// A card with this word's dictionary lemma already exists in the
    /// user's collection.
    AlreadyExists,
    /// Another form of the same lemma appeared earlier in this selection;
    /// the import would create the lemma card from the earlier word and
    /// skip this one as a duplicate.
    DuplicateInSelection,
    /// `from_text` produced no card — the word is not a dictionary word or
    /// has no dictionary entry. The import would fail this word.
    NoDictionaryEntry,
}

#[derive(Debug, Clone)]
pub struct WordImportPreview {
    pub word: String,
    pub outcome: WordImportOutcome,
    pub meaning: Option<String>,
}

/// Incremental variant of [`User::preview_word_imports`] for callers that
/// must yield to the browser between words (the WASM preview loop): the
/// seen-words / seen-lemmas state lives here and survives across slices.
pub struct WordImportClassifier<'a> {
    user: &'a User,
    seen_words: HashMap<String, WordImportPreview>,
    seen_lemmas: HashSet<String>,
}

impl<'a> WordImportClassifier<'a> {
    pub fn classify(&mut self, word: &str) -> WordImportPreview {
        let lang = self.user.native_language();

        if let Some(preview) = self.seen_words.get(word) {
            return preview.clone();
        }

        // The single source of truth for "can this word become a card":
        // the same from_text the import use case calls.
        let created = VocabularyCard::from_text(word, lang);
        let (outcome, lemma) = match created.cards.first() {
            None => (WordImportOutcome::NoDictionaryEntry, None),
            Some(card) => {
                let lemma = card.word().text().to_string();
                if !self.seen_lemmas.insert(lemma.clone()) {
                    (WordImportOutcome::DuplicateInSelection, Some(lemma))
                } else if self.user.is_word_known(&lemma).is_known {
                    (WordImportOutcome::AlreadyExists, Some(lemma))
                } else {
                    (WordImportOutcome::New, Some(lemma))
                }
            },
        };
        let meaning = get_translation(word, lang)
            .or_else(|| lemma.as_deref().and_then(|l| get_translation(l, lang)));
        let preview = WordImportPreview {
            word: word.to_string(),
            outcome,
            meaning,
        };
        self.seen_words.insert(word.to_string(), preview.clone());
        preview
    }
}

impl User {
    pub fn word_import_classifier(&self) -> WordImportClassifier<'_> {
        WordImportClassifier {
            user: self,
            seen_words: HashMap::new(),
            seen_lemmas: HashSet::new(),
        }
    }

    /// Classifies `words` the same way [`crate::use_cases::
    /// CreateCardsFromAnalysisUseCase`] will process them, so the preview
    /// summary can converge with the import result toast.
    ///
    /// The import receives a `HashSet` of selected words: an exact word
    /// repeated in the input is ONE selection entry processed once (its
    /// repeat gets the first occurrence's outcome, never a duplicate
    /// label), while two DIFFERENT forms of one lemma are two entries —
    /// the first is created, the second skipped as a duplicate.
    pub fn preview_word_imports(&self, words: &[String]) -> Vec<WordImportPreview> {
        let mut classifier = self.word_import_classifier();
        words.iter().map(|word| classifier.classify(word)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Card, NativeLanguage};

    fn user_with_card_for(word: &str) -> User {
        crate::use_cases::init_real_dictionaries();
        let mut user = User::new(
            "tester@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let card = Card::Vocabulary(
            VocabularyCard::from_known_word(word, &NativeLanguage::Russian)
                .expect("test word must have a dictionary entry"),
        );
        user.create_card(card).expect("card creation must succeed");
        user
    }

    fn user_without_cards() -> User {
        crate::use_cases::init_real_dictionaries();
        User::new(
            "tester@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        )
    }

    #[test]
    fn inflected_form_with_existing_lemma_card_is_already_exists() {
        let user = user_with_card_for("食べる");

        let previews = user.preview_word_imports(&["食べました".to_string()]);

        assert_eq!(previews[0].outcome, WordImportOutcome::AlreadyExists);
    }

    #[test]
    fn dictionary_word_without_existing_card_is_new() {
        let user = user_without_cards();

        let previews = user.preview_word_imports(&["ねこ".to_string()]);

        assert_eq!(previews[0].outcome, WordImportOutcome::New);
    }

    #[test]
    fn non_vocabulary_word_is_no_dictionary_entry() {
        let user = user_without_cards();

        let previews = user.preview_word_imports(&["は".to_string()]);

        assert_eq!(previews[0].outcome, WordImportOutcome::NoDictionaryEntry);
    }

    #[test]
    fn second_form_of_same_new_lemma_is_duplicate_in_selection() {
        let user = user_without_cards();

        let previews = user.preview_word_imports(&["読みます".to_string(), "読む".to_string()]);

        assert_eq!(previews[0].outcome, WordImportOutcome::New);
        assert_eq!(previews[1].outcome, WordImportOutcome::DuplicateInSelection);
    }

    #[test]
    fn exact_word_repeat_repeats_the_first_outcome_instead_of_duplicate() {
        let user = user_without_cards();

        let previews = user.preview_word_imports(&["ねこ".to_string(), "ねこ".to_string()]);

        assert_eq!(previews[0].outcome, WordImportOutcome::New);
        assert_eq!(
            previews[1].outcome,
            WordImportOutcome::New,
            "the import processes one HashSet entry for the repeated word"
        );
    }

    #[test]
    fn repeated_word_keeps_the_lemma_meaning_fallback() {
        let user = user_without_cards();

        let previews = user.preview_word_imports(&["読みます".to_string(), "読みます".to_string()]);

        assert!(previews[1].meaning.is_some());
        assert_eq!(
            previews[0].meaning, previews[1].meaning,
            "a repeat must return the same cached preview, not lose the meaning"
        );
    }

    #[test]
    fn multi_token_word_is_classified_by_its_first_card_like_the_import() {
        let user = user_without_cards();

        let previews = user.preview_word_imports(&["本を読む".to_string()]);

        assert_eq!(previews[0].outcome, WordImportOutcome::New);
    }

    #[test]
    fn meaning_falls_back_to_the_lemma_translation_for_inflected_forms() {
        let user = user_without_cards();

        let previews = user.preview_word_imports(&["読みます".to_string()]);

        assert!(
            previews[0].meaning.is_some(),
            "an inflected form must still show a meaning via its lemma"
        );
    }
}
