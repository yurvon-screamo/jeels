use crate::application::{EmbeddingService, LlmService, UserRepository};
use crate::domain::Card;
use crate::domain::error::JeersError;
use crate::domain::value_objects::{Answer, Question};
use ulid::Ulid;

#[derive(Clone)]
pub struct CreateCardUseCase<'a, R: UserRepository, E: EmbeddingService, L: LlmService> {
    repository: &'a R,
    embedding_service: &'a E,
    llm_service: &'a L,
}

impl<'a, R: UserRepository, E: EmbeddingService, L: LlmService> CreateCardUseCase<'a, R, E, L> {
    pub fn new(repository: &'a R, embedding_service: &'a E, llm_service: &'a L) -> Self {
        Self {
            repository,
            embedding_service,
            llm_service,
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        question_text: String,
        answer_text: Option<String>,
    ) -> Result<Card, JeersError> {
        let embedding = self
            .embedding_service
            .generate_embedding(&question_text)
            .await?;

        let question = Question::new(question_text.clone(), embedding)?;

        let answer_text = if let Some(answer_text) = answer_text {
            answer_text
        } else {
            self.llm_service.generate_text(&format!(
                "Объясни значение этого слова для рускоговорящего студента: '{}'. Ответь 1 предложением.",
                question_text
            )).await?.trim_matches(&['\n', '\r', '»', '«', '.', '"', ' ']).to_string()
        };

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
