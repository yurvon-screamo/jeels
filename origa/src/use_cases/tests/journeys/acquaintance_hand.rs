//! Journeys режима знакомства (docs/acquaintance-mode.md): закрытие руки,
//! прерывание, «Уже знаю» и учёт дневного лимита.

use crate::domain::{
    Card, DailyBudget, JlptContent, NativeLanguage, NewCardPolicy, OrigaError, RateMode, Rating,
    User,
};
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

fn user_with_new_cards(vocab: usize, kanji: usize, grammar: usize) -> User {
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    for index in 0..vocab {
        let card = create_test_vocab_card(&format!("テスト{index}"));
        user.create_card(card).unwrap();
    }
    for code in 0x4e00..0x4e00 + kanji as u32 {
        let card = Card::Kanji(crate::domain::KanjiCard::new_test(
            char::from_u32(code).unwrap().to_string(),
        ));
        user.create_card(card).unwrap();
    }
    for _ in 0..grammar {
        user.create_card(Card::Grammar(crate::domain::GrammarRuleCard::new_test()))
            .unwrap();
    }
    user
}

/// Состав руки по типам: (слова, кандзи, грамматика).
async fn hand_type_counts(repo: &InMemoryUserRepository, hand: &[Ulid]) -> (usize, usize, usize) {
    let user = repo.get_current_user().await.unwrap().unwrap();
    let mut counts = (0usize, 0usize, 0usize);
    for card_id in hand {
        match user.knowledge_set().get_card(*card_id).unwrap().card() {
            Card::Vocabulary(_) => counts.0 += 1,
            Card::Kanji(_) => counts.1 += 1,
            Card::Grammar(_) => counts.2 += 1,
            Card::Phrase(_) => {},
        }
    }
    counts
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
async fn select_takes_full_hand_up_to_max_size() {
    // Arrange: 20 новых слов при дефолтном лимите (Light = 14)
    let repo = InMemoryUserRepository::with_user(user_with_new_vocab_cards(20));
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();

    // Act
    let hand = select.execute(&jlpt_content).await.unwrap().unwrap();

    // Assert: рука всегда полного размера (до HAND_MAX_SIZE), пока
    // в пуле есть карты
    assert_eq!(
        hand.len(),
        7,
        "20 кандидатов в пуле дают руку ровно из HAND_MAX_SIZE карт"
    );
}

/// Рука всегда полная: малый остаток дневного лимита не
/// урезает размер руки — лимит кратен HAND_MAX_SIZE и гейтит только
/// открытие руки. После закрытия такой руки лимит исчерпан — второй
/// руки в этот день нет.
#[tokio::test]
async fn small_daily_remainder_still_takes_full_hand_then_stops() {
    // Arrange: дефолтный лимит Light = 14; 12 карт уже изучено сегодня
    // (штатный инкремент лимита — рейтинг новых карт), 9 остались новыми
    let mut user = user_with_new_vocab_cards(21);
    let studied_ids: Vec<Ulid> = user
        .knowledge_set()
        .study_cards()
        .keys()
        .copied()
        .take(12)
        .collect();
    for card_id in studied_ids {
        user.rate_card(card_id, Rating::Good, RateMode::StandardLesson)
            .unwrap();
    }
    assert_eq!(user.knowledge_set().new_cards_studied_today(), 12);
    let repo = InMemoryUserRepository::with_user(user);
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();

    // Act: рука — полная, из 7 карт, несмотря на остаток лимита 2
    let first_hand = select.execute(&jlpt_content).await.unwrap().unwrap();
    assert_eq!(first_hand.len(), 7, "остаток лимита не режет размер руки");

    // Закрываем руку: сидирование + списание лимита одной операцией
    CompleteAcquaintanceHandUseCase::new(&repo)
        .execute(first_hand)
        .await
        .unwrap();

    // Assert: лимит исчерпан (12 + 7 ≥ 14) — второй руки нет
    let second_hand = select.execute(&jlpt_content).await.unwrap();
    assert_eq!(second_hand, None, "после исчерпания лимита рук нет");
}

#[tokio::test]
async fn select_balanced_pool_splits_hand_by_card_type_weights() {
    // Arrange: запас карт каждого типа — доли CARD_TYPE_WEIGHTS (V:K:G ≈ 8:1:1)
    let repo = InMemoryUserRepository::with_user(user_with_new_cards(20, 5, 5));
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();

    // Act
    let hand = select.execute(&jlpt_content).await.unwrap().unwrap();

    // Assert: рука 7 карт = 5 слов + 1 кандзи + 1 грамматика, а не верхушка
    // пула одного типа (регресс: рука из одних грамматик)
    assert_eq!(hand.len(), 7);
    assert_eq!(hand_type_counts(&repo, &hand).await, (5, 1, 1));
}

#[tokio::test]
async fn select_exhausted_vocab_takes_all_words_and_fills_rest() {
    // Arrange: слов меньше доли веса — fallback добирает остаток другими типами
    let repo = InMemoryUserRepository::with_user(user_with_new_cards(2, 10, 10));
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();

    // Act
    let hand = select.execute(&jlpt_content).await.unwrap().unwrap();

    // Assert: все доступные слова в руке, рука заполнена до максимума
    let (vocab, kanji, grammar) = hand_type_counts(&repo, &hand).await;
    assert_eq!(vocab, 2, "каждое доступное новое слово попадает в руку");
    assert_eq!(hand.len(), 7);
    assert_eq!(kanji + grammar, 5);
}

#[tokio::test]
async fn select_pool_without_words_fills_hand_with_kanji_and_grammar() {
    // Arrange: пул без слов — веса ренормализуются на кандзи и грамматику
    let repo = InMemoryUserRepository::with_user(user_with_new_cards(0, 10, 10));
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();

    // Act
    let hand = select.execute(&jlpt_content).await.unwrap().unwrap();

    // Assert: рука заполнена только кандзи и грамматикой (точный сплит
    // K:G недетерминирован — random tie-break равных дробных частей)
    let (vocab, kanji, grammar) = hand_type_counts(&repo, &hand).await;
    assert_eq!(vocab, 0);
    assert_eq!(hand.len(), 7);
    assert_eq!(kanji + grammar, 7);
}

/// Превращает карту в просроченное ревью: стабильность/сложность любая
/// валидная, дата следующего показа — вчера (паттерн seed_known из
/// lesson_builder-тестов).
fn make_card_due(user: &mut User, card_id: Ulid) {
    let due_memory = crate::domain::MemoryState::new(
        crate::domain::Stability::new(10.0).unwrap(),
        crate::domain::Difficulty::new(5.0).unwrap(),
        Utc::now() - Duration::days(1),
    );
    let study_card = user
        .knowledge_set_mut()
        .study_cards_mut_for_test()
        .get_mut(&card_id)
        .unwrap();
    study_card.apply_review(due_memory, Rating::Good);
}

/// Долги вперёд с коротким хвостом: due-очередь, поверх которой
/// помещается полная рука, НЕ откладывает знакомство — рука кладётся
/// поверх хвоста (историческое поведение впрыска новых карт):
/// due + HAND_MAX_SIZE ≤ MAX_LESSON_SIZE.
#[tokio::test]
async fn select_takes_hand_when_due_tail_fits_lesson_capacity() {
    // Arrange: 8 due-долгов (8 + 7 ≤ 22) + новые слова
    let mut user = user_with_new_vocab_cards(11);
    let due_ids: Vec<Ulid> = user
        .knowledge_set()
        .study_cards()
        .keys()
        .copied()
        .take(8)
        .collect();
    for id in due_ids {
        make_card_due(&mut user, id);
    }
    let repo = InMemoryUserRepository::with_user(user);
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();

    // Act
    let hand = select.execute(&jlpt_content).await.unwrap();

    // Assert: короткий хвост — рука собирается из новых карт
    let hand = hand.expect("короткий due-хвост не блокирует руку");
    assert!(!hand.is_empty());
}

/// Глубокая due-очередь откладывает руку: due-долгов столько, что
/// полная рука поверх них не влезает в урок — юзер разбирает долги
/// чистым ревью, знакомство вернётся после очистки очереди.
#[tokio::test]
async fn select_defers_hand_when_due_queue_is_deep() {
    // Arrange: due-долгов больше, чем (MAX_LESSON_SIZE - HAND_MAX_SIZE)
    let deep = crate::domain::MAX_LESSON_SIZE - crate::domain::HAND_MAX_SIZE + 1;
    let mut user = user_with_new_vocab_cards(deep + 3);
    let due_ids: Vec<Ulid> = user
        .knowledge_set()
        .study_cards()
        .keys()
        .copied()
        .take(deep)
        .collect();
    for id in due_ids {
        make_card_due(&mut user, id);
    }
    let repo = InMemoryUserRepository::with_user(user);
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();

    // Act
    let hand = select.execute(&jlpt_content).await.unwrap();

    // Assert: глубокая очередь — рука отложена
    assert!(
        hand.is_none(),
        "глубокая due-очередь откладывает знакомство"
    );
}

/// Просроченная фраза — не долг для знакомства: фразы живут собственными
/// пайплайнами и не должны блокировать руку.
#[tokio::test]
async fn select_keeps_hand_when_only_due_phrases_are_pending() {
    // Arrange: новые слова + просроченная фраза
    let mut user = user_with_new_vocab_cards(3);
    let phrase_card_id = *user
        .create_card(Card::Phrase(crate::domain::PhraseCard::new(Ulid::new())))
        .unwrap()
        .card_id();
    make_card_due(&mut user, phrase_card_id);
    let repo = InMemoryUserRepository::with_user(user);
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();

    // Act
    let hand = select.execute(&jlpt_content).await.unwrap();

    // Assert: фраза не блокирует — рука собирается из новых слов
    let hand = hand.expect("due-фраза не блокирует руку знакомства");
    assert!(!hand.is_empty());
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

#[test]
fn favorited_unknown_card_never_enters_exclude_review_but_enters_inject() {
    // Arrange: A — новая избранная, B — известная должная, C — новая обычная
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let a = *user
        .create_card(create_test_vocab_card("あかい"))
        .unwrap()
        .card_id();
    let b = *user
        .create_card(create_test_vocab_card("ちいさい"))
        .unwrap()
        .card_id();
    let c = *user
        .create_card(create_test_vocab_card("おおきい"))
        .unwrap()
        .card_id();
    user.toggle_favorite(a).unwrap();
    user.mark_card_as_known(b).unwrap();

    let budget = DailyBudget::from_load(*user.daily_load());
    let level = user.current_japanese_level();
    let lang = *user.native_language();
    let jlpt_content = JlptContent::new();

    // Act
    let inject = user
        .knowledge_set()
        .cards_to_lesson(budget, &jlpt_content, level, lang);
    let exclude = user.knowledge_set().cards_to_lesson_with_policy(
        budget,
        &jlpt_content,
        level,
        lang,
        NewCardPolicy::Exclude,
    );

    // Assert Inject (историческое поведение): избранная новая попадает в урок
    assert!(
        inject.card_ids().contains(&a),
        "Inject сохраняет легаси-поведение впрыска новых"
    );

    // Assert Exclude: ни одна незнакомая карта не прошла, известная — на месте
    for (id, _) in &exclude.cards {
        let memory = user.knowledge_set().get_card(*id).unwrap().memory();
        assert!(!memory.is_new(), "Exclude пропустил незнакомую карту {id}");
    }
    assert!(!exclude.card_ids().contains(&a));
    assert!(!exclude.card_ids().contains(&c));
    assert!(
        exclude.card_ids().contains(&b),
        "известная должная карта остаётся в ревью"
    );
}

#[tokio::test]
async fn replacement_takes_top_new_card_excluding_hand() {
    // Arrange: три новых слова, два уже в руке
    let repo = InMemoryUserRepository::with_user(user_with_new_vocab_cards(3));
    let select = SelectAcquaintanceHandUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();
    let hand = select.execute(&jlpt_content).await.unwrap().unwrap();
    assert_eq!(hand.len(), 3);

    // Act: исключаем всю руку — пул пуст
    let take = crate::use_cases::TakeAcquaintanceReplacementUseCase::new(&repo);
    let none = take.execute(&jlpt_content, &hand).await.unwrap();
    assert!(none.is_none(), "пул вне руки пуст");

    // Исключаем часть руки — возвращается оставшаяся карта
    let (taken, card_type) = take
        .execute(&jlpt_content, &hand[..1])
        .await
        .unwrap()
        .unwrap();
    assert!(
        hand.contains(&taken),
        "замена приходит из руки-исключения только если не исключена"
    );
    assert_eq!(card_type, crate::domain::CardType::Vocabulary);
}

#[tokio::test]
async fn replacement_skips_known_and_phrase_cards() {
    // Arrange: рука-исключение + новая карта, но всё изучено
    let mut user = user_with_new_vocab_cards(1);
    let known_id = *user.knowledge_set().study_cards().keys().next().unwrap();
    user.mark_card_as_known(known_id).unwrap();
    let repo = InMemoryUserRepository::with_user(user);
    let take = crate::use_cases::TakeAcquaintanceReplacementUseCase::new(&repo);
    let jlpt_content = crate::domain::JlptContent::new();

    // Act / Assert: известных новых карт в пуле нет
    assert!(take.execute(&jlpt_content, &[]).await.unwrap().is_none());
    let _ = known_id;
}
