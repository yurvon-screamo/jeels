use crate::application::{
    CreateVocabularyCardUseCase, LlmService, MigiiClient, MigiiWord, UserRepository,
};
use crate::domain::error::KeikakuError;
use ulid::Ulid;

pub struct ImportMigiiPackResult {
    pub total_created_count: usize,
    pub skipped_words: Vec<String>,
}

pub struct ExportMigiiPackUseCase<'a, R: UserRepository, L: LlmService, M: MigiiClient> {
    repository: &'a R,
    create_card_use_case: CreateVocabularyCardUseCase<'a, R, L>,
    migii_client: &'a M,
}

impl<'a, R: UserRepository, L: LlmService, M: MigiiClient> ExportMigiiPackUseCase<'a, R, L, M> {
    pub fn new(repository: &'a R, llm_service: &'a L, migii_client: &'a M) -> Self {
        Self {
            repository,
            create_card_use_case: CreateVocabularyCardUseCase::new(repository, llm_service),
            migii_client,
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        lessons: Vec<u32>,
    ) -> Result<ImportMigiiPackResult, KeikakuError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(KeikakuError::UserNotFound { user_id })?;

        let mut total_created_count = 0;
        let mut total_skipped_words = Vec::new();

        for lesson in lessons {
            let words = self
                .migii_client
                .get_words(
                    user.native_language(),
                    user.current_japanese_level(),
                    lesson,
                )
                .await?;

            let (created, skipped) = self.process_words(user_id, words).await?;

            total_created_count += created;
            total_skipped_words.extend(skipped);
        }

        Ok(ImportMigiiPackResult {
            total_created_count,
            skipped_words: total_skipped_words,
        })
    }

    async fn process_words(
        &self,
        user_id: Ulid,
        words: Vec<MigiiWord>,
    ) -> Result<(usize, Vec<String>), KeikakuError> {
        let mut created_count = 0;
        let mut skipped_words = Vec::new();

        for word_data in words {
            let question = word_data.word.clone();

            match self
                .create_card_use_case
                .execute(user_id, question.clone())
                .await
            {
                Ok(_) => {
                    created_count += 1;
                }
                Err(KeikakuError::DuplicateCard { .. }) => {
                    skipped_words.push(question);
                }
                Err(e) => {
                    tracing::error!("Failed to create card for word {}: {}", question, e);
                    skipped_words.push(question);
                }
            }
        }

        Ok((created_count, skipped_words))
    }
}
