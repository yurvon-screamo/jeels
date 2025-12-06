#[path = "mod.rs"]
mod tests;

use keikaku::application::use_cases::CreateCardUseCase;
use keikaku::application::user_repository::UserRepository;
use keikaku::domain::JeersError;
use keikaku::settings::ApplicationEnvironment;
use rstest::rstest;
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

    assert_eq!(card.answer().text(), "Съедает, потребляет что-то как еду");
}

#[rstest]
#[case("食べる", "食べます", "есть")]
#[case("行く", "行きます", "идти")]
#[case("見る", "見ます", "видеть")]
#[case("来る", "来ます", "приходить")]
#[case("する", "します", "делать")]
#[case("ある", "あります", "быть")]
#[case("読む", "読みます", "читать")]
#[case("書く", "書きます", "писать")]
#[case("話す", "話します", "говорить")]
#[case("聞く", "聞きます", "слушать")]
#[case("買う", "買います", "покупать")]
#[case("売る", "売ります", "продавать")]
#[case("私", "わたし", "я")]
#[case("水", "みず", "вода")]
#[case("火", "ひ", "огонь")]
#[case("本", "ほん", "книга")]
#[tokio::test]
async fn create_card_use_case_should_return_error_if_similar_card_already_exists(
    #[case] existing_question: &str,
    #[case] similar_question: &str,
    #[case] answer: &str,
) {
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
            existing_question.to_string(),
            Some(answer.to_string()),
            Some(vec![]),
        )
        .await
        .unwrap();

    assert_eq!(card1.question().text(), existing_question);
    assert_eq!(card1.answer().text(), answer);

    let card2 = use_case
        .execute(
            user.id(),
            similar_question.to_string(),
            Some(answer.to_string()),
            Some(vec![]),
        )
        .await
        .unwrap_err();

    assert!(matches!(card2, JeersError::DuplicateCard { question: _ }));
    if let JeersError::DuplicateCard { question } = card2 {
        assert_eq!(question, similar_question);
    }
}

#[rstest]
#[case("良い", "хороший", "素晴らしい", "замечательный")]
#[case("速い", "быстрый", "迅速な", "быстрый")]
#[case("易しい", "легкий", "簡単", "простой")]
#[case("新しい", "новый", "新規の", "новый")]
#[case("大きい", "большой", "巨大な", "огромный")]
#[case("見る", "видеть", "観察する", "наблюдать")]
#[case("聞く", "слушать", "聴く", "слушать внимательно")]
#[case("話す", "говорить", "言う", "сказать")]
#[case("買う", "покупать", "購入する", "покупать")]
#[case("本", "книга", "書籍", "книга")]
#[case("学校", "школа", "学園", "учебное заведение")]
#[case("読む", "читать", "閲覧する", "читать")]
#[case("書く", "писать", "記述する", "описывать")]
#[case("美しい", "красивый", "綺麗な", "красивый")]
#[case("嬉しい", "радостный", "楽しい", "веселый")]
#[case("走る", "бежать", "駆ける", "бежать")]
#[case("食べる", "есть", "食う", "есть")]
#[case("飲む", "пить", "呑む", "пить")]
#[case("考える", "думать", "思う", "думать")]
#[case("教える", "учить", "指導する", "обучать")]
#[tokio::test]
async fn create_card_use_case_should_return_ok_is_not_similar_card_already_exists(
    #[case] first_question: &str,
    #[case] first_answer: &str,
    #[case] second_question: &str,
    #[case] second_answer: &str,
) {
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
            first_question.to_string(),
            Some(first_answer.to_string()),
            Some(vec![]),
        )
        .await
        .unwrap();

    assert_eq!(card1.question().text(), first_question);
    assert_eq!(card1.answer().text(), first_answer);

    let card2 = use_case
        .execute(
            user.id(),
            second_question.to_string(),
            Some(second_answer.to_string()),
            Some(vec![]),
        )
        .await
        .unwrap();

    assert_eq!(card2.question().text(), second_question);
    assert_eq!(card2.answer().text(), second_answer);
}
