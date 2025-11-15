#[path = "mod.rs"]
mod tests;

use jeels_cli::application::use_cases::{CreateCardUseCase, EditCardUseCase};
use jeels_cli::application::user_repository::UserRepository;
use jeels_cli::infrastructure::EmbeddingGenerator;
use tests::*;

#[tokio::test]
async fn edit_card_use_case_should_update_card_in_database() {
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

    let mut embedding_generator = EmbeddingGenerator::new().unwrap();
    let edit_use_case = EditCardUseCase::new(&ctx.repository);

    // Act
    edit_use_case
        .execute(
            &mut embedding_generator,
            user.id(),
            card.id(),
            "What is Rust language?".to_string(),
            "A memory-safe systems programming language".to_string(),
        )
        .await
        .unwrap();

    // Assert
    let loaded_user = ctx.repository.find_by_id(user.id()).await.unwrap().unwrap();
    let loaded_card = loaded_user.get_card(card.id()).unwrap();
    assert_eq!(loaded_card.question().text(), "What is Rust language?");
    assert_eq!(
        loaded_card.answer().text(),
        "A memory-safe systems programming language"
    );
}
