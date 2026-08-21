use crate::domain::{Card, NativeLanguage, User, VocabularyCard, WordImportOutcome};
use crate::traits::UserRepository;
use crate::use_cases::tests::fixtures::{InMemoryUserRepository, init_real_dictionaries};
use crate::use_cases::{CreateCardsFromAnalysisUseCase, WordToCreate};

/// Words covering every import path:
/// - `食べました` — inflected form whose lemma card already exists → skip,
/// - `ねこ` — new dictionary word → create,
/// - `は` — particle, no dictionary card → fail,
/// - `読みます` / `読む` — two forms of one new lemma → create + skip.
const WORDS: &[&str] = &["食べました", "ねこ", "は", "読みます", "読む"];

fn repo_with_lemma_card() -> InMemoryUserRepository {
    init_real_dictionaries();
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let card = Card::Vocabulary(
        VocabularyCard::from_known_word("食べる", &NativeLanguage::Russian)
            .expect("dictionary word must resolve"),
    );
    user.create_card(card).expect("card creation must succeed");
    InMemoryUserRepository::with_user(user)
}

fn preview_outcomes(user: &User) -> Vec<(String, WordImportOutcome)> {
    let words: Vec<String> = WORDS.iter().map(|w| w.to_string()).collect();
    user.preview_word_imports(&words)
        .into_iter()
        .map(|p| (p.word, p.outcome))
        .collect()
}

/// The convergence guarantee, scoped to the DEFAULT selection (all
/// importable words checked): `New` predicts the created bucket,
/// `AlreadyExists` + `DuplicateInSelection` predict the skipped bucket,
/// `NoDictionaryEntry` predicts the failed bucket. If either side changes
/// its classification path, this test fails — that regression is exactly
/// the mismatched-numbers bug the preview classification exists to fix.
#[tokio::test]
async fn preview_word_imports_default_selection_matches_import_result() {
    let repo = repo_with_lemma_card();
    let user = repo.get_current_user().await.unwrap().unwrap();
    let outcomes = preview_outcomes(&user);

    let use_case = CreateCardsFromAnalysisUseCase::new(&repo);
    let words: Vec<WordToCreate> = WORDS
        .iter()
        .map(|w| WordToCreate {
            base_form: w.to_string(),
        })
        .collect();
    let result = use_case.execute(words, None).await.unwrap();

    for (word, outcome) in &outcomes {
        let in_skipped = result.skipped_words.iter().any(|skipped| skipped == word);
        let in_failed = result.failed_words.iter().any(|(failed, _)| failed == word);

        // Skipped/failed buckets hold the original word strings, so they
        // are checkable per-word. The created bucket holds dictionary
        // lemmas (読みます creates 読む), so New words are verified by
        // exclusion here + the exact bucket-size equality below: every
        // word lands in exactly one bucket, which makes the partition
        // provably equal to the prediction.
        match outcome {
            WordImportOutcome::New => {
                assert!(
                    !in_skipped && !in_failed,
                    "`{word}` predicted New but landed in skipped/failed; skipped={:?} failed={:?}",
                    result.skipped_words,
                    result.failed_words
                );
            },
            WordImportOutcome::AlreadyExists | WordImportOutcome::DuplicateInSelection => {
                assert!(
                    in_skipped && !in_failed,
                    "`{word}` predicted {outcome:?} but was not skipped; failed={:?}",
                    result.failed_words
                );
            },
            WordImportOutcome::NoDictionaryEntry => {
                assert!(
                    in_failed && !in_skipped,
                    "`{word}` predicted NoDictionaryEntry but did not fail; skipped={:?}",
                    result.skipped_words
                );
            },
        }
    }

    // Bucket sizes must converge with the prediction counts as well.
    let predicted_new = outcomes
        .iter()
        .filter(|(_, o)| *o == WordImportOutcome::New)
        .count();
    let predicted_skipped = outcomes
        .iter()
        .filter(|(_, o)| {
            *o == WordImportOutcome::AlreadyExists || *o == WordImportOutcome::DuplicateInSelection
        })
        .count();
    let predicted_failed = outcomes
        .iter()
        .filter(|(_, o)| *o == WordImportOutcome::NoDictionaryEntry)
        .count();
    assert_eq!(result.created_cards.len(), predicted_new);
    assert_eq!(result.skipped_words.len(), predicted_skipped);
    assert_eq!(result.failed_words.len(), predicted_failed);
}

/// The user-facing summary counts unique words — the import processes a
/// `HashSet` of selected words, so a word listed in two sets is one
/// selection entry: it is NOT a duplicate-in-selection (nothing gets
/// skipped for it) and must not be counted twice as creatable.
#[tokio::test]
async fn preview_word_imports_counts_a_word_duplicated_across_sets_once() {
    let repo = repo_with_lemma_card();
    let user = repo.get_current_user().await.unwrap().unwrap();

    let words = vec!["ねこ".to_string(), "ねこ".to_string()];
    let outcomes = user.preview_word_imports(&words);

    assert_eq!(outcomes[0].outcome, WordImportOutcome::New);
    assert_eq!(
        outcomes[1].outcome,
        WordImportOutcome::New,
        "an exact repeat is the same selection entry — not a duplicate label"
    );

    // The import would process it once and skip nothing.
    let use_case = CreateCardsFromAnalysisUseCase::new(&repo);
    let result = use_case
        .execute(
            vec![WordToCreate {
                base_form: "ねこ".to_string(),
            }],
            None,
        )
        .await
        .unwrap();
    assert_eq!(result.created_cards.len(), 1);
    assert!(result.skipped_words.is_empty());
}
