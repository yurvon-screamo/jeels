use crate::domain::error::JeersError;

pub trait LlmService {
    fn generate_answer(&mut self, question: &str) -> Result<String, JeersError>;
}
