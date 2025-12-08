use super::generate_card_content::GenerateCardContentUseCase;
use super::generate_embedding::GenerateEmbeddingUseCase;
use crate::application::{EmbeddingService, LlmService, UserRepository};
use crate::domain::error::JeersError;
use crate::domain::value_objects::{Embedding, Question};
use tracing::error;
use ulid::Ulid;

#[derive(Clone)]
pub struct RebuildDatabaseUseCase<'a, R: UserRepository, E: EmbeddingService, L: LlmService> {
    repository: &'a R,
    generate_embedding_use_case: GenerateEmbeddingUseCase<'a, E>,
    generate_content_use_case: GenerateCardContentUseCase<'a, L>,
}

impl<'a, R: UserRepository, E: EmbeddingService, L: LlmService>
    RebuildDatabaseUseCase<'a, R, E, L>
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
        rebuild_example_phrases: bool,
        rebuild_embedding: bool,
        rebuild_answer: bool,
    ) -> Result<usize, JeersError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let cards = user.cards().clone();
        let mut data = Vec::new();

        for card in cards.values() {
            let needs_generation =
                (rebuild_example_phrases && card.example_phrases().is_empty()) || rebuild_answer;

            let (generated_answer, generated_examples) = if needs_generation {
                match self
                    .generate_content_use_case
                    .generate_content(
                        card.question().text(),
                        user.native_language(),
                        user.current_japanese_level(),
                    )
                    .await
                {
                    Ok((answer, examples)) => (Some(answer), Some(examples)),
                    Err(e) => {
                        error!("Failed to generate content for card {}: {}", card.id(), e);
                        continue;
                    }
                }
            } else {
                (None, None)
            };

            let new_example_phrases =
                if rebuild_example_phrases && card.example_phrases().is_empty() {
                    generated_examples.unwrap_or_else(|| card.example_phrases().to_vec())
                } else {
                    card.example_phrases().to_vec()
                };

            let new_answer = if rebuild_answer {
                generated_answer.unwrap_or_else(|| card.answer().clone())
            } else {
                card.answer().clone()
            };

            let new_embedding = if rebuild_embedding {
                match self
                    .generate_embedding_use_case
                    .generate_embedding(card.question().text())
                    .await
                {
                    Ok(value) => value,
                    Err(e) => {
                        error!("Failed to generate embedding for card {}: {}", card.id(), e);
                        continue;
                    }
                }
            } else {
                Embedding(card.question().embedding().clone())
            };

            let new_question =
                match Question::new(card.question().text().to_string(), new_embedding) {
                    Ok(x) => x,
                    Err(e) => {
                        error!("Failed to create question for card {}: {}", card.id(), e);
                        continue;
                    }
                };

            println!(
                "For card {}: generated question embedding and answer {} and example phrases {}",
                card.id(),
                new_answer.text(),
                new_example_phrases
                    .iter()
                    .map(|x| format!("{}: {}", x.text(), x.translation()))
                    .collect::<Vec<String>>()
                    .join(", "),
            );

            data.push((card.id(), new_question, new_answer, new_example_phrases));
        }

        for (card_id, new_question, new_answer, new_example_phrases) in data {
            let res = user.edit_card(card_id, new_question, new_answer, new_example_phrases);
            if let Err(e) = res {
                error!("Failed to edit card {}: {}", card_id, e);
            }
        }

        self.repository.save(&user).await?;
        Ok(cards.len())
    }
}
