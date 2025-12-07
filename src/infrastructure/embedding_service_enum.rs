use crate::application::embedding_service::EmbeddingService;
use crate::domain::error::JeersError;
use crate::domain::value_objects::Embedding;

use crate::infrastructure::{CandleEmbeddingService, OpenAiEmbeddingService};

pub enum EmbeddingServiceEnum {
    Candle(CandleEmbeddingService),
    OpenAi(OpenAiEmbeddingService),
}

impl EmbeddingService for EmbeddingServiceEnum {
    async fn generate_embedding(&self, input: &str) -> Result<Embedding, JeersError> {
        match self {
            EmbeddingServiceEnum::Candle(service) => service.generate_embedding(input).await,
            EmbeddingServiceEnum::OpenAi(service) => service.generate_embedding(input).await,
        }
    }

    async fn generate_embeddings(&self, inputs: &[String]) -> Result<Vec<Embedding>, JeersError> {
        match self {
            EmbeddingServiceEnum::Candle(service) => service.generate_embeddings(inputs).await,
            EmbeddingServiceEnum::OpenAi(service) => service.generate_embeddings(inputs).await,
        }
    }
}
