use crate::application::UserRepository;
use crate::domain::error::JeersError;
use crate::domain::{EmbeddingSettings, LearnSettings, LlmSettings, TranslationSettings};
use ulid::Ulid;

#[derive(Clone)]
pub struct UpdateUserSettingsUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

#[derive(Clone)]
pub struct UpdateUserSettingsRequest {
    pub llm: Option<LlmSettings>,
    pub embedding: Option<EmbeddingSettings>,
    pub translation: Option<TranslationSettings>,
    pub duolingo_jwt_token: Option<Option<String>>,
    pub learn: Option<LearnSettings>,
}

impl<'a, R: UserRepository> UpdateUserSettingsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        request: UpdateUserSettingsRequest,
    ) -> Result<(), JeersError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let settings = user.settings_mut();

        if let Some(llm) = request.llm {
            settings.set_llm(llm);
        }

        if let Some(embedding) = request.embedding {
            settings.set_embedding(embedding);
        }

        if let Some(translation) = request.translation {
            settings.set_translation(translation);
        }

        if let Some(duolingo_jwt_token) = request.duolingo_jwt_token {
            settings.set_duolingo_jwt_token(duolingo_jwt_token);
        }

        if let Some(learn) = request.learn {
            settings.set_learn(learn);
        }

        self.repository.save(&user).await?;
        Ok(())
    }
}
