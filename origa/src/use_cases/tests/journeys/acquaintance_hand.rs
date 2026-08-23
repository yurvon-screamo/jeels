//! Journeys режима знакомства (docs/acquaintance-mode.md): закрытие руки,
//! прерывание, «Уже знаю» и учёт дневного лимита.

use crate::domain::{NativeLanguage, OrigaError, RateMode, Rating, User};
use crate::traits::UserRepository;
use crate::use_cases::tests::fixtures::{InMemoryUserRepository, create_test_vocab_card};
use crate::use_cases::{
    CompleteAcquaintanceHandUseCase, MarkCardAsKnownUseCase, SelectAcquaintanceHandUseCase,
};
use chrono::{Duration, Utc};
use ulid::Ulid;

fn user_with_new_vocab_cards(count: usize) -> User {
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    for index in 0..count {
        let card = create_test_vocab_card(&format!("テスト{index}"));
        user.create_card(card).unwrap();
    }
    user
}

#[tokio::test]
async fn no_current_user_returns_current_user_not_exist() {
    // Arrange
    let repo = InMemoryUserRepository::new();
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();

    // Act
    let result = select.execute(&jlpt_content).await;

    // Assert
    assert!(matches!(result, Err(OrigaError::CurrentUserNotExist)));
}

#[tokio::test]
async fn select_returns_none_when_pool_is_empty() {
    // Arrange: юзер без карт
    let user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let repo = InMemoryUserRepository::with_user(user);
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();

    // Act
    let hand = select.execute(&jlpt_content).await.unwrap();

    // Assert
    assert_eq!(hand, None);
}

#[tokio::test]
async fn select_caps_hand_size_at_max_and_daily_limit() {
    // Arrange: 20 новых слов при дефолтном лимите (Medium = 9)
    let repo = InMemoryUserRepository::with_user(user_with_new_vocab_cards(20));
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();

    // Act
    let hand = select.execute(&jlpt_content).await.unwrap().unwrap();

    // Assert: рука не больше максимума и не больше дневного лимита
    assert_eq!(
        hand.len(),
        7,
        "20 кандидатов при лимите 9 дают руку ровно из HAND_MAX_SIZE карт"
    );
}

#[tokio::test]
async fn completed_hand_seeds_first_review_for_tomorrow_and_spends_limit_once() {
    // Arrange: 5 новых слов → рука ровно из 5 карт
    let repo = InMemoryUserRepository::with_user(user_with_new_vocab_cards(5));
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();
    let card_ids = select.execute(&jlpt_content).await.unwrap().unwrap();
    assert_eq!(card_ids.len(), 5);

    let complete = CompleteAcquaintanceHandUseCase::new(&repo);

    // Act
    complete.execute(card_ids.clone()).await.unwrap();

    // Assert: каждой карте сидировано состояние с ревью ~завтра,
    // журналы повторений не тронуты, лимит списан одной операцией
    let now = Utc::now();
    let user = repo.get_current_user().await.unwrap().unwrap();
    assert_eq!(user.knowledge_set().new_cards_studied_today(), 5);
    for card_id in &card_ids {
        let memory = user.knowledge_set().get_card(*card_id).unwrap().memory();
        assert!(!memory.is_new(), "seeded card must leave the New state");
        assert_eq!(memory.reps(), 0, "сидирование не является ревью");
        assert_eq!(memory.last_review_date(), None);
        let due = memory.next_review_date().unwrap();
        let delta = due.signed_duration_since(now);
        assert!(
            delta > Duration::days(1) - Duration::hours(1),
            "первое ревью должно быть завтра, got {delta:?}"
        );
        assert!(
            delta <= Duration::days(1) + Duration::hours(1),
            "первое ревью не должно улетать дальше завтра, got {delta:?}"
        );
    }
}

#[tokio::test]
async fn interrupted_hand_leaves_repository_state_identical() {
    // Arrange: снимок состояния до старта руки
    let repo = InMemoryUserRepository::with_user(user_with_new_vocab_cards(6));
    let before_limit = repo
        .get_current_user()
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .new_cards_studied_today();

    // Act: юзер выбывает посреди тренировки — рука эфемерна, никакой записи
    // нет по построению (Select ничего не сохраняет)
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();
    let hand = select.execute(&jlpt_content).await.unwrap().unwrap();

    // Assert: состояние репозитория логически идентично исходному
    let after = repo.get_current_user().await.unwrap().unwrap();
    assert_eq!(
        after.knowledge_set().new_cards_studied_today(),
        before_limit,
        "прерывание не тратит дневной лимит"
    );
    for card_id in &hand {
        let memory = after.knowledge_set().get_card(*card_id).unwrap().memory();
        assert!(memory.is_new(), "карты руки остались в пуле новыми");
    }
}

#[tokio::test]
async fn mark_known_during_presentation_skips_daily_limit() {
    // Arrange
    let repo = InMemoryUserRepository::with_user(user_with_new_vocab_cards(3));
    let known_id = repo
        .get_current_user()
        .await
        .unwrap()
        .unwrap()
        .knowledge_set()
        .study_cards()
        .keys()
        .copied()
        .next()
        .unwrap();
    let mark_known = MarkCardAsKnownUseCase::new(&repo);

    // Act
    mark_known.execute(known_id).await.unwrap();

    // Assert: карта известна, лимит не потрачен, рука собирается из остатка
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();
    let hand = select.execute(&jlpt_content).await.unwrap().unwrap();
    let user = repo.get_current_user().await.unwrap().unwrap();
    assert_eq!(user.knowledge_set().new_cards_studied_today(), 0);
    assert!(!hand.contains(&known_id), "известная карта выбыла из руки");
    assert_eq!(hand.len(), 2);
}

#[tokio::test]
async fn completing_unknown_cards_returns_card_not_found() {
    // Arrange
    let repo = InMemoryUserRepository::with_user(user_with_new_vocab_cards(2));
    let complete = CompleteAcquaintanceHandUseCase::new(&repo);

    // Act
    let result = complete.execute(vec![Ulid::new()]).await;

    // Assert
    assert!(matches!(result, Err(OrigaError::CardNotFound { .. })));
}

#[tokio::test]
async fn complete_skips_known_cards_and_charges_only_seeded() {
    // Arrange: рука из 3 карт, одна уже помечена «Уже знаю»
    let repo = InMemoryUserRepository::with_user(user_with_new_vocab_cards(3));
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();
    let card_ids = select.execute(&jlpt_content).await.unwrap().unwrap();

    let known_id = card_ids[0];
    MarkCardAsKnownUseCase::new(&repo)
        .execute(known_id)
        .await
        .unwrap();

    // Act: Complete получает ПОЛНЫЙ исходный список руки
    CompleteAcquaintanceHandUseCase::new(&repo)
        .execute(card_ids.clone())
        .await
        .unwrap();

    // Assert: известная карта осталась известной без повторного сидирования,
    // лимит списан только за две фактически сидированные
    let user = repo.get_current_user().await.unwrap().unwrap();
    assert_eq!(user.knowledge_set().new_cards_studied_today(), 2);
    assert!(
        user.knowledge_set()
            .get_card(known_id)
            .unwrap()
            .memory()
            .is_known_card()
    );
    for card_id in &card_ids[1..] {
        let memory = user.knowledge_set().get_card(*card_id).unwrap().memory();
        assert!(!memory.is_new());
        assert!(!memory.is_known_card());
    }
}

#[tokio::test]
async fn rating_a_seeded_card_tomorrow_keeps_fsrs_evolution_normal() {
    // Arrange: закрытая рука + рейтинговый путь штатной оценки Good
    let repo = InMemoryUserRepository::with_user(user_with_new_vocab_cards(1));
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();
    let card_ids = select.execute(&jlpt_content).await.unwrap().unwrap();
    CompleteAcquaintanceHandUseCase::new(&repo)
        .execute(card_ids.clone())
        .await
        .unwrap();

    // Act: «повторка назавтра» через обычный рейтинг (последний успех
    // тренировки уже дал карте Review-состояние с интервалом ≥ дня)
    let mut user = repo.get_current_user().await.unwrap().unwrap();
    let card_id = card_ids[0];
    let stability_before = user
        .knowledge_set()
        .get_card(card_id)
        .unwrap()
        .memory()
        .stability()
        .unwrap()
        .value();
    user.rate_card(card_id, Rating::Good, RateMode::StandardLesson)
        .unwrap();

    // Assert: штатная эволюция без деградации в learning-минуты
    let stability_after = user
        .knowledge_set()
        .get_card(card_id)
        .unwrap()
        .memory()
        .stability()
        .unwrap()
        .value();
    assert!(
        stability_after >= stability_before,
        "успешное первое ревью не должно уменьшать стабильность"
    );
}
