#[path = "mod.rs"]
mod tests;

use jeels_cli::application::use_cases::{CreateCardUseCase, StartStudySessionUseCase};
use jeels_cli::infrastructure::EmbeddingGenerator;
use tests::*;

#[tokio::test]
async fn start_study_session_use_case_should_return_due_cards() {
    // Arrange
    let ctx = create_test_repository().await;
    let user = create_test_user(&ctx.repository).await;
    let mut embedding_generator = EmbeddingGenerator::new().unwrap();
    let create_use_case = CreateCardUseCase::new(&ctx.repository);
    create_use_case
        .execute(
            &mut embedding_generator,
            user.id(),
            "What is Rust?".to_string(),
            "A systems programming language".to_string(),
        )
        .await
        .unwrap();

    let start_session_use_case = StartStudySessionUseCase::new(&ctx.repository);

    // Act
    let cards = start_session_use_case.execute(user.id()).await.unwrap();

    // Assert
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0].question().text(), "What is Rust?");
}
