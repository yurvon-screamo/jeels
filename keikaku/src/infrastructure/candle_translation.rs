use crate::application::translation_service::TranslationService;
use crate::domain::error::JeersError;
use crate::domain::value_objects::NativeLanguage;
use async_trait::async_trait;

pub struct CandleTranslationService {}

#[async_trait]
impl TranslationService for CandleTranslationService {
    async fn translate_to_ja(
        &self,
        _text: &str,
        _source_language: &NativeLanguage,
    ) -> Result<String, JeersError> {
        Err(JeersError::TranslationError {
            reason: "Not implemented".to_string(),
        })
    }

    async fn translate_from_ja(
        &self,
        _text: &str,
        _target_language: &NativeLanguage,
    ) -> Result<String, JeersError> {
        Err(JeersError::TranslationError {
            reason: "Not implemented".to_string(),
        })
    }
}
