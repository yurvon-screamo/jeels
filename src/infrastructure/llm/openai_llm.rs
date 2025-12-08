use crate::application::LlmService;
use crate::domain::error::JeersError;
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs},
};
use async_trait::async_trait;
use std::sync::Arc;

pub struct OpenAiLlm {
    client: Arc<Client<OpenAIConfig>>,
    model: String,
    temperature: f32,
}

impl OpenAiLlm {
    pub fn new(
        temperature: f32,
        model: String,
        base_url: String,
        env_var_name: String,
    ) -> Result<Self, JeersError> {
        let api_key = std::env::var(&env_var_name).map_err(|_| JeersError::LlmError {
            reason: format!("{} environment variable not set", env_var_name),
        })?;

        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(base_url);

        let client = Client::with_config(config);

        Ok(Self {
            client: Arc::new(client),
            model,
            temperature,
        })
    }

    async fn make_request(&self, prompt: &str) -> Result<String, JeersError> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(vec![ChatCompletionRequestMessage::User(prompt.into())])
            .temperature(self.temperature)
            .build()
            .map_err(|e| JeersError::LlmError {
                reason: format!("Failed to build chat completion request: {}", e),
            })?;

        let response =
            self.client
                .chat()
                .create(request)
                .await
                .map_err(|e| JeersError::LlmError {
                    reason: format!("Failed to send request to LLM: {}", e),
                })?;

        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| JeersError::LlmError {
                reason: "No content in LLM response".to_string(),
            })?;

        Ok(content.clone())
    }
}

#[async_trait]
impl LlmService for OpenAiLlm {
    async fn generate_text(&self, question: &str) -> Result<String, JeersError> {
        self.make_request(question).await
    }
}
