use crate::domain::{Card, OrigaError, RateMode, Rating};
use crate::traits::UserRepository;
use crate::use_cases::RateCardUseCase;
use tracing::warn;
use ulid::Ulid;

pub struct RateCardWithSideEffectsUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> RateCardWithSideEffectsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        card_id: Ulid,
        rate_mode: RateMode,
        rating: Rating,
        grammar_rule_id: Option<Ulid>,
    ) -> Result<(), OrigaError> {
        RateCardUseCase::new(self.repository)
            .execute(card_id, rate_mode, rating)
            .await?;

        if let Some(grammar_rule_id) = grammar_rule_id {
            self.handle_grammar_dual_rating(grammar_rule_id, card_id, rating)
                .await;
        }

        Ok(())
    }

    /// Двойной рейтинг грамматики: неявно рейтиится только
    /// карта правила, уже прошедшая знакомство (`!is_new`), и не совпадающая
    /// с только что оценённой — иначе квиз на самой грамматической карте
    /// двигал бы её дважды за один ответ. Новые карты не рейтим: они пройдут
    /// руку знакомства с показом и тренировкой. Карт правил не создаём:
    /// `rule_id` во вью всегда приходит из существующей карты, а молчаливое
    /// создание карты посреди урока — фантомная утечка мимо знакомства.
    async fn handle_grammar_dual_rating(
        &self,
        grammar_rule_id: Ulid,
        rated_card_id: Ulid,
        rating: Rating,
    ) {
        let Some(user) = self.repository.get_current_user().await.ok().flatten() else {
            return;
        };

        let existing = user
            .knowledge_set()
            .study_cards()
            .iter()
            .find(|(card_id, study_card)| {
                if **card_id == rated_card_id {
                    return false;
                }
                let Card::Grammar(grammar_card) = study_card.card() else {
                    return false;
                };
                grammar_card.rule_id() == &grammar_rule_id && !study_card.memory().is_new()
            })
            .map(|(card_id, _)| *card_id);

        let Some(card_id) = existing else {
            return;
        };

        if let Err(e) = RateCardUseCase::new(self.repository)
            .execute(card_id, RateMode::GrammarReview, rating)
            .await
        {
            warn!(error = ?e, "Failed to rate grammar card during dual rating");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Card, GrammarRuleCard, NativeLanguage, Question, User, VocabularyCard};
    use crate::use_cases::tests::fixtures::InMemoryUserRepository;

    fn create_test_user_with_vocab() -> User {
        User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        )
    }

    fn create_vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    #[tokio::test]
    async fn rates_card_without_grammar_dual_rating() {
        let mut user = create_test_user_with_vocab();
        let study_card = user.create_card(create_vocab_card("猫")).unwrap();
        let card_id = *study_card.card_id();
        let repo = InMemoryUserRepository::with_user(user);

        let use_case = RateCardWithSideEffectsUseCase::new(&repo);

        let result = use_case
            .execute(card_id, RateMode::StandardLesson, Rating::Good, None)
            .await;

        assert!(result.is_ok());

        let updated_user = repo.get_current_user().await.unwrap().unwrap();
        let rated_card = updated_user
            .knowledge_set()
            .study_cards()
            .get(&card_id)
            .unwrap();
        assert!(!rated_card.is_new());
    }

    #[tokio::test]
    async fn dual_rating_without_existing_card_creates_nothing() {
        // Arrange: карты правила нет — рейтинг не должен тихо создавать её
        // (молчаливое создание мимо знакомства — фантомная утечка)
        let mut user = create_test_user_with_vocab();
        let study_card = user.create_card(create_vocab_card("猫")).unwrap();
        let card_id = *study_card.card_id();
        let repo = InMemoryUserRepository::with_user(user);

        let use_case = RateCardWithSideEffectsUseCase::new(&repo);

        // Act
        let result = use_case
            .execute(
                card_id,
                RateMode::StandardLesson,
                Rating::Good,
                Some(Ulid::new()),
            )
            .await;

        // Assert: запрос успешен, но грамматика не появилась
        assert!(result.is_ok());
        let updated_user = repo.get_current_user().await.unwrap().unwrap();
        let grammar_count = updated_user
            .knowledge_set()
            .study_cards()
            .values()
            .filter(|sc| matches!(sc.card(), Card::Grammar(_)))
            .count();
        assert_eq!(grammar_count, 0, "карта правила не создаётся рейтингом");
    }

    #[tokio::test]
    async fn dual_rating_skips_new_grammar_card() {
        // Arrange: карта правила существует и ещё новая — не прошла руку
        // знакомства; рейтинг мутированного слова не должен уводить её
        // из «новых»
        let mut user = create_test_user_with_vocab();
        let study_card = user.create_card(create_vocab_card("猫")).unwrap();
        let card_id = *study_card.card_id();

        let grammar_rule_id = Ulid::new();
        let grammar_card = GrammarRuleCard::new_test_with_id(grammar_rule_id);
        let grammar_id = *user
            .create_card(Card::Grammar(grammar_card))
            .unwrap()
            .card_id();

        let repo = InMemoryUserRepository::with_user(user);
        let use_case = RateCardWithSideEffectsUseCase::new(&repo);

        // Act
        let result = use_case
            .execute(
                card_id,
                RateMode::StandardLesson,
                Rating::Good,
                Some(grammar_rule_id),
            )
            .await;

        // Assert: новая грамматика осталась новой, без единого ревью
        assert!(result.is_ok());
        let updated_user = repo.get_current_user().await.unwrap().unwrap();
        let grammar = updated_user
            .knowledge_set()
            .study_cards()
            .get(&grammar_id)
            .unwrap();
        assert!(grammar.is_new(), "новая грамматика ждёт свою руку");
        assert_eq!(grammar.memory().reps(), 0);
    }

    #[tokio::test]
    async fn dual_rating_rates_existing_non_new_grammar_card_once() {
        // Arrange: карта правила уже прошла знакомство — двойной рейтинг
        // записывает повторение ровно один раз
        let mut user = create_test_user_with_vocab();
        let study_card = user.create_card(create_vocab_card("猫")).unwrap();
        let card_id = *study_card.card_id();

        let grammar_rule_id = Ulid::new();
        let grammar_card = GrammarRuleCard::new_test_with_id(grammar_rule_id);
        let grammar_id = *user
            .create_card(Card::Grammar(grammar_card))
            .unwrap()
            .card_id();
        user.rate_card(grammar_id, Rating::Good, RateMode::GrammarReview)
            .unwrap();
        assert!(!user.knowledge_set().get_card(grammar_id).unwrap().is_new());
        let reps_before = user
            .knowledge_set()
            .get_card(grammar_id)
            .unwrap()
            .memory()
            .reps();

        let repo = InMemoryUserRepository::with_user(user);
        let use_case = RateCardWithSideEffectsUseCase::new(&repo);

        // Act
        let result = use_case
            .execute(
                card_id,
                RateMode::StandardLesson,
                Rating::Good,
                Some(grammar_rule_id),
            )
            .await;

        // Assert: грамматика продвинулась ровно на одно повторение
        assert!(result.is_ok());
        let updated_user = repo.get_current_user().await.unwrap().unwrap();
        let reps_after = updated_user
            .knowledge_set()
            .get_card(grammar_id)
            .unwrap()
            .memory()
            .reps();
        assert_eq!(reps_after, reps_before + 1);
    }

    #[tokio::test]
    async fn quiz_on_own_grammar_card_rates_it_exactly_once() {
        // Arrange: квиз на самой грамматической карте — primary-рейтинг уже
        // оценивает её; dual rating не должен оценивать второй раз
        // (регресс: двойное продвижение FSRS за один ответ)
        let mut user = create_test_user_with_vocab();
        let grammar_rule_id = Ulid::new();
        let grammar_card = GrammarRuleCard::new_test_with_id(grammar_rule_id);
        let grammar_id = *user
            .create_card(Card::Grammar(grammar_card))
            .unwrap()
            .card_id();
        user.rate_card(grammar_id, Rating::Good, RateMode::GrammarReview)
            .unwrap();
        let reps_before = user
            .knowledge_set()
            .get_card(grammar_id)
            .unwrap()
            .memory()
            .reps();

        let repo = InMemoryUserRepository::with_user(user);
        let use_case = RateCardWithSideEffectsUseCase::new(&repo);

        // Act: рейтится сама грамматическая карта с её же rule_id
        let result = use_case
            .execute(
                grammar_id,
                RateMode::GrammarReview,
                Rating::Good,
                Some(grammar_rule_id),
            )
            .await;

        // Assert: продвинулась ровно на одно повторение, не на два
        assert!(result.is_ok());
        let updated_user = repo.get_current_user().await.unwrap().unwrap();
        let reps_after = updated_user
            .knowledge_set()
            .get_card(grammar_id)
            .unwrap()
            .memory()
            .reps();
        assert_eq!(reps_after, reps_before + 1);
    }

    #[tokio::test]
    async fn returns_error_for_nonexistent_card() {
        let user = create_test_user_with_vocab();
        let repo = InMemoryUserRepository::with_user(user);

        let use_case = RateCardWithSideEffectsUseCase::new(&repo);

        let result = use_case
            .execute(Ulid::new(), RateMode::StandardLesson, Rating::Good, None)
            .await;

        assert!(result.is_err());
    }
}
