#[path = "mod.rs"]
mod tests;

use jeels_cli::application::use_cases::{CreateCardUseCase, DeleteCardUseCase};
use jeels_cli::application::user_repository::UserRepository;
use jeels_cli::infrastructure::EmbeddingGenerator;
use tests::*;

#[tokio::test]
async fn delete_card_use_case_should_remove_card_from_database() {
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

    let delete_use_case = DeleteCardUseCase::new(&ctx.repository);

    // Act
    delete_use_case.execute(user.id(), card.id()).await.unwrap();

    // Assert
    let loaded_user = ctx.repository.find_by_id(user.id()).await.unwrap().unwrap();
    assert!(loaded_user.get_card(card.id()).is_none());
}
