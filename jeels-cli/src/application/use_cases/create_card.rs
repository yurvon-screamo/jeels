use crate::application::{EmbeddingService, UserRepository};
use crate::domain::Card;
use crate::domain::error::JeersError;
use crate::domain::value_objects::{Answer, Question};
use ulid::Ulid;

#[derive(Clone)]
pub struct CreateCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CreateCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute<E: EmbeddingService>(
        &self,
        embedding_service: &mut E,
        user_id: Ulid,
        question_text: String,
        answer_text: String,
    ) -> Result<Card, JeersError> {
        let embedding = embedding_service.embed(&question_text)?;

        let question = Question::new(question_text.clone(), embedding)?;
        let answer = Answer::new(answer_text)?;

        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let card = user.create_card(question, answer)?;
        self.repository.save(&user).await?;

        Ok(card)
    }
}
