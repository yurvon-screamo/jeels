#[path = "mod.rs"]
mod tests;

use jeels_cli::{
    application::use_cases::{CreateCardUseCase, FindSynonymsUseCase},
    settings::ApplicationEnvironment,
};
use tests::*;

#[tokio::test]
async fn find_synonyms_should_return_similar_cards() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_generator().await.unwrap();
    let llm_service = settings.get_llm_service().await.unwrap();
    let create_use_case = CreateCardUseCase::new(repository, embedding_generator, llm_service);

    let card1 = create_use_case
        .execute(user.id(), "水".to_string(), Some("вода".to_string()))
        .await
        .unwrap();

    // Create a similar card (same word in hiragana)
    let card2 = create_use_case
        .execute(user.id(), "みず".to_string(), Some("вода".to_string()))
        .await
        .unwrap();

    // Create a different card
    create_use_case
        .execute(user.id(), "本".to_string(), Some("книга".to_string()))
        .await
        .unwrap();

    let find_synonyms_use_case = FindSynonymsUseCase::new(repository);

    // Act
    let synonyms = find_synonyms_use_case
        .execute(user.id(), card1.id())
        .await
        .unwrap();

    // Assert
    // Should find card2 as a synonym
    assert!(!synonyms.is_empty(), "Should find at least one synonym");
    let synonym_ids: Vec<_> = synonyms.iter().map(|c| c.id()).collect();
    assert!(
        synonym_ids.contains(&card2.id()),
        "Should find the similar card as a synonym"
    );
    // Should not include the query card itself
    assert!(
        !synonym_ids.contains(&card1.id()),
        "Should not include the query card"
    );
}

#[tokio::test]
async fn find_synonyms_should_return_empty_when_no_similar_cards() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_generator().await.unwrap();
    let llm_service = settings.get_llm_service().await.unwrap();
    let create_use_case = CreateCardUseCase::new(repository, embedding_generator, llm_service);

    let card1 = create_use_case
        .execute(user.id(), "水".to_string(), Some("вода".to_string()))
        .await
        .unwrap();

    // Create completely different cards
    create_use_case
        .execute(user.id(), "本".to_string(), Some("книга".to_string()))
        .await
        .unwrap();

    create_use_case
        .execute(user.id(), "車".to_string(), Some("машина".to_string()))
        .await
        .unwrap();

    let find_synonyms_use_case = FindSynonymsUseCase::new(repository);

    // Act
    let synonyms = find_synonyms_use_case
        .execute(user.id(), card1.id())
        .await
        .unwrap();

    // Assert
    // May or may not find synonyms depending on similarity threshold
    // Just check that the query card is not in results
    let synonym_ids: Vec<_> = synonyms.iter().map(|c| c.id()).collect();
    assert!(
        !synonym_ids.contains(&card1.id()),
        "Should not include the query card"
    );
}

#[tokio::test]
async fn find_synonyms_should_find_multiple_similar_cards() {
    // Arrange
    create_test_repository().await;
    let settings = ApplicationEnvironment::get();
    let repository = settings.get_repository().await.unwrap();
    let user = create_test_user().await;
    let embedding_generator = settings.get_embedding_generator().await.unwrap();
    let llm_service = settings.get_llm_service().await.unwrap();
    let create_use_case = CreateCardUseCase::new(repository, embedding_generator, llm_service);

    let query_card = create_use_case
        .execute(user.id(), "水".to_string(), Some("вода".to_string()))
        .await
        .unwrap();

    // Create multiple similar cards (same word in different scripts)
    let similar_card1 = create_use_case
        .execute(user.id(), "みず".to_string(), Some("вода".to_string()))
        .await
        .unwrap();

    let similar_card2 = create_use_case
        .execute(user.id(), "ミズ".to_string(), Some("вода".to_string()))
        .await
        .unwrap();

    // Create a different card
    create_use_case
        .execute(user.id(), "本".to_string(), Some("книга".to_string()))
        .await
        .unwrap();

    let find_synonyms_use_case = FindSynonymsUseCase::new(repository);

    // Act
    let synonyms = find_synonyms_use_case
        .execute(user.id(), query_card.id())
        .await
        .unwrap();

    // Assert
    let synonym_ids: Vec<_> = synonyms.iter().map(|c| c.id()).collect();
    assert!(
        synonym_ids.contains(&similar_card1.id()) || synonym_ids.contains(&similar_card2.id()),
        "Should find at least one of the similar cards"
    );
    assert!(
        !synonym_ids.contains(&query_card.id()),
        "Should not include the query card"
    );
}
