use ulid::Ulid;

use crate::domain::{Card, NativeLanguage, User};
use crate::traits::UserRepository;
use crate::use_cases::CompleteOnboardingScoringUseCase;
use crate::use_cases::tests::fixtures::{
    InMemoryUserRepository, create_test_vocab_card, init_phrase_index_from_cdn,
    init_real_dictionaries,
};

fn user_with_skipped_cards(skipped: &[Ulid]) -> User {
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    for id in skipped {
        user.mark_card_skipped_in_onboarding(*id);
    }
    user
}

#[tokio::test]
async fn no_current_user_returns_current_user_not_exist() {
    // Arrange
    let repo = InMemoryUserRepository::new();
    let use_case = CompleteOnboardingScoringUseCase::new(&repo);

    // Act
    let result = use_case.execute().await;

    // Assert
    assert!(matches!(
        result,
        Err(crate::domain::OrigaError::CurrentUserNotExist)
    ));
}

#[tokio::test]
async fn complete_onboarding_clears_skipped_cards() {
    // Arrange
    let skipped = [Ulid::new(), Ulid::new()];
    let user = user_with_skipped_cards(&skipped);
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = CompleteOnboardingScoringUseCase::new(&repo);

    // Act
    use_case.execute().await.unwrap();

    // Assert
    let updated = repo.get_current_user().await.unwrap().unwrap();
    assert!(
        updated.onboarding_scoring_skipped().is_empty(),
        "skipped cards must be cleared once onboarding reaches terminal state"
    );
}

#[tokio::test]
async fn complete_onboarding_marks_completed() {
    // Arrange
    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = CompleteOnboardingScoringUseCase::new(&repo);

    // Act
    use_case.execute().await.unwrap();

    // Assert
    let updated = repo.get_current_user().await.unwrap().unwrap();
    assert!(
        updated.is_onboarding_completed(),
        "user must be marked as onboarding-completed for routing to /home"
    );
}

#[tokio::test]
async fn complete_onboarding_returns_zero_phrases_when_no_known_words() {
    // Arrange — phrases dictionary loaded but the user has no known cards, so
    // SeedReadyPhrasesUseCase has nothing to seed.
    init_real_dictionaries();
    init_phrase_index_from_cdn();

    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = CompleteOnboardingScoringUseCase::new(&repo);

    // Act
    let phrase_count = use_case.execute().await.unwrap();

    // Assert
    assert_eq!(
        phrase_count, 0,
        "no known vocabulary means no phrases can be seeded"
    );
    let updated = repo.get_current_user().await.unwrap().unwrap();
    assert!(
        !updated
            .knowledge_set()
            .study_cards()
            .values()
            .any(|sc| matches!(sc.card(), Card::Phrase(_))),
        "no phrase cards should have been created"
    );
}

#[tokio::test]
async fn complete_onboarding_persists_via_save_sync() {
    // Arrange — verify the cleared skipped set is observable after a fresh
    // repository read (i.e. save_sync was invoked, not just an in-memory
    // mutation).
    let skipped = [Ulid::new()];
    let user = user_with_skipped_cards(&skipped);
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = CompleteOnboardingScoringUseCase::new(&repo);

    // Act
    use_case.execute().await.unwrap();

    // Assert
    let reloaded = repo.get_current_user().await.unwrap().unwrap();
    assert!(reloaded.onboarding_scoring_skipped().is_empty());
    assert!(reloaded.is_onboarding_completed());
}

#[tokio::test]
async fn complete_onboarding_persists_completion_and_runs_seed_step() {
    // Arrange — the goal of this test is to verify the *integration* between
    // `CompleteOnboardingScoringUseCase` and `SeedReadyPhrasesUseCase`: the
    // use case must (a) mark the user as onboarding-completed, (b) clear the
    // skipped-cards record, and (c) actually invoke the seed step rather
    // than short-circuiting before it. We do NOT assert a non-zero phrase
    // count because the canned phrase fixture corpus may not contain a
    // phrase that this single known word unlocks; the seed step running at
    // all is the contract being verified (and the no-known-words case below
    // covers the "no phrases" branch).
    init_real_dictionaries();
    init_phrase_index_from_cdn();

    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    user.mark_card_skipped_in_onboarding(Ulid::new());
    let card = create_test_vocab_card("hello");
    let study_card = user.create_card(card).unwrap();
    user.knowledge_set_mut()
        .mark_card_as_known(*study_card.card_id())
        .unwrap();
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = CompleteOnboardingScoringUseCase::new(&repo);

    // Act
    let _phrase_count = use_case.execute().await.expect("execute must succeed");

    // Assert
    let updated = repo.get_current_user().await.unwrap().unwrap();
    assert!(
        updated.is_onboarding_completed(),
        "user must be marked onboarding-completed"
    );
    assert!(
        updated.onboarding_scoring_skipped().is_empty(),
        "skipped cards must be cleared on completion"
    );
    // known_vocab_hash is recomputed by SeedReadyPhrasesUseCase; a non-zero
    // value proves the seed step actually ran (it would stay at 0 otherwise).
    assert_ne!(
        updated.known_vocab_hash(),
        0,
        "known_vocab_hash must be populated by the seed step"
    );
}
