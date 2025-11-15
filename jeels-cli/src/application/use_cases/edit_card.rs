use crate::application::{EmbeddingService, UserRepository};
use crate::domain::error::JeersError;
use crate::domain::value_objects::{Answer, Question};
use ulid::Ulid;

#[derive(Clone)]
pub struct EditCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> EditCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute<E: EmbeddingService>(
        &self,
        embedding_service: &mut E,
        user_id: Ulid,
        card_id: Ulid,
        question_text: String,
        answer_text: String,
    ) -> Result<(), JeersError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let new_embedding = embedding_service.embed(&question_text)?;
        let new_question = Question::new(question_text, new_embedding)?;
        let new_answer = Answer::new(answer_text)?;

        user.edit_card(card_id, new_question, new_answer)?;

        self.repository.save(&user).await?;
        Ok(())
    }
}
