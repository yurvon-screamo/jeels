use rstest::rstest;
use ulid::Ulid;

use crate::domain::{
    JlptContent, NativeLanguage, NewCardPolicy, OrigaError, RateMode, Rating, User,
};
use crate::traits::UserRepository;
use crate::use_cases::tests::fixtures::{InMemoryUserRepository, create_user_with_vocab_cards};
use crate::use_cases::{CreateGrammarCardUseCase, RateCardUseCase, SelectCardsToLessonUseCase};

#[tokio::test]
async fn select_cards_to_lesson_returns_cards() {
    let user = create_user_with_vocab_cards(5);
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = SelectCardsToLessonUseCase::new(&repo);

    let cards = use_case
        .execute(NewCardPolicy::Inject, &JlptContent::new())
        .await
        .unwrap();

    assert!(!cards.is_empty());
}

#[tokio::test]
async fn select_cards_to_lesson_returns_empty_for_empty_knowledge_set() {
    let repo = InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ));
    let use_case = SelectCardsToLessonUseCase::new(&repo);

    let cards = use_case
        .execute(NewCardPolicy::Inject, &JlptContent::new())
        .await
        .unwrap();

    assert!(cards.is_empty());
}

#[rstest]
#[case(Rating::Again)]
#[case(Rating::Hard)]
#[case(Rating::Good)]
#[case(Rating::Easy)]
#[tokio::test]
async fn rate_card_updates_memory(#[case] rating: Rating) {
    let user = create_user_with_vocab_cards(1);
    let repo = InMemoryUserRepository::with_user(user);
    let user = repo.get_current_user().await.unwrap().unwrap();
    let card_id = *user.knowledge_set().study_cards().keys().next().unwrap();
    let use_case = RateCardUseCase::new(&repo);

    use_case
        .execute(card_id, RateMode::StandardLesson, rating)
        .await
        .unwrap();

    let updated = repo.get_current_user().await.unwrap().unwrap();
    let card = updated.knowledge_set().get_card(card_id).unwrap();
    assert!(!card.memory().is_new());
}

#[tokio::test]
async fn full_lesson_cycle_updates_history() {
    let user = create_user_with_vocab_cards(3);
    let repo = InMemoryUserRepository::with_user(user);

    let select_use_case = SelectCardsToLessonUseCase::new(&repo);
    let rate_use_case = RateCardUseCase::new(&repo);

    let cards = select_use_case
        .execute(NewCardPolicy::Inject, &JlptContent::new())
        .await
        .unwrap();
    for (_, lc) in cards {
        rate_use_case
            .execute(lc.card_id(), RateMode::StandardLesson, Rating::Good)
            .await
            .unwrap();
    }

    let updated = repo.get_current_user().await.unwrap().unwrap();
    assert!(!updated.knowledge_set().lesson_history().is_empty());
}

#[tokio::test]
async fn rate_card_nonexistent_returns_error() {
    let repo = InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ));
    let use_case = RateCardUseCase::new(&repo);
    let non_existent_card_id = Ulid::new();

    let result = use_case
        .execute(non_existent_card_id, RateMode::StandardLesson, Rating::Good)
        .await;

    assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
}

#[tokio::test]
async fn rate_card_with_short_term_mode_updates_memory() {
    let user = create_user_with_vocab_cards(1);
    let repo = InMemoryUserRepository::with_user(user);
    let user = repo.get_current_user().await.unwrap().unwrap();
    let card_id = *user.knowledge_set().study_cards().keys().next().unwrap();
    let use_case = RateCardUseCase::new(&repo);

    use_case
        .execute(card_id, RateMode::ShortTerm, Rating::Good)
        .await
        .unwrap();

    let updated = repo.get_current_user().await.unwrap().unwrap();
    let card = updated.knowledge_set().get_card(card_id).unwrap();
    assert!(!card.memory().is_new());
}

#[rstest]
#[case::again(Rating::Again)]
#[case::hard(Rating::Hard)]
#[case::good(Rating::Good)]
#[case::easy(Rating::Easy)]
#[tokio::test]
async fn rate_card_short_term_mode_all_ratings(#[case] rating: Rating) {
    let user = create_user_with_vocab_cards(1);
    let repo = InMemoryUserRepository::with_user(user);
    let user = repo.get_current_user().await.unwrap().unwrap();
    let card_id = *user.knowledge_set().study_cards().keys().next().unwrap();
    let use_case = RateCardUseCase::new(&repo);

    let result = use_case.execute(card_id, RateMode::ShortTerm, rating).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn rate_card_twice_in_short_term_mode_updates_state() {
    let user = create_user_with_vocab_cards(1);
    let repo = InMemoryUserRepository::with_user(user);
    let user = repo.get_current_user().await.unwrap().unwrap();
    let card_id = *user.knowledge_set().study_cards().keys().next().unwrap();
    let use_case = RateCardUseCase::new(&repo);

    use_case
        .execute(card_id, RateMode::ShortTerm, Rating::Good)
        .await
        .unwrap();
    use_case
        .execute(card_id, RateMode::ShortTerm, Rating::Easy)
        .await
        .unwrap();

    let updated = repo.get_current_user().await.unwrap().unwrap();
    let card = updated.knowledge_set().get_card(card_id).unwrap();
    assert!(!card.memory().is_new());
}

#[rstest]
#[case(Rating::Again)]
#[case(Rating::Hard)]
#[case(Rating::Good)]
#[case(Rating::Easy)]
#[tokio::test]
async fn rate_card_and_create_and_rate_grammar_card_dual_rating(#[case] rating: Rating) {
    crate::use_cases::init_real_dictionaries();

    let user = create_user_with_vocab_cards(1);
    let repo = InMemoryUserRepository::with_user(user);

    let user = repo.get_current_user().await.unwrap().unwrap();
    let vocab_card_id = *user
        .knowledge_set()
        .study_cards()
        .keys()
        .next()
        .expect("No vocab card found");

    let rule_id = Ulid::from_string("01KJ9AVWBGC2BT0DMFPDYYFEWB").expect("Invalid ULID");

    let rate_use_case = RateCardUseCase::new(&repo);
    rate_use_case
        .execute(vocab_card_id, RateMode::StandardLesson, rating)
        .await
        .unwrap();

    let create_grammar_use_case = CreateGrammarCardUseCase::new(&repo);
    let grammar_cards = create_grammar_use_case
        .execute(vec![rule_id])
        .await
        .expect("Failed to create grammar card");

    assert_eq!(grammar_cards.len(), 1);
    let grammar_card_id = *grammar_cards
        .first()
        .expect("No grammar card created")
        .card_id();

    rate_use_case
        .execute(grammar_card_id, RateMode::StandardLesson, rating)
        .await
        .unwrap();

    let updated = repo.get_current_user().await.unwrap().unwrap();

    let vocab_card = updated
        .knowledge_set()
        .get_card(vocab_card_id)
        .expect("Vocab card not found");
    assert!(!vocab_card.memory().is_new());

    let grammar_card = updated
        .knowledge_set()
        .get_card(grammar_card_id)
        .expect("Grammar card not found");
    assert!(!grammar_card.memory().is_new());

    assert_ne!(vocab_card_id, grammar_card_id);
}
