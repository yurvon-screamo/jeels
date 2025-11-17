use crate::application::LlmService;

pub struct GenerateAnswerUseCase;

impl Default for GenerateAnswerUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl GenerateAnswerUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute<L: LlmService>(
        &self,
        llm_service: &L,
        question: String,
    ) -> Result<String, crate::domain::error::JeersError> {
        if question.trim().is_empty() {
            return Err(crate::domain::error::JeersError::InvalidQuestion {
                reason: "Question cannot be empty".to_string(),
            });
        }

        llm_service.generate_answer(&question).await
    }
}
