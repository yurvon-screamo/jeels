#[path = "mod.rs"]
mod tests;

use jeels_cli::application::use_cases::{CreateCardUseCase, ViewCardUseCase};
use jeels_cli::infrastructure::EmbeddingGenerator;
use tests::*;

#[tokio::test]
async fn view_card_use_case_should_return_card_question_and_answer() {
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

    let view_use_case = ViewCardUseCase::new(&ctx.repository);

    // Act
    let (question, answer) = view_use_case.execute(user.id(), card.id()).await.unwrap();

    // Assert
    assert_eq!(question.text(), "What is Rust?");
    assert_eq!(answer.text(), "A systems programming language");
}
