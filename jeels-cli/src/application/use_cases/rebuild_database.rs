use std::collections::HashMap;

use crate::application::{CreateCardUseCase, EmbeddingService, LlmService, UserRepository};
use crate::domain::error::JeersError;
use crate::domain::value_objects::Question;
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

    pub async fn execute(&self, user_id: Ulid, embedding_only: bool) -> Result<usize, JeersError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let cards = user.cards().values();
        let mut new_cards = HashMap::new();
        let mut processed_count = 0;

        for card in cards {
            let question_text = card.question().text();
            let answer = card.answer();

            let embedding = self
                .embedding_service
                .generate_embedding(question_text)
                .await?;

            let new_answer = if embedding_only {
                answer.clone()
            } else {
                self.create_card_use_case
                    .generate_translation(question_text)
                    .await?
            };

            let new_question = Question::new(question_text.to_string(), embedding)?;

            println!(
                "For card {}: generated question embedding and answer {}",
                card.id(),
                new_answer.text(),
            );

            new_cards.insert(card.id(), (new_question, new_answer));
            processed_count += 1;
        }

        for (card_id, (new_question, new_answer)) in new_cards {
            user.edit_card(card_id, new_question, new_answer)?;
        }

        self.repository.save(&user).await?;
        Ok(processed_count)
    }
}
