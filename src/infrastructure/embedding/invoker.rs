use crate::application::embedding_service::EmbeddingService;
use crate::domain::error::JeersError;
use crate::domain::value_objects::Embedding;

use crate::infrastructure::{CandleEmbeddingService, OpenAiEmbeddingService};

pub enum EmbeddingServiceInvoker {
    Candle(CandleEmbeddingService),
    OpenAi(OpenAiEmbeddingService),
}

impl EmbeddingService for EmbeddingServiceInvoker {
    async fn generate_embedding(&self, input: &str) -> Result<Embedding, JeersError> {
        match self {
            EmbeddingServiceInvoker::Candle(service) => service.generate_embedding(input).await,
            EmbeddingServiceInvoker::OpenAi(service) => service.generate_embedding(input).await,
        }
    }

    async fn generate_embeddings(&self, inputs: &[String]) -> Result<Vec<Embedding>, JeersError> {
        match self {
            EmbeddingServiceInvoker::Candle(service) => service.generate_embeddings(inputs).await,
            EmbeddingServiceInvoker::OpenAi(service) => service.generate_embeddings(inputs).await,
        }
    }
}
