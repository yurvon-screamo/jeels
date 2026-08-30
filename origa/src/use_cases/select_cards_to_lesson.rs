use crate::domain::{DailyBudget, JlptContent, LessonData, NewCardPolicy, OrigaError};
use crate::traits::UserRepository;
use tracing::{debug, info};

#[derive(Clone)]
pub struct SelectCardsToLessonUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> SelectCardsToLessonUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        new_card_policy: NewCardPolicy,
        jlpt_content: &JlptContent,
    ) -> Result<LessonData, OrigaError> {
        let user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        debug!(user_id = %user.id(), "Selecting cards to lesson");

        let budget = DailyBudget::from_load(*user.daily_load());
        let user_level = user.current_japanese_level();
        let native_language = *user.native_language();
        let lesson_data = user.knowledge_set().cards_to_lesson_with_policy(
            budget,
            jlpt_content,
            user_level,
            native_language,
            new_card_policy,
        );

        info!(user_id = %user.id(), count = lesson_data.total_count(), "Cards selected for lesson");

        Ok(lesson_data)
    }
}
