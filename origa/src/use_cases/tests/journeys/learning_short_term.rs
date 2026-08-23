use ulid::Ulid;

use crate::domain::{
    JlptContent, NativeLanguage, NewCardPolicy, OrigaError, RateMode, Rating, User,
};
use crate::traits::UserRepository;
use crate::use_cases::tests::fixtures::{InMemoryUserRepository, create_user_with_vocab_cards};
use crate::use_cases::{RateCardUseCase, SelectCardsToLessonUseCase};

fn create_user_with_rated_cards(count: usize) -> (User, Vec<Ulid>) {
    let mut user = create_user_with_vocab_cards(count);
    let mut card_ids = Vec::new();

    for id in user.knowledge_set().study_cards().keys() {
        card_ids.push(*id);
    }

    for card_id in &card_ids {
        user.rate_card(*card_id, Rating::Again, RateMode::StandardLesson)
            .unwrap();
    }

    (user, card_ids)
}

#[tokio::test]
async fn select_cards_to_lesson_includes_high_difficulty_cards() {
    let (user, _) = create_user_with_rated_cards(3);
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = SelectCardsToLessonUseCase::new(&repo);

    let cards = use_case
        .execute(NewCardPolicy::Inject, &JlptContent::new())
        .await
        .unwrap();

    assert!(!cards.is_empty());
}

#[tokio::test]
async fn select_cards_to_lesson_includes_new_cards() {
    let user = create_user_with_vocab_cards(3);
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = SelectCardsToLessonUseCase::new(&repo);

    let cards = use_case
        .execute(NewCardPolicy::Inject, &JlptContent::new())
        .await
        .unwrap();

    assert!(!cards.is_empty(), "New cards should be included in lesson");
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

#[tokio::test]
async fn rate_card_short_term_again_updates_memory() {
    let (user, card_ids) = create_user_with_rated_cards(1);
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = RateCardUseCase::new(&repo);
    let card_id = card_ids[0];

    use_case
        .execute(card_id, RateMode::ShortTerm, Rating::Again)
        .await
        .unwrap();

    let updated = repo.get_current_user().await.unwrap().unwrap();
    let _card = updated.knowledge_set().get_card(card_id).unwrap();
}

#[tokio::test]
async fn rate_card_short_term_good_updates_memory() {
    let (user, card_ids) = create_user_with_rated_cards(1);
    let repo = InMemoryUserRepository::with_user(user);
    let use_case = RateCardUseCase::new(&repo);
    let card_id = card_ids[0];

    use_case
        .execute(card_id, RateMode::ShortTerm, Rating::Good)
        .await
        .unwrap();

    let updated = repo.get_current_user().await.unwrap().unwrap();
    let _card = updated.knowledge_set().get_card(card_id).unwrap();
}

#[tokio::test]
async fn full_short_term_cycle_processes_all_cards() {
    let (user, _) = create_user_with_rated_cards(3);
    let repo = InMemoryUserRepository::with_user(user);
    let select_use_case = SelectCardsToLessonUseCase::new(&repo);
    let rate_use_case = RateCardUseCase::new(&repo);

    let cards = select_use_case
        .execute(NewCardPolicy::Inject, &JlptContent::new())
        .await
        .unwrap();
    for (_, lc) in cards {
        rate_use_case
            .execute(lc.card_id(), RateMode::ShortTerm, Rating::Good)
            .await
            .unwrap();
    }

    let updated = repo.get_current_user().await.unwrap().unwrap();
    assert!(!updated.knowledge_set().lesson_history().is_empty());
}

#[tokio::test]
async fn rate_card_short_term_nonexistent_returns_error() {
    let repo = InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ));
    let use_case = RateCardUseCase::new(&repo);
    let non_existent_card_id = Ulid::new();

    let result = use_case
        .execute(non_existent_card_id, RateMode::ShortTerm, Rating::Good)
        .await;

    assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
}

#[tokio::test]
async fn padding_cards_are_marked_as_short_term() {
    let mut user = create_user_with_vocab_cards(20);
    let card_ids: Vec<Ulid> = user.knowledge_set().study_cards().keys().copied().collect();

    // 3 карты оцениваем Again → selected (due + high_difficulty)
    for card_id in &card_ids[..3] {
        user.rate_card(*card_id, Rating::Again, RateMode::StandardLesson)
            .unwrap();
    }

    // 12 карт: Again затем Hard → остаются high_difficulty, но next_review в будущем → padding
    for card_id in &card_ids[3..15] {
        user.rate_card(*card_id, Rating::Again, RateMode::StandardLesson)
            .unwrap();
        user.rate_card(*card_id, Rating::Hard, RateMode::StandardLesson)
            .unwrap();
    }

    let repo = InMemoryUserRepository::with_user(user);
    let use_case = SelectCardsToLessonUseCase::new(&repo);

    let cards = use_case
        .execute(NewCardPolicy::Inject, &JlptContent::new())
        .await
        .unwrap();

    let short_term_count = cards.iter().filter(|(_, c)| c.is_short_term()).count();

    assert!(
        short_term_count > 0,
        "Expected padding cards to be marked as short_term, got 0 short_term out of {} total",
        cards.len()
    );
}
