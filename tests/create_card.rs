#[path = "mod.rs"]
mod tests;

use keikaku::application::use_cases::CreateCardUseCase;
use keikaku::settings::ApplicationEnvironment;
use tests::*;

#[tokio::test]
async fn create_card_use_case_should_create_card_and_save_to_database() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_service().await.unwrap();
    let llm_service = settings.get_llm_service().await.unwrap();
    let use_case = CreateCardUseCase::new(repository, embedding_generator, llm_service);
    let card = use_case
        .execute(user.id(), "あります".to_string(), None)
        .await
        .unwrap();

    // Assert
    assert_eq!(card.word().text(), "あります");
    assert_eq!(card.meaning().text(), "есть");
}
