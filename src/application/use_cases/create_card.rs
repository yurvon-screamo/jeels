use super::generate_card_content::GenerateCardContentUseCase;
use crate::application::{EmbeddingService, UserRepository};
use crate::domain::VocabularyCard;
use crate::domain::error::JeersError;
use crate::domain::value_objects::{Answer, ExamplePhrase, Question};
use crate::domain::vocabulary::VOCABULARY_DB;
use ulid::Ulid;

#[derive(Clone, Debug)]
pub struct CardContent {
    pub answer: Answer,
    pub example_phrases: Vec<ExamplePhrase>,
}

#[derive(Clone)]
pub struct CreateCardUseCase<
    'a,
    R: UserRepository,
    E: EmbeddingService,
    L: crate::application::LlmService,
> {
    repository: &'a R,
    embedding_service: &'a E,
    generate_content_use_case: GenerateCardContentUseCase<'a, L>,
}

impl<'a, R: UserRepository, E: EmbeddingService, L: crate::application::LlmService>
    CreateCardUseCase<'a, R, E, L>
{
    pub fn new(repository: &'a R, embedding_service: &'a E, llm_service: &'a L) -> Self {
        Self {
            repository,
            embedding_service,
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
            .any(|card| card.question().text() == question_text)
        {
            return Err(JeersError::DuplicateCard {
                question: question_text,
            });
        }

        let embedding = if let Some(embedding) = VOCABULARY_DB.get_embedding(&question_text) {
            embedding
        } else {
            self.embedding_service
                .generate_embedding(&question_text)
                .await?
        };

        let (answer, example_phrases) = if let Some(content) = content {
            (content.answer, content.example_phrases)
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

        let card = user.create_card(question, answer, example_phrases)?;
        self.repository.save(&user).await?;

        Ok(card)
    }
}
