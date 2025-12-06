#[path = "mod.rs"]
mod tests;

use keikaku::application::use_cases::CreateCardUseCase;
use keikaku::application::user_repository::UserRepository;
use keikaku::domain::JeersError;
use keikaku::settings::ApplicationEnvironment;
use tests::*;

#[tokio::test]
async fn create_card_use_case_should_create_card_and_save_to_database() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_generator().await.unwrap();
    let llm_service = settings.get_llm_service().await.unwrap();
    let use_case = CreateCardUseCase::new(repository, embedding_generator, llm_service);

    // Act
    let card = use_case
        .execute(
            user.id(),
            "あります".to_string(),
            Some("есть".to_string()),
            Some(vec![]),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(card.question().text(), "あります");
    assert_eq!(card.answer().text(), "есть");
}

#[tokio::test]
async fn create_card_use_case_should_persist_card_in_database() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_generator().await.unwrap();
    let llm_service = settings.get_llm_service().await.unwrap();
    let use_case = CreateCardUseCase::new(repository, embedding_generator, llm_service);
    let card = use_case
        .execute(
            user.id(),
            "私".to_string(),
            Some("я".to_string()),
            Some(vec![]),
        )
        .await
        .unwrap();

    // Act
    let loaded_user = repository.find_by_id(user.id()).await.unwrap().unwrap();

    // Assert
    let loaded_card = loaded_user.get_card(card.id()).unwrap();
    assert_eq!(loaded_card.question().text(), "私");
    assert_eq!(loaded_card.answer().text(), "я");
}

#[tokio::test]
async fn create_card_use_case_should_generate_answer_if_not_provided() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_generator().await.unwrap();
    let llm_service = settings.get_llm_service().await.unwrap();
    let use_case = CreateCardUseCase::new(repository, embedding_generator, llm_service);

    let card = use_case
        .execute(user.id(), "食べます".to_string(), None, Some(vec![]))
        .await
        .unwrap();

    assert_eq!(card.answer().text(), "Есть");
}

#[tokio::test]
async fn create_card_use_case_should_return_error_if_similar_card_already_exists() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_generator().await.unwrap();
    let llm_service = settings.get_llm_service().await.unwrap();
    let use_case = CreateCardUseCase::new(repository, embedding_generator, llm_service);

    let card1 = use_case
        .execute(
            user.id(),
            "食べる".to_string(),
            Some("есть".to_string()),
            Some(vec![]),
        )
        .await
        .unwrap();

    assert!(card1.question().text() == "食べる");
    assert!(card1.answer().text() == "есть");

    let card2 = use_case
        .execute(
            user.id(),
            "食べます".to_string(),
            Some("есть".to_string()),
            Some(vec![]),
        )
        .await
        .unwrap_err();

    assert!(matches!(card2, JeersError::DuplicateCard { question: _ }));
}

#[tokio::test]
async fn create_card_use_case_should_return_ok_is_not_similar_card_already_exists() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_generator().await.unwrap();
    let llm_service = settings.get_llm_service().await.unwrap();
    let use_case = CreateCardUseCase::new(repository, embedding_generator, llm_service);

    let card1 = use_case
        .execute(
            user.id(),
            "食べます".to_string(),
            Some("есть".to_string()),
            Some(vec![]),
        )
        .await
        .unwrap();

    assert!(card1.question().text() == "食べる");
    assert!(card1.answer().text() == "есть");

    let card2 = use_case
        .execute(
            user.id(),
            "飲みます".to_string(),
            Some("пить".to_string()),
            Some(vec![]),
        )
        .await
        .unwrap();

    assert!(card2.question().text() == "飲みます");
    assert!(card2.answer().text() == "пить");
}
