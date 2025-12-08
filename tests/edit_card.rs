#[path = "mod.rs"]
mod tests;

use keikaku::application::use_cases::{CreateCardUseCase, EditCardUseCase};
use keikaku::application::user_repository::UserRepository;
use keikaku::domain::value_objects::{Answer, CardContent};
use keikaku::settings::ApplicationEnvironment;
use tests::*;

#[tokio::test]
async fn edit_card_use_case_should_update_card_in_database() {
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
            Some(CardContent::new(
                Answer::new("A systems programming language".to_string()).unwrap(),
                Vec::new(),
            )),
        )
        .await
        .unwrap();

    let edit_use_case = EditCardUseCase::new(repository, embedding_generator);

    // Act
    edit_use_case
        .execute(
            user.id(),
            card.id(),
            "What is Rust language?".to_string(),
            "A memory-safe systems programming language".to_string(),
            vec![],
        )
        .await
        .unwrap();

    // Assert
    let loaded_user = repository.find_by_id(user.id()).await.unwrap().unwrap();
    let loaded_card = loaded_user.get_card(card.id()).unwrap();
    assert_eq!(loaded_card.question().text(), "What is Rust language?");
    assert_eq!(
        loaded_card.answer().text(),
        "A memory-safe systems programming language"
    );
}
