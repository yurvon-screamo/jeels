use crate::application::{
    CreateCardUseCase, DuolingoClient, EmbeddingService, LlmService, UserRepository,
};
use crate::domain::error::JeersError;
use ulid::Ulid;

pub struct SyncDuolingoWordsResult {
    pub total_created_count: usize,
    pub skipped_words: Vec<String>,
}

pub struct SyncDuolingoWordsUseCase<
    'a,
    R: UserRepository,
    E: EmbeddingService,
    L: LlmService,
    D: DuolingoClient,
> {
    repository: &'a R,
    create_card_use_case: CreateCardUseCase<'a, R, E, L>,
    duolingo_client: &'a D,
}

impl<'a, R: UserRepository, E: EmbeddingService, L: LlmService, D: DuolingoClient>
    SyncDuolingoWordsUseCase<'a, R, E, L, D>
{
    pub fn new(
        repository: &'a R,
        embedding_service: &'a E,
        llm_service: &'a L,
        duolingo_client: &'a D,
    ) -> Self {
        Self {
            repository,
            create_card_use_case: CreateCardUseCase::new(
                repository,
                embedding_service,
                llm_service,
            ),
            duolingo_client,
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        question_only: bool,
    ) -> Result<SyncDuolingoWordsResult, JeersError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let jwt_token = user
            .duolingo_jwt_token()
            .ok_or_else(|| JeersError::RepositoryError {
                reason: "Duolingo JWT token not set".to_string(),
            })?;

        let words = self.duolingo_client.get_words(jwt_token).await?;

        let mut total_created_count = 0;
        let mut skipped_words = Vec::new();

        for word in words {
            let question = word.text.clone();
            let answer = Self::extract_answer_from_word(&word, question_only);

            if answer.is_none() && !question_only {
                continue;
            }

            match self
                .create_card_use_case
                .execute(user_id, question.clone(), answer, None)
                .await
            {
                Ok(_) => {
                    total_created_count += 1;
                }
                Err(JeersError::DuplicateCard { .. }) => {
                    skipped_words.push(question);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(SyncDuolingoWordsResult {
            total_created_count,
            skipped_words,
        })
    }

    fn extract_answer_from_word(word: &crate::application::DuolingoWord, question_only: bool) -> Option<String> {
        if question_only {
            return None;
        }

        word.translations
            .first()
            .map(|t| t.clone())
            .filter(|t| !t.is_empty())
    }
}
