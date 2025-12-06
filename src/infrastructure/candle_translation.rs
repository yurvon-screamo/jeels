use std::sync::Arc;

use async_trait::async_trait;
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_t5 as t5;
use hf_hub::{Repo, RepoType, api::sync::Api};
use tokenizers::Tokenizer;
use tokio::sync::Mutex;

use crate::application::translation_service::TranslationService;
use crate::domain::error::JeersError;
use crate::domain::value_objects::NativeLanguage;

const JAPANESE_LANGUAGE: &str = "Japanese";
const MODEL_ID: &str = "lmz/candle-quantized-t5";
const REPEAT_PENALTY: f32 = 1.1;
const REPEAT_LAST_N: usize = 64;
const MAX_GENERATION_LEN: usize = 512;

pub struct CandleTranslationService {
    model: Arc<Mutex<t5::T5ForConditionalGeneration>>,
    tokenizer: Tokenizer,
    device: Device,
    logits_processor: Arc<Mutex<LogitsProcessor>>,
    config: t5::Config,
}

impl CandleTranslationService {
    pub fn new(temperature: f64, seed: u64) -> Result<Self, JeersError> {
        let device = Device::Cpu;
        let repo = Repo::with_revision(MODEL_ID.to_string(), RepoType::Model, "main".to_string());
        let api = Api::new().map_err(|e| JeersError::TranslationError {
            reason: format!("Failed to create HuggingFace API: {}", e),
        })?;
        let api_repo = api.repo(repo);

        let config_filename =
            api_repo
                .get("config.json")
                .map_err(|e| JeersError::TranslationError {
                    reason: format!("Failed to download config.json: {}", e),
                })?;
        let tokenizer_filename =
            api_repo
                .get("tokenizer.json")
                .map_err(|e| JeersError::TranslationError {
                    reason: format!("Failed to download tokenizer.json: {}", e),
                })?;
        let weights_filename =
            api_repo
                .get("model.gguf")
                .map_err(|e| JeersError::TranslationError {
                    reason: format!("Failed to download model.gguf: {}", e),
                })?;

        let config_str =
            std::fs::read_to_string(config_filename).map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to read config file: {}", e),
            })?;
        let mut config: t5::Config =
            serde_json::from_str(&config_str).map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to parse config: {}", e),
            })?;
        config.use_cache = true;

        let mut tokenizer =
            Tokenizer::from_file(tokenizer_filename).map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to load tokenizer: {}", e),
            })?;

        tokenizer
            .with_padding(None)
            .with_truncation(None)
            .map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to configure tokenizer: {}", e),
            })?;

        let vb = t5::VarBuilder::from_gguf(&weights_filename, &device).map_err(|e| {
            JeersError::TranslationError {
                reason: format!("Failed to load model weights: {}", e),
            }
        })?;
        let model = t5::T5ForConditionalGeneration::load(vb, &config).map_err(|e| {
            JeersError::TranslationError {
                reason: format!("Failed to load model: {}", e),
            }
        })?;

        let temperature = if temperature <= 0.0 {
            None
        } else {
            Some(temperature)
        };
        let logits_processor = LogitsProcessor::new(seed, temperature, None);

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
            tokenizer,
            device,
            logits_processor: Arc::new(Mutex::new(logits_processor)),
            config,
        })
    }

    fn create_translation_prompt(text: &str, source_lang: &str, target_lang: &str) -> String {
        format!("translate {} to {}: {}", source_lang, target_lang, text)
    }

    async fn generate_translation(&self, prompt: &str) -> Result<String, JeersError> {
        let tokens =
            self.tokenizer
                .encode(prompt, true)
                .map_err(|e| JeersError::TranslationError {
                    reason: format!("Failed to encode prompt: {}", e),
                })?;
        let input_token_ids: Vec<u32> = tokens.get_ids().to_vec();
        let input_tensor = Tensor::new(input_token_ids.as_slice(), &self.device)
            .map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to create input tensor: {}", e),
            })?
            .unsqueeze(0)
            .map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to unsqueeze input tensor: {}", e),
            })?;

        let mut model = self.model.lock().await;
        let encoder_output =
            model
                .encode(&input_tensor)
                .map_err(|e| JeersError::TranslationError {
                    reason: format!("Failed to encode input: {}", e),
                })?;

        let decoder_start_token_id = self
            .config
            .decoder_start_token_id
            .unwrap_or(self.config.pad_token_id) as u32;
        let mut output_token_ids = vec![decoder_start_token_id];

        for index in 0..MAX_GENERATION_LEN {
            let decoder_token_ids = if index == 0 || !self.config.use_cache {
                Tensor::new(output_token_ids.as_slice(), &self.device)
                    .map_err(|e| JeersError::TranslationError {
                        reason: format!("Failed to create decoder tensor: {}", e),
                    })?
                    .unsqueeze(0)
                    .map_err(|e| JeersError::TranslationError {
                        reason: format!("Failed to unsqueeze decoder tensor: {}", e),
                    })?
            } else {
                let last_token = *output_token_ids.last().unwrap();
                Tensor::new(&[last_token], &self.device)
                    .map_err(|e| JeersError::TranslationError {
                        reason: format!("Failed to create decoder tensor: {}", e),
                    })?
                    .unsqueeze(0)
                    .map_err(|e| JeersError::TranslationError {
                        reason: format!("Failed to unsqueeze decoder tensor: {}", e),
                    })?
            };

            let logits = model
                .decode(&decoder_token_ids, &encoder_output)
                .map_err(|e| JeersError::TranslationError {
                    reason: format!("Failed to decode: {}", e),
                })?
                .squeeze(0)
                .map_err(|e| JeersError::TranslationError {
                    reason: format!("Failed to squeeze logits: {}", e),
                })?;

            let logits = if REPEAT_PENALTY == 1.0 {
                logits
            } else {
                let start_at = output_token_ids.len().saturating_sub(REPEAT_LAST_N);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    REPEAT_PENALTY,
                    &output_token_ids[start_at..],
                )
                .map_err(|e| JeersError::TranslationError {
                    reason: format!("Failed to apply repeat penalty: {}", e),
                })?
            };

            let mut logits_processor = self.logits_processor.lock().await;
            let next_token_id =
                logits_processor
                    .sample(&logits)
                    .map_err(|e| JeersError::TranslationError {
                        reason: format!("Failed to sample token: {}", e),
                    })?;

            if next_token_id as usize == self.config.eos_token_id {
                break;
            }

            output_token_ids.push(next_token_id);
        }

        let output_text = self
            .tokenizer
            .decode(&output_token_ids, true)
            .map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to decode output: {}", e),
            })?;

        Ok(output_text.trim().to_string())
    }
}

#[async_trait]
impl TranslationService for CandleTranslationService {
    async fn translate_to_ja(
        &self,
        text: &str,
        source_language: &NativeLanguage,
    ) -> Result<String, JeersError> {
        let source = format_native_language(source_language);
        let prompt = Self::create_translation_prompt(text, source, JAPANESE_LANGUAGE);
        self.generate_translation(&prompt).await
    }

    async fn translate_from_ja(
        &self,
        text: &str,
        target_language: &NativeLanguage,
    ) -> Result<String, JeersError> {
        let target = format_native_language(target_language);
        let prompt = Self::create_translation_prompt(text, JAPANESE_LANGUAGE, target);
        self.generate_translation(&prompt).await
    }
}

fn format_native_language(native_language: &NativeLanguage) -> &str {
    match native_language {
        NativeLanguage::English => "English",
        NativeLanguage::Russian => "Russian",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEMPERATURE: f64 = 0.8;
    const SEED: u64 = 299792458;

    #[tokio::test]
    async fn translate_to_ja_should_translate_english_to_japanese() {
        let service = init_service();
        let result = service
            .translate_to_ja("Hello, world!", &NativeLanguage::English)
            .await
            .unwrap();
        assert_eq!(result, "こんにちは、世界");
    }

    #[tokio::test]
    async fn translate_from_ja_should_translate_japanese_to_english() {
        let service = init_service();
        let result = service
            .translate_from_ja("こんにちは、世界", &NativeLanguage::English)
            .await
            .unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[tokio::test]
    async fn translate_to_ja_should_translate_russian_to_japanese() {
        let service = init_service();
        let result = service
            .translate_to_ja("Привет, мир!", &NativeLanguage::Russian)
            .await
            .unwrap();
        assert_eq!(result, "こんにちは、世界");
    }

    #[tokio::test]
    async fn translate_from_ja_should_translate_japanese_to_russian() {
        let service = init_service();
        let result = service
            .translate_from_ja("こんにちは、世界", &NativeLanguage::Russian)
            .await
            .unwrap();
        assert_eq!(result, "Привет, мир!");
    }

    fn init_service() -> CandleTranslationService {
        CandleTranslationService::new(TEMPERATURE, SEED).unwrap()
    }
}
