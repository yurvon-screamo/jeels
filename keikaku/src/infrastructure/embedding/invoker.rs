use crate::domain::error::JeersError;

use crate::infrastructure::OpenAiEmbeddingService;

pub enum EmbeddingServiceInvoker {
    OpenAi(OpenAiEmbeddingService),
}

impl EmbeddingServiceInvoker {
    pub async fn generate_embedding(
        &self,
        instruction: &str,
        input: &str,
    ) -> Result<Vec<f32>, JeersError> {
        match self {
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
            EmbeddingServiceInvoker::OpenAi(service) => {
                service.generate_embeddings(instruction, inputs).await
            }
        }
    }
}
