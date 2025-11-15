use crate::application::EmbeddingService;
use crate::domain::error::JeersError;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

pub struct EmbeddingGenerator {
    model: TextEmbedding,
}

impl EmbeddingGenerator {
    pub fn new() -> Result<Self, JeersError> {
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::NomicEmbedTextV15Q).with_show_download_progress(true),
        )
        .map_err(|e| JeersError::EmbeddingError {
            reason: format!("Failed to initialize fastembed model: {}", e),
        })?;

        Ok(Self { model })
    }
}

impl EmbeddingService for EmbeddingGenerator {
    fn embed(&mut self, text: &str) -> Result<Vec<f32>, JeersError> {
        let documents = vec![text];
        let embeddings =
            self.model
                .embed(documents, None)
                .map_err(|e| JeersError::EmbeddingError {
                    reason: format!("Failed to generate embedding: {}", e),
                })?;

        if let Some(embedding) = embeddings.first() {
            Ok(embedding.clone())
        } else {
            Err(JeersError::EmbeddingError {
                reason: "No embedding generated".to_string(),
            })
        }
    }
}
