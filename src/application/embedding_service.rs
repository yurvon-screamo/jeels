use async_trait::async_trait;

use crate::domain::error::JeersError;
use crate::domain::value_objects::Embedding;

#[async_trait]
pub trait EmbeddingService: Send + Sync {
    async fn generate_embedding(
        &self,
        instruction: &str,
        input: &str,
    ) -> Result<Embedding, JeersError>;

    async fn generate_embeddings(
        &self,
        instruction: &str,
        inputs: &[String],
    ) -> Result<Vec<Embedding>, JeersError>;
}
