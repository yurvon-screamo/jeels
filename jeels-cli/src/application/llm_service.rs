use crate::domain::error::JeersError;

pub trait LlmService: Send + Sync {
    fn generate_answer(&mut self, question: &str) -> Result<String, JeersError>;
}
