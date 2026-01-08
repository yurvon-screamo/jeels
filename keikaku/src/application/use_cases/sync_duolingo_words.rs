use crate::application::{CreateVocabularyCardUseCase, DuolingoClient, LlmService, UserRepository};
use crate::domain::error::KeikakuError;
use ulid::Ulid;

pub struct SyncDuolingoWordsResult {
    pub total_created_count: usize,
    pub skipped_words: Vec<String>,
}

pub struct SyncDuolingoWordsUseCase<'a, R: UserRepository, L: LlmService, D: DuolingoClient> {
    repository: &'a R,
    create_card_use_case: CreateVocabularyCardUseCase<'a, R, L>,
    duolingo_client: &'a D,
}

impl<'a, R: UserRepository, L: LlmService, D: DuolingoClient>
    SyncDuolingoWordsUseCase<'a, R, L, D>
{
    pub fn new(repository: &'a R, llm_service: &'a L, duolingo_client: &'a D) -> Self {
        Self {
            repository,
            create_card_use_case: CreateVocabularyCardUseCase::new(repository, llm_service),
            duolingo_client,
        }
    }

    pub async fn execute(&self, user_id: Ulid) -> Result<SyncDuolingoWordsResult, KeikakuError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(KeikakuError::UserNotFound { user_id })?;

        let jwt_token =
            user.settings()
                .duolingo_jwt_token()
                .ok_or_else(|| KeikakuError::RepositoryError {
                    reason: "Duolingo JWT token not set".to_string(),
                })?;

        let words = self.duolingo_client.get_words(jwt_token).await?;

        let mut total_created_count = 0;
        let mut skipped_words = Vec::new();

        for word in words {
            let question = word.text.clone();

            match self
                .create_card_use_case
                .execute(user_id, question.clone())
                .await
            {
                Ok(_) => {
                    total_created_count += 1;
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

        Ok(SyncDuolingoWordsResult {
            total_created_count,
            skipped_words,
        })
    }
}
