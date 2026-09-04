use rstest::rstest;

use crate::domain::NewCardPolicy;
use crate::domain::User;
use crate::domain::value_objects::Question;
use crate::domain::{
    Card, CardAnswer, JlptContent, NativeLanguage, RateMode, Rating, VocabularyCard, YesNoCard,
};
use crate::traits::UserRepository;
use crate::use_cases::tests::fixtures::InMemoryUserRepository;
use crate::use_cases::{RateCardUseCase, SelectCardsToLessonUseCase};

fn answer_display_text(answer: &CardAnswer) -> String {
    answer.text_projection()
}

fn create_user_with_real_vocab_cards(words: &[&str]) -> User {
    let mut user = User::new(
        "test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );

    for word in words {
        let card = Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).expect("Failed to create Question"),
        ));
        user.create_card(card).expect("Failed to create card");
    }

    user
}

fn create_yesno_card_with_words(
    card: Card,
    word: String,
    statement: String,
    is_correct: bool,
) -> YesNoCard {
    YesNoCard::new(card, word, statement, is_correct)
}

#[tokio::test]
async fn yesno_journey_correct_answer_results_in_good_rating() {
    crate::use_cases::init_real_dictionaries();

    // Arrange: создаём пользователя с реальными японскими словами
    let words = vec!["猫", "犬", "鳥", "魚", "馬"];
    let user = create_user_with_real_vocab_cards(&words);
    let repo = InMemoryUserRepository::with_user(user);

    // Получаем карточки для урока
    let use_case = SelectCardsToLessonUseCase::new(&repo);
    let cards = use_case
        .execute(NewCardPolicy::Inject, &JlptContent::new())
        .await
        .unwrap();

    // Берём первую карточку
    let (card_id, first_card_view) = cards.iter().next().unwrap();
    let card = first_card_view.card().clone();

    // Создаём YesNo с верным утверждением (is_correct = true)
    let question = card.question(&NativeLanguage::Russian).unwrap();
    let answer = card.answer(&NativeLanguage::Russian).unwrap();
    let word = question.text().to_string();
    let statement = answer_display_text(&answer);
    let yesno = create_yesno_card_with_words(card.clone(), word, statement, true);

    // Act: пользователь отвечает "Да" на верное утверждение
    let user_said_yes = true;
    let is_answer_correct = yesno.check_answer(user_said_yes);
    assert!(is_answer_correct, "Ответ должен быть правильным");

    // Правильный ответ → Rating::Good
    let rating = if is_answer_correct {
        Rating::Good
    } else {
        Rating::Again
    };

    // Рейтинг карточки
    let rate_use_case = RateCardUseCase::new(&repo);
    rate_use_case
        .execute(*card_id, RateMode::StandardLesson, rating)
        .await
        .unwrap();

    // Assert: карточка получила рейтинг и больше не является новой
    let updated = repo.get_current_user().await.unwrap().unwrap();
    let study_card = updated.knowledge_set().get_card(*card_id).unwrap();
    assert!(
        !study_card.memory().is_new(),
        "Карточка не должна быть новой после рейтинга"
    );
}

#[tokio::test]
async fn yesno_journey_wrong_answer_results_in_again_rating() {
    crate::use_cases::init_real_dictionaries();

    // Arrange: создаём пользователя с реальными японскими словами
    let words = vec!["猫", "犬", "鳥", "魚", "馬"];
    let user = create_user_with_real_vocab_cards(&words);
    let repo = InMemoryUserRepository::with_user(user);

    // Получаем карточки для урока
    let use_case = SelectCardsToLessonUseCase::new(&repo);
    let cards = use_case
        .execute(NewCardPolicy::Inject, &JlptContent::new())
        .await
        .unwrap();

    // Берём первую карточку
    let (card_id, first_card_view) = cards.iter().next().unwrap();
    let card = first_card_view.card().clone();

    // Создаём YesNo с верным утверждением (is_correct = true)
    let question = card.question(&NativeLanguage::Russian).unwrap();
    let answer = card.answer(&NativeLanguage::Russian).unwrap();
    let word = question.text().to_string();
    let statement = answer_display_text(&answer);
    let yesno = create_yesno_card_with_words(card.clone(), word, statement, true);

    // Act: пользователь отвечает "Нет" на верное утверждение (неправильно)
    let user_said_yes = false;
    let is_answer_correct = yesno.check_answer(user_said_yes);
    assert!(!is_answer_correct, "Ответ должен быть неправильным");

    // Неправильный ответ → Rating::Again
    let rating = if is_answer_correct {
        Rating::Good
    } else {
        Rating::Again
    };

    // Рейтинг карточки
    let rate_use_case = RateCardUseCase::new(&repo);
    rate_use_case
        .execute(*card_id, RateMode::StandardLesson, rating)
        .await
        .unwrap();

    // Assert: карточка получила рейтинг Again
    let updated = repo.get_current_user().await.unwrap().unwrap();
    let study_card = updated.knowledge_set().get_card(*card_id).unwrap();
    assert!(
        !study_card.memory().is_new(),
        "Карточка не должна быть новой после рейтинга"
    );
    assert!(
        study_card.memory().is_high_difficulty(),
        "Карточка должна быть с высокой сложностью после Again рейтинга"
    );
}

#[tokio::test]
async fn yesno_journey_false_statement_correct_no_answer() {
    // Arrange: Тестируем случай когда утверждение ложно и пользователь отвечает "Нет"
    crate::use_cases::init_real_dictionaries();

    let card = VocabularyCard::new(Question::new("猫".to_string()).unwrap());
    let question = Card::Vocabulary(card.clone())
        .question(&NativeLanguage::Russian)
        .unwrap();
    let distractor_answer = "собака"; // Неверный ответ (дистрактор)
    let word = question.text().to_string();
    let yesno = create_yesno_card_with_words(
        Card::Vocabulary(card),
        word,
        distractor_answer.to_string(),
        false, // is_correct = false
    );

    // Act: Пользователь отвечает "Нет" на ложное утверждение
    let user_said_yes = false;
    let is_correct = yesno.check_answer(user_said_yes);

    // Assert: Это правильный ответ (ложное утверждение + "Нет")
    assert!(
        is_correct,
        "\"Нет\" на ложное утверждение должно быть правильным ответом"
    );
}

#[tokio::test]
async fn yesno_journey_false_statement_wrong_yes_answer() {
    // Arrange: Тестируем случай когда утверждение ложно и пользователь ошибается
    crate::use_cases::init_real_dictionaries();

    let card = VocabularyCard::new(Question::new("猫".to_string()).unwrap());
    let question = Card::Vocabulary(card.clone())
        .question(&NativeLanguage::Russian)
        .unwrap();
    let distractor_answer = "собака"; // Неверный ответ
    let word = question.text().to_string();
    let yesno = create_yesno_card_with_words(
        Card::Vocabulary(card),
        word,
        distractor_answer.to_string(),
        false, // is_correct = false
    );

    // Act: Пользователь отвечает "Да" на ложное утверждение
    let user_said_yes = true;
    let is_correct = yesno.check_answer(user_said_yes);

    // Assert: Это неправильный ответ (ложное утверждение + "Да")
    assert!(
        !is_correct,
        "\"Да\" на ложное утверждение должно быть неправильным ответом"
    );
}

#[tokio::test]
async fn yesno_journey_transition_to_next_card_after_rating() {
    crate::use_cases::init_real_dictionaries();

    // Arrange: создаём пользователя с несколькими карточками
    let words = vec!["猫", "犬", "鳥", "魚", "馬"];
    let user = create_user_with_real_vocab_cards(&words);
    let repo = InMemoryUserRepository::with_user(user);

    // Получаем карточки для урока
    let select_use_case = SelectCardsToLessonUseCase::new(&repo);
    let cards = select_use_case
        .execute(NewCardPolicy::Inject, &JlptContent::new())
        .await
        .unwrap();

    assert!(cards.len() >= 3, "Должно быть минимум 3 карточки для теста");

    let card_ids: Vec<_> = cards.keys().cloned().collect();
    let first_card_id = card_ids[0];

    // Act: Отвечаем на первую карточку
    let rate_use_case = RateCardUseCase::new(&repo);
    rate_use_case
        .execute(first_card_id, RateMode::StandardLesson, Rating::Good)
        .await
        .unwrap();

    // Assert: Проверяем историю уроков
    let updated_user = repo.get_current_user().await.unwrap().unwrap();
    assert!(
        !updated_user.knowledge_set().lesson_history().is_empty(),
        "История уроков не должна быть пустой"
    );
}

#[rstest]
#[case::correct_yes_on_true_statement(true, true, Rating::Good)]
#[case::wrong_no_on_true_statement(false, true, Rating::Again)]
#[case::correct_no_on_false_statement(false, false, Rating::Good)]
#[case::wrong_yes_on_false_statement(true, false, Rating::Again)]
#[tokio::test]
async fn yesno_journey_all_rating_cases(
    #[case] user_said_yes: bool,
    #[case] statement_is_correct: bool,
    #[case] expected_rating: Rating,
) {
    crate::use_cases::init_real_dictionaries();

    // Arrange: создаём пользователя с реальными японскими словами
    let words = vec!["猫", "犬", "鳥", "魚", "馬"];
    let user = create_user_with_real_vocab_cards(&words);
    let repo = InMemoryUserRepository::with_user(user);

    let use_case = SelectCardsToLessonUseCase::new(&repo);
    let cards = use_case
        .execute(NewCardPolicy::Inject, &JlptContent::new())
        .await
        .unwrap();
    let (card_id, first_card_view) = cards.iter().next().unwrap();
    let card = first_card_view.card().clone();

    // Создаём YesNo карточку с нужным значением is_correct
    let question = card.question(&NativeLanguage::Russian).unwrap();
    let answer = card.answer(&NativeLanguage::Russian).unwrap();
    let word = question.text().to_string();
    let statement = answer_display_text(&answer);
    let yesno = create_yesno_card_with_words(card, word, statement, statement_is_correct);

    // Act: Проверяем ответ и определяем рейтинг
    let is_answer_correct = yesno.check_answer(user_said_yes);
    let actual_rating = if is_answer_correct {
        Rating::Good
    } else {
        Rating::Again
    };

    assert_eq!(
        actual_rating, expected_rating,
        "Рейтинг должен соответствовать: user_said_yes={}, statement_is_correct={}",
        user_said_yes, statement_is_correct
    );

    // Рейтинг карточки
    let rate_use_case = RateCardUseCase::new(&repo);
    rate_use_case
        .execute(*card_id, RateMode::StandardLesson, actual_rating)
        .await
        .unwrap();

    // Assert: карточка обработана успешно
    let updated = repo.get_current_user().await.unwrap().unwrap();
    assert!(
        updated.knowledge_set().get_card(*card_id).is_some(),
        "Карточка должна существовать"
    );
}
