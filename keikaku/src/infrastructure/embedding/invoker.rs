use crate::domain::error::JeersError;

use crate::infrastructure::{CandleEmbeddingService, OpenAiEmbeddingService};

pub enum EmbeddingServiceInvoker {
    Candle(CandleEmbeddingService),
    OpenAi(OpenAiEmbeddingService),
}

impl EmbeddingServiceInvoker {
    pub async fn generate_embedding(
        &self,
        instruction: &str,
        input: &str,
    ) -> Result<Vec<f32>, JeersError> {
        match self {
            EmbeddingServiceInvoker::Candle(service) => {
                service.generate_embedding(instruction, input).await
            }
            EmbeddingServiceInvoker::OpenAi(service) => {
                service.generate_embedding(instruction, input).await
            }
        }
    }

    pub async fn generate_embeddings(
        &self,
        instruction: &str,
        inputs: &[String],
    ) -> Result<Vec<Vec<f32>>, JeersError> {
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
