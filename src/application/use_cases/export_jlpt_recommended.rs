use crate::application::{CreateCardUseCase, LlmService, UserRepository};
use crate::domain::{
    dictionary::{JLPT_DB, VOCABULARY_DB},
    error::JeersError,
    value_objects::{Answer, CardContent, ExamplePhrase, JapaneseLevel, NativeLanguage},
};
use ulid::Ulid;

pub struct ExportJlptRecommendedResult {
    pub total_created_count: usize,
    pub skipped_words: Vec<String>,
}

pub struct ExportJlptRecommendedUseCase<'a, R: UserRepository, L: LlmService> {
    repository: &'a R,
    create_card_use_case: CreateCardUseCase<'a, R, L>,
}

impl<'a, R: UserRepository, L: LlmService> ExportJlptRecommendedUseCase<'a, R, L> {
    pub fn new(repository: &'a R, llm_service: &'a L) -> Self {
        Self {
            repository,
            create_card_use_case: CreateCardUseCase::new(repository, llm_service),
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        levels: Vec<JapaneseLevel>,
    ) -> Result<ExportJlptRecommendedResult, JeersError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let mut total_created_count = 0;
        let mut total_skipped_words = Vec::new();

        for level in levels {
            let words = JLPT_DB.get_words_for_level(&level);
            let (created, skipped) = self
                .process_words(user_id, user.native_language(), words)
                .await?;

            total_created_count += created;
            total_skipped_words.extend(skipped);
        }

        Ok(ExportJlptRecommendedResult {
            total_created_count,
            skipped_words: total_skipped_words,
        })
    }

    async fn process_words(
        &self,
        user_id: Ulid,
        native_language: &NativeLanguage,
        words: Vec<String>,
    ) -> Result<(usize, Vec<String>), JeersError> {
        let mut created_count = 0;
        let mut skipped_words = Vec::new();

        for word in words {
            let question = word.clone();
            let content = self.build_content(&question, native_language)?;

            match self
                .create_card_use_case
                .execute(user_id, question.clone(), content)
                .await
            {
                Ok(_) => {
                    created_count += 1;
                }
                Err(JeersError::DuplicateCard { .. }) => {
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

    fn build_content(
        &self,
        word: &str,
        native_language: &NativeLanguage,
    ) -> Result<Option<CardContent>, JeersError> {
        if let Some(info) = VOCABULARY_DB.get_vocabulary_info(word) {
            let (translation, examples): (String, Vec<ExamplePhrase>) = match native_language {
                NativeLanguage::Russian => (
                    info.russian_translation().to_string(),
                    info.russian_examples().to_vec(),
                ),
                NativeLanguage::English => (
                    info.english_translation().to_string(),
                    info.english_examples().to_vec(),
                ),
            };

            let answer = Answer::new(translation)?;
            return Ok(Some(CardContent::new(answer, examples)));
        }

        Ok(None)
    }
}
