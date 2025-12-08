use super::generate_embedding::GenerateEmbeddingUseCase;
use crate::application::{EmbeddingService, UserRepository};
use crate::domain::VocabularyCard;
use crate::domain::error::JeersError;
use crate::domain::value_objects::{Answer, ExamplePhrase, Question};
use ulid::Ulid;

#[derive(Clone)]
pub struct EditCardUseCase<'a, R: UserRepository, E: EmbeddingService> {
    repository: &'a R,
    generate_embedding_use_case: GenerateEmbeddingUseCase<'a, E>,
}

impl<'a, R: UserRepository, E: EmbeddingService> EditCardUseCase<'a, R, E> {
    pub fn new(repository: &'a R, embedding_service: &'a E) -> Self {
        Self {
            repository,
            generate_embedding_use_case: GenerateEmbeddingUseCase::new(embedding_service),
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        card_id: Ulid,
        question_text: String,
        answer_text: String,
        example_phrases: Vec<ExamplePhrase>,
    ) -> Result<VocabularyCard, JeersError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let new_embedding = self
            .generate_embedding_use_case
            .generate_embedding(&question_text)
            .await?;
        let new_question = Question::new(question_text, new_embedding)?;
        let new_answer = Answer::new(answer_text)?;

        user.edit_card(card_id, new_question, new_answer, example_phrases)?;

        self.repository.save(&user).await?;

        let card = user
            .get_card(card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        Ok(card.clone())
    }
}
