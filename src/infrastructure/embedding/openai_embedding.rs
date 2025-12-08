use crate::application::embedding_service::EmbeddingService;
use crate::domain::error::JeersError;
use crate::domain::value_objects::Embedding;
use async_openai::{Client, config::OpenAIConfig, types::CreateEmbeddingRequestArgs};
use async_trait::async_trait;
use std::sync::Arc;

pub struct OpenAiEmbeddingService {
    client: Arc<Client<OpenAIConfig>>,
    model: String,
}

impl OpenAiEmbeddingService {
    pub fn new(model: String, base_url: String, env_var_name: String) -> Result<Self, JeersError> {
        let api_key = std::env::var(&env_var_name).map_err(|_| JeersError::EmbeddingError {
            reason: format!("{} environment variable not set", env_var_name),
        })?;

        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(base_url);

        let client = Client::with_config(config);

        Ok(Self {
            client: Arc::new(client),
            model,
        })
    }

    fn create_instruction(&self, instruction: &str, word: &str) -> String {
        format!("Instruct: {}\nQuery: {}", instruction, word)
    }
}

#[async_trait]
impl EmbeddingService for OpenAiEmbeddingService {
    async fn generate_embedding(
        &self,
        instruction: &str,
        input: &str,
    ) -> Result<Embedding, JeersError> {
        let input = self.create_instruction(instruction, input);

        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.model)
            .input(vec![input])
            .build()
            .map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to build embedding request: {}", e),
            })?;

        let response = self
            .client
            .embeddings()
            .create(request)
            .await
            .map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to generate embedding: {}", e),
            })?;

        let embedding_vec = response
            .data
            .first()
            .ok_or_else(|| JeersError::EmbeddingError {
                reason: "No embedding data in response".to_string(),
            })?
            .embedding
            .clone();

        Ok(Embedding(embedding_vec))
    }

    async fn generate_embeddings(
        &self,
        instruction: &str,
        inputs: &[String],
    ) -> Result<Vec<Embedding>, JeersError> {
        let inputs = inputs
            .iter()
            .map(|s| self.create_instruction(instruction, s))
            .collect::<Vec<_>>();

        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.model)
            .input(inputs)
            .build()
            .map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to build embedding request: {}", e),
            })?;

        let response = self
            .client
            .embeddings()
            .create(request)
            .await
            .map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to generate embeddings: {}", e),
            })?;

        let embeddings: Result<Vec<Embedding>, JeersError> = response
            .data
            .into_iter()
            .map(|item| Ok(Embedding(item.embedding)))
            .collect();

        embeddings
    }
}
