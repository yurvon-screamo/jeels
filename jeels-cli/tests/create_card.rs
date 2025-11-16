#[path = "mod.rs"]
mod tests;

use jeels_cli::application::use_cases::CreateCardUseCase;
use jeels_cli::application::user_repository::UserRepository;
use jeels_cli::settings::Settings;
use tests::*;

#[tokio::test]
async fn create_card_use_case_should_create_card_and_save_to_database() {
    // Arrange
    create_test_repository().await;
    let settings = Settings::get();
    let repository = settings.get_repository();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_generator();
    let use_case = CreateCardUseCase::new(repository, embedding_generator);

    // Act
    let card = use_case
        .execute(
            user.id(),
            "What is Rust?".to_string(),
            "A systems programming language".to_string(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(card.question().text(), "What is Rust?");
    assert_eq!(card.answer().text(), "A systems programming language");
}

#[tokio::test]
async fn create_card_use_case_should_persist_card_in_database() {
    // Arrange
    let ctx = create_test_repository().await;
    let settings = Settings::get();
    let repository = settings.get_repository();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_generator();
    let use_case = CreateCardUseCase::new(repository, embedding_generator);
    let card = use_case
        .execute(
            user.id(),
            "What is Rust?".to_string(),
            "A systems programming language".to_string(),
        )
        .await
        .unwrap();

    // Act
    let loaded_user = repository.find_by_id(user.id()).await.unwrap().unwrap();

    // Assert
    let loaded_card = loaded_user.get_card(card.id()).unwrap();
    assert_eq!(loaded_card.question().text(), "What is Rust?");
    assert_eq!(
        loaded_card.answer().text(),
        "A systems programming language"
    );
}
