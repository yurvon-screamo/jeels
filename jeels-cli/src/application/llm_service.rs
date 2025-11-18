use crate::domain::error::JeersError;

pub trait LlmService: Send + Sync {
    fn generate_text(
        &self,
        question: &str,
    ) -> impl Future<Output = Result<String, JeersError>> + Send;
}
