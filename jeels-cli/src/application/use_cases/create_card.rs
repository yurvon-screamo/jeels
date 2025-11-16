use crate::application::{EmbeddingService, UserRepository};
use crate::domain::Card;
use crate::domain::error::JeersError;
use crate::domain::value_objects::{Answer, Question};
use ulid::Ulid;

#[derive(Clone)]
pub struct CreateCardUseCase<'a, R: UserRepository, E: EmbeddingService> {
    repository: &'a R,
    embedding_service: &'a E,
}

impl<'a, R: UserRepository, E: EmbeddingService> CreateCardUseCase<'a, R, E> {
    pub fn new(repository: &'a R, embedding_service: &'a E) -> Self {
        Self {
            repository,
            embedding_service,
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        question_text: String,
        answer_text: String,
    ) -> Result<Card, JeersError> {
        let embedding = self.embedding_service.embed(&question_text)?;

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
