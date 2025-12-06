use crate::application::LlmService;
use crate::domain::error::JeersError;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::env;

pub struct GeminiLlm {
    client: reqwest::Client,
    api_key: String,
    model: String,
    temperature: f32,
}

impl GeminiLlm {
    pub fn new(temperature: f32, model: String) -> Result<Self, JeersError> {
        let api_key = env::var("GEMINI_API_KEY").map_err(|_| JeersError::LlmError {
            reason: "GEMINI_API_KEY environment variable not set".to_string(),
        })?;

        let client = reqwest::Client::new();

        Ok(Self {
            client,
            api_key,
            model,
            temperature,
        })
    }

    async fn make_request(&self, prompt: &str) -> Result<String, JeersError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.model
        );

        let request_body = json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }],
            "generationConfig": {
                "temperature": self.temperature
            }
        });

        let response = self
            .client
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| JeersError::LlmError {
                reason: format!("Failed to send request to Gemini: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(JeersError::LlmError {
                reason: format!("Gemini API error ({}): {}", status, error_text),
            });
        }

        let response_json: Value = response.json().await.map_err(|e| JeersError::LlmError {
            reason: format!("Failed to parse Gemini response JSON: {}", e),
        })?;

        let content = response_json["candidates"]
            .get(0)
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|parts| parts.get(0))
            .and_then(|part| part.get("text"))
            .and_then(|text| text.as_str())
            .ok_or_else(|| JeersError::LlmError {
                reason: "No content or unexpected format in Gemini response".to_string(),
            })?;

        Ok(content.to_string())
    }
}

#[async_trait]
impl LlmService for GeminiLlm {
    async fn generate_text(&self, question: &str) -> Result<String, JeersError> {
        self.make_request(question).await
    }
}
