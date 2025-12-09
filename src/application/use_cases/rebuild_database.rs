use super::generate_card_content::GenerateCardContentUseCase;
use crate::application::{LlmService, UserRepository};
use crate::domain::error::JeersError;
use crate::domain::value_objects::{CardContent, Question};
use clap::ValueEnum;
use tracing::error;
use ulid::Ulid;

#[derive(Clone)]
pub struct RebuildDatabaseUseCase<'a, R: UserRepository, L: LlmService> {
    repository: &'a R,
    generate_content_use_case: GenerateCardContentUseCase<'a, L>,
}

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum RebuildDatabaseOptions {
    Content,
    All,
}

impl<'a, R: UserRepository, L: LlmService> RebuildDatabaseUseCase<'a, R, L> {
    pub fn new(repository: &'a R, llm_service: &'a L) -> Self {
        Self {
            repository,
            generate_content_use_case: GenerateCardContentUseCase::new(llm_service),
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        options: RebuildDatabaseOptions,
    ) -> Result<usize, JeersError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let cards = user.cards().clone();
        let mut data = Vec::new();

        for card in cards.values() {
            let generated_content = if options == RebuildDatabaseOptions::Content
                || options == RebuildDatabaseOptions::All
            {
                match self
                    .generate_content_use_case
                    .generate_content(
                        card.word().text(),
                        user.native_language(),
                        user.current_japanese_level(),
                    )
                    .await
                {
                    Ok(content) => content,
                    Err(e) => {
                        error!("Failed to generate content for card {}: {}", card.id(), e);
                        continue;
                    }
                }
            } else {
                CardContent::new(card.meaning().clone(), card.example_phrases().to_vec())
            };

            let question = Question::new(card.word().text().to_string())?;

            data.push((card.id(), question, generated_content));
        }

        for (card_id, question, new_content) in data {
            let res = user.edit_card(
                card_id,
                question,
                new_content.answer().clone(),
                new_content.example_phrases().to_vec(),
            );

            if let Err(e) = res {
                error!("Failed to edit card {}: {}", card_id, e);
            }
        }

        self.repository.save(&user).await?;
        Ok(cards.len())
    }
}
