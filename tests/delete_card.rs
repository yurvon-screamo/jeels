#[path = "mod.rs"]
mod tests;

use keikaku::application::use_cases::{CreateCardUseCase, DeleteCardUseCase};
use keikaku::application::user_repository::UserRepository;
use keikaku::settings::ApplicationEnvironment;
use tests::*;

#[tokio::test]
async fn delete_card_use_case_should_remove_card_from_database() {
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
            Some(vec![]),
        )
        .await
        .unwrap();

    let delete_use_case = DeleteCardUseCase::new(repository);

    // Act
    delete_use_case.execute(user.id(), card.id()).await.unwrap();

    // Assert
    let loaded_user = repository.find_by_id(user.id()).await.unwrap().unwrap();
    assert!(loaded_user.get_card(card.id()).is_none());
}
