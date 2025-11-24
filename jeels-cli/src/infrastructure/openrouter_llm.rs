use crate::application::LlmService;
use crate::domain::error::JeersError;
use crate::settings::LlmSettings;
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs},
};
use std::sync::Arc;

pub struct OpenRouterLlm {
    client: Arc<Client<OpenAIConfig>>,
    model: String,
    temperature: f32,
}

impl OpenRouterLlm {
    pub fn new(settings: &LlmSettings) -> Result<Self, JeersError> {
        let api_key = std::env::var("OPENROUTER_API_KEY").map_err(|_| JeersError::LlmError {
            reason: "OPENROUTER_API_KEY environment variable not set".to_string(),
        })?;

        let model = "qwen/qwen3-30b-a3b:free".to_string();

        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base("https://openrouter.ai/api/v1");

        let client = Client::with_config(config);

        Ok(Self {
            client: Arc::new(client),
            model,
            temperature: settings.temperature as f32,
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
                    reason: format!("Failed to send request to OpenRouter: {}", e),
                })?;

        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| JeersError::LlmError {
                reason: "No content in OpenRouter response".to_string(),
            })?;

        Ok(content.clone())
    }
}

impl LlmService for OpenRouterLlm {
    async fn generate_text(&self, question: &str) -> Result<String, JeersError> {
        self.make_request(question).await
    }
}
