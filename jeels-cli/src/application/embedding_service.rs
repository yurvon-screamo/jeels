use crate::domain::error::JeersError;

pub trait EmbeddingService: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>, JeersError>;
}
