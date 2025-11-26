use crate::domain::error::JeersError;
use crate::domain::value_objects::Embedding;

use std::future::Future;

pub trait EmbeddingService: Send + Sync {
    fn generate_embedding(
        &self,
        input: &str,
    ) -> impl Future<Output = Result<Embedding, JeersError>> + Send;

    fn generate_embeddings(
        &self,
        inputs: &[String],
    ) -> impl Future<Output = Result<Vec<Embedding>, JeersError>> + Send;
}
