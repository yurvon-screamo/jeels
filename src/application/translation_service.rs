use crate::domain::value_objects::NativeLanguage;
use async_trait::async_trait;

use crate::domain::error::JeersError;

#[async_trait]
pub trait TranslationService: Send + Sync {
    async fn translate_to_ja(
        &self,
        text: &str,
        source_language: &NativeLanguage,
    ) -> Result<String, JeersError>;

    async fn translate_from_ja(
        &self,
        text: &str,
        target_language: &NativeLanguage,
    ) -> Result<String, JeersError>;
}
