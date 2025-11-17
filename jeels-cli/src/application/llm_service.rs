use crate::domain::error::JeersError;

pub trait LlmService: Send + Sync {
    fn generate_answer(
        &self,
        question: &str,
    ) -> impl Future<Output = Result<String, JeersError>> + Send;
}
