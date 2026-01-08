use async_trait::async_trait;

use crate::domain::error::KeikakuError;

#[async_trait]
pub trait LlmService: Send + Sync {
    async fn generate_text(&self, question: &str) -> Result<String, KeikakuError>;
}
