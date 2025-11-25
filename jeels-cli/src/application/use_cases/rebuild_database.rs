use std::collections::HashMap;

use crate::application::{CreateCardUseCase, EmbeddingService, LlmService, UserRepository};
use crate::domain::error::JeersError;
use ulid::Ulid;

#[derive(Clone)]
pub struct RebuildDatabaseUseCase<'a, R: UserRepository, E: EmbeddingService, L: LlmService> {
    repository: &'a R,
    create_card_use_case: CreateCardUseCase<'a, R, E, L>,
}

impl<'a, R: UserRepository, E: EmbeddingService, L: LlmService>
    RebuildDatabaseUseCase<'a, R, E, L>
{
    pub fn new(repository: &'a R, create_card_use_case: CreateCardUseCase<'a, R, E, L>) -> Self {
        Self {
            repository,
            create_card_use_case,
        }
    }

    pub async fn execute(&self, user_id: Ulid) -> Result<usize, JeersError> {
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

            let (new_question, new_answer) = self
                .create_card_use_case
                .generate_translation_and_embedding(question_text)
                .await?;

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
