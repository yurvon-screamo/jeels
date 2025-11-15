#[path = "mod.rs"]
mod tests;

use jeels_cli::application::use_cases::{CreateCardUseCase, GetHintsUseCase};
use jeels_cli::infrastructure::EmbeddingGenerator;
use tests::*;

#[tokio::test]
async fn get_hints_use_case_should_return_similar_cards() {
    // Arrange
    let ctx = create_test_repository().await;
    let user = create_test_user(&ctx.repository).await;
    let mut embedding_generator = EmbeddingGenerator::new().unwrap();
    let create_use_case = CreateCardUseCase::new(&ctx.repository);

    let card1 = create_use_case
        .execute(
            &mut embedding_generator,
            user.id(),
            "What is Rust?".to_string(),
            "A systems programming language".to_string(),
        )
        .await
        .unwrap();

    let mut embedding_generator = EmbeddingGenerator::new().unwrap();
    let create_use_case = CreateCardUseCase::new(&ctx.repository);
    create_use_case
        .execute(
            &mut embedding_generator,
            user.id(),
            "What is Python?".to_string(),
            "A high-level programming language".to_string(),
        )
        .await
        .unwrap();

    let mut embedding_generator = EmbeddingGenerator::new().unwrap();
    let create_use_case = CreateCardUseCase::new(&ctx.repository);
    create_use_case
        .execute(
            &mut embedding_generator,
            user.id(),
            "What is JavaScript?".to_string(),
            "A scripting language".to_string(),
        )
        .await
        .unwrap();

    let get_hints_use_case = GetHintsUseCase::new(&ctx.repository);

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
    let ctx = create_test_repository().await;
    let user = create_test_user(&ctx.repository).await;
    let mut embedding_generator = EmbeddingGenerator::new().unwrap();
    let create_use_case = CreateCardUseCase::new(&ctx.repository);

    let card = create_use_case
        .execute(
            &mut embedding_generator,
            user.id(),
            "What is Rust?".to_string(),
            "A systems programming language".to_string(),
        )
        .await
        .unwrap();

    let get_hints_use_case = GetHintsUseCase::new(&ctx.repository);

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
    let ctx = create_test_repository().await;
    let user = create_test_user(&ctx.repository).await;
    let mut embedding_generator = EmbeddingGenerator::new().unwrap();
    let create_use_case = CreateCardUseCase::new(&ctx.repository);

    let query_card = create_use_case
        .execute(
            &mut embedding_generator,
            user.id(),
            "What is Rust?".to_string(),
            "A systems programming language".to_string(),
        )
        .await
        .unwrap();

    // Create multiple cards
    for i in 1..=5 {
        let mut embedding_generator = EmbeddingGenerator::new().unwrap();
        let create_use_case = CreateCardUseCase::new(&ctx.repository);
        create_use_case
            .execute(
                &mut embedding_generator,
                user.id(),
                format!("Question {}", i),
                format!("Answer {}", i),
            )
            .await
            .unwrap();
    }

    let get_hints_use_case = GetHintsUseCase::new(&ctx.repository);

    // Act
    let hints = get_hints_use_case
        .execute(user.id(), query_card.id(), 3)
        .await
        .unwrap();

    // Assert
    assert!(hints.len() <= 3);
}
