use crate::application::{TranslationService, UserRepository};
use crate::domain::error::JeersError;
use crate::domain::japanese::IsJapaneseText;
use ulid::Ulid;

pub struct TranslateUseCase<'a, R: UserRepository, T: TranslationService> {
    repository: &'a R,
    translation_service: &'a T,
}

impl<'a, R: UserRepository, T: TranslationService> TranslateUseCase<'a, R, T> {
    pub fn new(repository: &'a R, translation_service: &'a T) -> Self {
        Self {
            repository,
            translation_service,
        }
    }

    pub async fn execute(&self, user_id: Ulid, text: String) -> Result<String, JeersError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let native_language = user.native_language();

        if text.contains_japanese() {
            self.translation_service
                .translate_from_ja(&text, native_language)
                .await
        } else {
            self.translation_service
                .translate_to_ja(&text, native_language)
                .await
        }
    }
}
