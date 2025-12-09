use super::generate_card_content::GenerateCardContentUseCase;
use super::generate_embedding::GenerateEmbeddingUseCase;
use crate::application::{EmbeddingService, UserRepository};
use crate::domain::VocabularyCard;
use crate::domain::error::JeersError;
use crate::domain::value_objects::{CardContent, Question};
use ulid::Ulid;

#[derive(Clone)]
pub struct CreateCardUseCase<
    'a,
    R: UserRepository,
    E: EmbeddingService,
    L: crate::application::LlmService,
> {
    repository: &'a R,
    generate_embedding_use_case: GenerateEmbeddingUseCase<'a, E>,
    generate_content_use_case: GenerateCardContentUseCase<'a, L>,
}

impl<'a, R: UserRepository, E: EmbeddingService, L: crate::application::LlmService>
    CreateCardUseCase<'a, R, E, L>
{
    pub fn new(repository: &'a R, embedding_service: &'a E, llm_service: &'a L) -> Self {
        Self {
            repository,
            generate_embedding_use_case: GenerateEmbeddingUseCase::new(embedding_service),
            generate_content_use_case: GenerateCardContentUseCase::new(llm_service),
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        question_text: String,
        content: Option<CardContent>,
    ) -> Result<VocabularyCard, JeersError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        if user
            .cards()
            .values()
            .any(|card| card.word().text() == question_text)
        {
            return Err(JeersError::DuplicateCard {
                question: question_text,
            });
        }

        let embedding = self
            .generate_embedding_use_case
            .generate_embedding(&question_text)
            .await?;

        let card_content = if let Some(content) = content {
            content
        } else {
            self.generate_content_use_case
                .generate_content(
                    question_text.as_str(),
                    user.native_language(),
                    user.current_japanese_level(),
                )
                .await?
        };

        let question = Question::new(question_text, embedding)?;

        let card = user.create_card(question, card_content)?;
        self.repository.save(&user).await?;

        Ok(card)
    }
}
