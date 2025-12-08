#[path = "mod.rs"]
mod tests;

use keikaku::application::create_card::CardContent;
use keikaku::application::use_cases::{CreateCardUseCase, RateCardUseCase};
use keikaku::application::user_repository::UserRepository;
use keikaku::domain::value_objects::{Answer, Rating};
use keikaku::settings::ApplicationEnvironment;
use tests::*;

#[tokio::test]
async fn rate_card_use_case_should_add_review_and_update_schedule() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_service().await.unwrap();
    let llm_service = settings.get_llm_service().await.unwrap();
    let create_use_case = CreateCardUseCase::new(repository, embedding_generator, llm_service);
    let card = create_use_case
        .execute(
            user.id(),
            "What is Rust?".to_string(),
            Some(CardContent {
                answer: Answer::new("A systems programming language".to_string()).unwrap(),
                example_phrases: vec![],
            }),
        )
        .await
        .unwrap();

    let srs_service = settings.get_srs_service().await.unwrap();
    let rate_use_case = RateCardUseCase::new(repository, srs_service);

    // Act
    rate_use_case
        .execute(user.id(), card.id(), Rating::Good)
        .await
        .unwrap();

    // Assert
    let loaded_user = repository.find_by_id(user.id()).await.unwrap().unwrap();
    let loaded_card = loaded_user.get_card(card.id()).unwrap();
    assert_eq!(loaded_card.reviews().len(), 1);
    assert_eq!(loaded_card.reviews()[0].rating(), Rating::Good);
    assert!(loaded_card.difficulty().is_some());
    assert!(loaded_card.stability().is_some());
}

#[tokio::test]
async fn rate_card_use_case_should_use_reviews_for_memory_state_calculation() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_service().await.unwrap();
    let llm_service = settings.get_llm_service().await.unwrap();
    let create_use_case = CreateCardUseCase::new(repository, embedding_generator, llm_service);
    let card = create_use_case
        .execute(
            user.id(),
            "What is Rust?".to_string(),
            Some(CardContent {
                answer: Answer::new("A systems programming language".to_string()).unwrap(),
                example_phrases: vec![],
            }),
        )
        .await
        .unwrap();

    let srs_service = settings.get_srs_service().await.unwrap();
    let rate_use_case = RateCardUseCase::new(repository, srs_service);

    // Act - First review
    rate_use_case
        .execute(user.id(), card.id(), Rating::Good)
        .await
        .unwrap();

    // Second review - should use reviews history, not previous_state
    rate_use_case
        .execute(user.id(), card.id(), Rating::Easy)
        .await
        .unwrap();

    // Assert
    let loaded_user = repository.find_by_id(user.id()).await.unwrap().unwrap();
    let loaded_card = loaded_user.get_card(card.id()).unwrap();
    assert_eq!(loaded_card.reviews().len(), 2);
    assert_eq!(loaded_card.reviews()[0].rating(), Rating::Good);
    assert_eq!(loaded_card.reviews()[1].rating(), Rating::Easy);
    assert!(loaded_card.difficulty().is_some());
    assert!(loaded_card.stability().is_some());
    // Memory state should be updated based on both reviews
    assert!(loaded_card.next_review_date() > loaded_card.reviews()[1].timestamp());
}
