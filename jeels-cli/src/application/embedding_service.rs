use crate::domain::error::JeersError;

pub trait EmbeddingService {
    fn embed(&mut self, text: &str) -> Result<Vec<f32>, JeersError>;
}
