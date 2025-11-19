#[path = "mod.rs"]
mod tests;

use jeels_cli::{
    application::use_cases::{CreateCardUseCase, GetHintsUseCase},
    settings::ApplicationEnvironment,
};
use tests::*;

#[tokio::test]
async fn get_hints_use_case_should_return_similar_cards() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_generator().await.unwrap();
    let llm_service = settings.get_llm_service().await.unwrap();
    let create_use_case = CreateCardUseCase::new(repository, embedding_generator, llm_service);

    let card1 = create_use_case
        .execute(
            user.id(),
            "What is Rust?".to_string(),
            Some("A systems programming language".to_string()),
        )
        .await
        .unwrap();

    create_use_case
        .execute(
            user.id(),
            "What is Python?".to_string(),
            Some("A high-level programming language".to_string()),
        )
        .await
        .unwrap();

    create_use_case
        .execute(
            user.id(),
            "What is JavaScript?".to_string(),
            Some("A scripting language".to_string()),
        )
        .await
        .unwrap();

    let get_hints_use_case = GetHintsUseCase::new(repository);

    // Act
    let hints = get_hints_use_case
        .execute(user.id(), card1.id(), 2)
        .await
        .unwrap();

    // Assert
    assert!(!hints.is_empty());
    assert!(hints.len() <= 2);
    // The query card should not be in the results
    assert!(hints.iter().all(|hint| hint.card.id() != card1.id()));
    // Results should be sorted by similarity (highest first)
    for i in 0..hints.len().saturating_sub(1) {
        assert!(hints[i].similarity_score >= hints[i + 1].similarity_score);
    }
}

#[tokio::test]
async fn get_hints_use_case_should_return_empty_when_no_other_cards() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_generator().await.unwrap();
    let llm_service = settings.get_llm_service().await.unwrap();
    let create_use_case = CreateCardUseCase::new(repository, embedding_generator, llm_service);

    let card = create_use_case
        .execute(
            user.id(),
            "What is Rust?".to_string(),
            Some("A systems programming language".to_string()),
        )
        .await
        .unwrap();

    let get_hints_use_case = GetHintsUseCase::new(repository);

    // Act
    let hints = get_hints_use_case
        .execute(user.id(), card.id(), 5)
        .await
        .unwrap();

    // Assert
    assert!(hints.is_empty());
}

#[tokio::test]
async fn get_hints_use_case_should_respect_limit() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_generator().await.unwrap();
    let llm_service = settings.get_llm_service().await.unwrap();
    let create_use_case = CreateCardUseCase::new(repository, embedding_generator, llm_service);

    let query_card = create_use_case
        .execute(
            user.id(),
            "What is Rust?".to_string(),
            Some("A systems programming language".to_string()),
        )
        .await
        .unwrap();

    // Create multiple cards with distinct questions
    let questions = vec![
        "What is Python?",
        "What is JavaScript?",
        "What is Java?",
        "What is C++?",
        "What is Go?",
    ];

    for question in questions {
        create_use_case
            .execute(
                user.id(),
                question.to_string(),
                Some("Some answer".to_string()),
            )
            .await
            .unwrap();
    }

    let get_hints_use_case = GetHintsUseCase::new(repository);

    // Act
    let hints = get_hints_use_case
        .execute(user.id(), query_card.id(), 3)
        .await
        .unwrap();

    // Assert
    assert!(hints.len() <= 3);
}
