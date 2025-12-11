use crate::application::LlmService;
use crate::domain::error::JeersError;

use super::{CandleLlm, GeminiLlm, OpenAiLlm};

pub enum LlmServiceInvoker {
    Candle(CandleLlm),
    OpenAi(OpenAiLlm),
    Gemini(GeminiLlm),
}

#[async_trait::async_trait]
impl LlmService for LlmServiceInvoker {
    async fn generate_text(&self, question: &str) -> Result<String, JeersError> {
        match self {
            LlmServiceInvoker::Candle(service) => service.generate_text(question).await,
            LlmServiceInvoker::OpenAi(service) => service.generate_text(question).await,
            LlmServiceInvoker::Gemini(service) => service.generate_text(question).await,
        }
    }
}
