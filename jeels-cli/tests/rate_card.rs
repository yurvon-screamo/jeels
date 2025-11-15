#[path = "mod.rs"]
mod tests;

use jeels_cli::application::use_cases::{CreateCardUseCase, RateCardUseCase};
use jeels_cli::application::user_repository::UserRepository;
use jeels_cli::domain::value_objects::Rating;
use jeels_cli::infrastructure::{EmbeddingGenerator, FsrsSrsService};
use tests::*;

#[tokio::test]
async fn rate_card_use_case_should_add_review_and_update_schedule() {
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

    let srs_service = FsrsSrsService::new().unwrap();
    let rate_use_case = RateCardUseCase::new(&ctx.repository, &srs_service);

    // Act
    rate_use_case
        .execute(user.id(), card.id(), Rating::Good)
        .await
        .unwrap();

    // Assert
    let loaded_user = ctx.repository.find_by_id(user.id()).await.unwrap().unwrap();
    let loaded_card = loaded_user.get_card(card.id()).unwrap();
    assert_eq!(loaded_card.reviews().len(), 1);
    assert_eq!(loaded_card.reviews()[0].rating(), Rating::Good);
    assert!(loaded_card.memory_state().is_some());
}
