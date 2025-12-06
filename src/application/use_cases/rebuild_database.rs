use crate::application::{CreateCardUseCase, EmbeddingService, LlmService, UserRepository};
use crate::domain::error::JeersError;
use crate::domain::value_objects::{Embedding, Question};
use tracing::error;
use ulid::Ulid;

#[derive(Clone)]
pub struct RebuildDatabaseUseCase<'a, R: UserRepository, E: EmbeddingService, L: LlmService> {
    repository: &'a R,
    embedding_service: &'a E,
    create_card_use_case: CreateCardUseCase<'a, R, E, L>,
}

impl<'a, R: UserRepository, E: EmbeddingService, L: LlmService>
    RebuildDatabaseUseCase<'a, R, E, L>
{
    pub fn new(
        repository: &'a R,
        embedding_service: &'a E,
        create_card_use_case: CreateCardUseCase<'a, R, E, L>,
    ) -> Self {
        Self {
            repository,
            embedding_service,
            create_card_use_case,
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
            let new_example_phrases =
                if rebuild_example_phrases && card.example_phrases().is_empty() {
                    match self
                        .create_card_use_case
                        .generate_example_phrases(
                            card.question().text(),
                            user.native_language(),
                            user.current_japanese_level(),
                        )
                        .await
                    {
                        Ok(value) => value,
                        Err(e) => {
                            error!(
                                "Failed to generate example phrases for card {}: {}",
                                card.id(),
                                e
                            );
                            continue;
                        }
                    }
                } else {
                    card.example_phrases().to_vec()
                };

            let new_answer = if rebuild_answer {
                match self
                    .create_card_use_case
                    .generate_translation(card.question().text(), user.native_language())
                    .await
                {
                    Ok(value) => value,
                    Err(e) => {
                        error!(
                            "Failed to generate translation for card {}: {}",
                            card.id(),
                            e
                        );
                        continue;
                    }
                }
            } else {
                card.answer().clone()
            };

            let new_embedding = if rebuild_embedding {
                match self
                    .embedding_service
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
