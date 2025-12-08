use async_trait::async_trait;

use crate::application::embedding_service::EmbeddingService;
use crate::domain::error::JeersError;
use crate::domain::value_objects::Embedding;

use crate::infrastructure::{CandleEmbeddingService, OpenAiEmbeddingService};

pub enum EmbeddingServiceInvoker {
    Candle(CandleEmbeddingService),
    OpenAi(OpenAiEmbeddingService),
}

#[async_trait]
impl EmbeddingService for EmbeddingServiceInvoker {
    async fn generate_embedding(
        &self,
        instruction: &str,
        input: &str,
    ) -> Result<Embedding, JeersError> {
        match self {
            EmbeddingServiceInvoker::Candle(service) => {
                service.generate_embedding(instruction, input).await
            }
            EmbeddingServiceInvoker::OpenAi(service) => {
                service.generate_embedding(instruction, input).await
            }
        }
    }

    async fn generate_embeddings(
        &self,
        instruction: &str,
        inputs: &[String],
    ) -> Result<Vec<Embedding>, JeersError> {
        match self {
            EmbeddingServiceInvoker::Candle(service) => {
                service.generate_embeddings(instruction, inputs).await
            }
            EmbeddingServiceInvoker::OpenAi(service) => {
                service.generate_embeddings(instruction, inputs).await
            }
        }
    }
}
