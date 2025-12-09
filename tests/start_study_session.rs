#[path = "mod.rs"]
mod tests;

use keikaku::{
    application::use_cases::{CreateCardUseCase, SelectCardsToLearnUseCase},
    domain::{
        study_session::StudySessionItem,
        value_objects::{Answer, CardContent},
    },
    settings::ApplicationEnvironment,
};
use tests::*;

#[tokio::test]
async fn start_study_session_use_case_should_return_due_cards() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_service().await.unwrap();
    let llm_service = settings.get_llm_service().await.unwrap();
    let create_use_case = CreateCardUseCase::new(repository, embedding_generator, llm_service);
    create_use_case
        .execute(
            user.id(),
            "あります".to_string(),
            Some(CardContent::new(
                Answer::new("есть".to_string()).unwrap(),
                Vec::new(),
            )),
        )
        .await
        .unwrap();

    let start_session_use_case = SelectCardsToLearnUseCase::new(repository);

    // Act
    let cards = start_session_use_case
        .execute(user.id(), false, false)
        .await
        .unwrap();

    // Assert
    assert_eq!(cards.len(), 1);

    let card = &cards[0];
    if let StudySessionItem::Vocabulary(card) = card {
        assert_eq!(card.word(), "あります");
    } else {
        panic!("Card is not a vocabulary card");
    }
}
