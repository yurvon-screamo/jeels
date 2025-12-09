#[path = "mod.rs"]
mod tests;

use keikaku::application::use_cases::{CreateCardUseCase, RateCardUseCase};
use keikaku::application::user_repository::UserRepository;
use keikaku::domain::value_objects::{Answer, CardContent, Rating};
use keikaku::settings::ApplicationEnvironment;
use tests::*;

#[tokio::test]
async fn rate_card_use_case_should_add_review_and_update_schedule() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let llm_service = settings.get_llm_service().await.unwrap();
    let create_use_case = CreateCardUseCase::new(repository, llm_service);
    let card = create_use_case
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

    let srs_service = settings.get_srs_service().await.unwrap();
    let rate_use_case = RateCardUseCase::new(repository, srs_service);

    // Act
    rate_use_case
        .execute(user.id(), card.id(), Rating::Good)
        .await
        .unwrap();

    // Assert
    let loaded_user = repository.find_by_id(user.id()).await.unwrap().unwrap();
    let loaded_card = loaded_user.get_card(card.id()).unwrap();
    assert_eq!(loaded_card.memory().reviews().len(), 1);
    assert_eq!(loaded_card.memory().reviews()[0].rating(), Rating::Good);
    assert!(loaded_card.memory().difficulty().is_some());
    assert!(loaded_card.memory().stability().is_some());
}
