use ulid::Ulid;

use crate::domain::{JapaneseLevel, NativeLanguage, OrigaError, SetType, User, WellKnownSetMeta};
use crate::traits::UserRepository;
use crate::use_cases::tests::fixtures::{
    InMemoryUserRepository, create_test_vocab_card, init_real_dictionaries,
};
use crate::use_cases::{
    CreateKanjiCardUseCase, CreateVocabularyCardUseCase, DeleteCardUseCase, ToggleFavoriteUseCase,
};

async fn create_repo() -> InMemoryUserRepository {
    InMemoryUserRepository::with_user(User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ))
}

fn user_with_card(word: &str) -> User {
    let mut u = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    u.create_card(create_test_vocab_card(word)).unwrap();
    u
}

async fn first_card_id(repo: &InMemoryUserRepository) -> Ulid {
    *repo
        .get_current_user()
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .study_cards()
        .keys()
        .next()
        .expect("repo must contain at least one card")
}

#[tokio::test]
async fn well_known_set_minna_nihongo_serialization() {
    let meta = WellKnownSetMeta {
        id: "minna_n5".to_string(),
        set_type: SetType::from("MinnaNoNihongo"),
        level: JapaneseLevel::N5,
        title_ru: "Minna no Nihongo N5".to_string(),
        title_en: "Minna no Nihongo N5".to_string(),
        desc_ru: "Базовый японский учебник уровень N5".to_string(),
        desc_en: "Basic Japanese textbook N5 level".to_string(),
        word_count: 100,
    };

    let set_type = meta.set_type;
    assert_eq!(set_type, "MinnaNoNihongo");
}

#[tokio::test]
async fn toggle_favorite_on_card_marks_it_as_favorite() {
    // Arrange
    let repo = InMemoryUserRepository::with_user(user_with_card("word"));
    let card_id = first_card_id(&repo).await;
    let use_case = ToggleFavoriteUseCase::new(&repo);

    // Act
    let is_favorite = use_case.execute(card_id).await.unwrap();

    // Assert
    assert!(is_favorite);
}

#[tokio::test]
async fn toggle_favorite_twice_returns_to_original_state() {
    // Arrange
    let repo = InMemoryUserRepository::with_user(user_with_card("word"));
    let card_id = first_card_id(&repo).await;
    let use_case = ToggleFavoriteUseCase::new(&repo);

    // Act
    let first = use_case.execute(card_id).await.unwrap();
    let second = use_case.execute(card_id).await.unwrap();

    // Assert
    assert!(first, "first toggle turns favorite on");
    assert!(!second, "second toggle turns favorite off");
}

#[tokio::test]
async fn delete_card_removes_from_knowledge_set() {
    // Arrange
    let repo = InMemoryUserRepository::with_user(user_with_card("word"));
    let card_id = first_card_id(&repo).await;
    let use_case = DeleteCardUseCase::new(&repo);

    // Act
    use_case.execute(card_id).await.unwrap();

    // Assert
    let updated = repo.get_current_user().await.unwrap().unwrap();
    assert!(updated.knowledge_set().study_cards().is_empty());
}

#[tokio::test]
async fn delete_nonexistent_card_returns_error() {
    // Arrange
    let repo = create_repo().await;
    let use_case = DeleteCardUseCase::new(&repo);
    let non_existent_card_id = Ulid::new();

    // Act
    let result = use_case.execute(non_existent_card_id).await;

    // Assert
    assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
}

#[tokio::test]
async fn toggle_favorite_nonexistent_card_returns_error() {
    // Arrange
    let repo = create_repo().await;
    let use_case = ToggleFavoriteUseCase::new(&repo);
    let non_existent_card_id = Ulid::new();

    // Act
    let result = use_case.execute(non_existent_card_id).await;

    // Assert
    assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
}

#[tokio::test]
async fn create_vocabulary_card_empty_text_returns_empty_result() {
    // Arrange
    init_real_dictionaries();
    let repo = create_repo().await;
    let use_case = CreateVocabularyCardUseCase::new(&repo);

    // Act
    let result = use_case.execute("".to_string()).await.unwrap();

    // Assert
    assert!(result.created_cards.is_empty());
    assert!(result.skipped_no_translation.is_empty());
    assert!(result.skipped_duplicates.is_empty());
}

#[tokio::test]
async fn create_vocabulary_card_whitespace_only_returns_empty_result() {
    // Arrange
    init_real_dictionaries();
    let repo = create_repo().await;
    let use_case = CreateVocabularyCardUseCase::new(&repo);

    // Act
    let result = use_case.execute("   ".to_string()).await.unwrap();

    // Assert
    assert!(result.created_cards.is_empty());
}

#[tokio::test]
async fn create_kanji_card_duplicate_returns_error() {
    // Arrange
    init_real_dictionaries();
    let repo = create_repo().await;
    let use_case = CreateKanjiCardUseCase::new(&repo);

    // Act
    use_case.execute(vec!["人".to_string()]).await.unwrap();
    let result = use_case.execute(vec!["人".to_string()]).await;

    // Assert
    assert!(matches!(result, Err(OrigaError::DuplicateCard { .. })));
}

#[tokio::test]
async fn delete_card_already_deleted_returns_error() {
    // Arrange
    let repo = InMemoryUserRepository::with_user(user_with_card("word"));
    let card_id = first_card_id(&repo).await;
    let use_case = DeleteCardUseCase::new(&repo);

    // Act
    use_case.execute(card_id).await.unwrap();
    let result = use_case.execute(card_id).await;

    // Assert
    assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
}
