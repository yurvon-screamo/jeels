use crate::application::{CreateVocabularyCardUseCase, LlmService, UserRepository};
use crate::domain::{dictionary::JLPT_DB, error::KeikakuError, value_objects::JapaneseLevel};
use ulid::Ulid;

pub struct ImportJlptRecommendedResult {
    pub total_created_count: usize,
    pub skipped_words: Vec<String>,
}

pub struct ExportJlptRecommendedUseCase<'a, R: UserRepository, L: LlmService> {
    create_card_use_case: CreateVocabularyCardUseCase<'a, R, L>,
}

impl<'a, R: UserRepository, L: LlmService> ExportJlptRecommendedUseCase<'a, R, L> {
    pub fn new(repository: &'a R, llm_service: &'a L) -> Self {
        Self {
            create_card_use_case: CreateVocabularyCardUseCase::new(repository, llm_service),
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        levels: Vec<JapaneseLevel>,
    ) -> Result<ImportJlptRecommendedResult, KeikakuError> {
        let mut total_created_count = 0;
        let mut total_skipped_words = Vec::new();

        for level in levels {
            let words = JLPT_DB.get_words_for_level(&level);
            let (created, skipped) = self.process_words(user_id, words).await?;

            total_created_count += created;
            total_skipped_words.extend(skipped);
        }

        Ok(ImportJlptRecommendedResult {
            total_created_count,
            skipped_words: total_skipped_words,
        })
    }

    async fn process_words(
        &self,
        user_id: Ulid,
        words: Vec<String>,
    ) -> Result<(usize, Vec<String>), KeikakuError> {
        let mut created_count = 0;
        let mut skipped_words = Vec::new();

        for word in words {
            let question = word.clone();

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
