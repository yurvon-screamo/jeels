#[path = "mod.rs"]
mod tests;

use keikaku::{
    application::{
        create_card::CardContent,
        use_cases::{CreateCardUseCase, ViewCardUseCase},
    },
    domain::value_objects::Answer,
    settings::ApplicationEnvironment,
};
use tests::*;

#[tokio::test]
async fn view_card_use_case_should_return_card_question_and_answer() {
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
            Some(CardContent {
                answer: Answer::new("A systems programming language".to_string()).unwrap(),
                example_phrases: vec![],
            }),
        )
        .await
        .unwrap();

    let view_use_case = ViewCardUseCase::new(repository);

    // Act
    let (question, answer) = view_use_case.execute(user.id(), card.id()).await.unwrap();

    // Assert
    assert_eq!(question.text(), "What is Rust?");
    assert_eq!(answer.text(), "A systems programming language");
}
