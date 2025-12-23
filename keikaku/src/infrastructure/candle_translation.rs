// #[cfg(feature = "mkl")]
// extern crate intel_mkl_src;
// #[cfg(feature = "accelerate")]
// extern crate accelerate_src;

use std::sync::Arc;

use candle_transformers::models::quantized_t5 as t5;

use candle_transformers::generation::LogitsProcessor;
use hf_hub::{Repo, RepoType, api::sync::Api};
use rand::Rng;
use tokenizers::Tokenizer;
use tokio::sync::Mutex;

use crate::application::translation_service::TranslationService;
use crate::domain::error::JeersError;
use crate::domain::value_objects::NativeLanguage;
use async_trait::async_trait;
use candle_core::{Device, Tensor};

pub struct CandleTranslationService {
    model: Arc<Mutex<t5::T5ForConditionalGeneration>>,
    tokenizer: Tokenizer,
    config: t5::Config,
    device: Device,
}

impl CandleTranslationService {
    pub fn new() -> Result<Self, JeersError> {
        let device = Device::Cpu;
        let model_id = "jbochi/madlad400-3b-mt";
        let revision = "main";

        let repo = Repo::with_revision(model_id.to_string(), RepoType::Model, revision.to_string());
        let api = Api::new()
            .map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to create API: {}", e),
            })?
            .repo(repo);
        let config_filename = api
            .get("config.json")
            .map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to get config: {}", e),
            })?;
        let tokenizer_filename =
            api.get("tokenizer.json")
                .map_err(|e| JeersError::TranslationError {
                    reason: format!("Failed to get tokenizer: {}", e),
                })?;
        let weights_filename =
            api.get("model-q4k.gguf")
                .map_err(|e| JeersError::TranslationError {
                    reason: format!("Failed to get weights: {}", e),
                })?;
        let config_str =
            std::fs::read_to_string(config_filename).map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to read config: {}", e),
            })?;
        let config: t5::Config =
            serde_json::from_str(&config_str).map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to parse config: {}", e),
            })?;
        let vb = t5::VarBuilder::from_gguf(&weights_filename, &device).map_err(|e| {
            JeersError::TranslationError {
                reason: format!("Failed to load weights: {}", e),
            }
        })?;

        Ok(Self {
            model: Arc::new(Mutex::new(
                t5::T5ForConditionalGeneration::load(vb, &config).map_err(|e| {
                    JeersError::TranslationError {
                        reason: format!("Failed to load model: {}", e),
                    }
                })?,
            )),
            tokenizer: Tokenizer::from_file(tokenizer_filename).map_err(|e| {
                JeersError::TranslationError {
                    reason: format!("Failed to load tokenizer: {}", e),
                }
            })?,
            config,
            device,
        })
    }
}

#[async_trait]
impl TranslationService for CandleTranslationService {
    async fn translate_to_ja(
        &self,
        text: &str,
        _source_language: &NativeLanguage,
    ) -> Result<String, JeersError> {
        self.translate(text, "ja")
            .await
            .map_err(|e| JeersError::TranslationError {
                reason: e.to_string(),
            })
    }

    async fn translate_from_ja(
        &self,
        text: &str,
        target_language: &NativeLanguage,
    ) -> Result<String, JeersError> {
        let target_language = match target_language {
            NativeLanguage::English => "en",
            NativeLanguage::Russian => "ru",
        };

        self.translate(text, target_language)
            .await
            .map_err(|e| JeersError::TranslationError {
                reason: e.to_string(),
            })
    }
}

impl CandleTranslationService {
    fn create_logits_processor(&self) -> Result<LogitsProcessor, JeersError> {
        let seed = rand::rng().random_range(0..=u64::MAX);
        Ok(LogitsProcessor::new(seed, None, None))
    }

    fn create_prompt(&self, text: &str, target_language: &str) -> Result<String, JeersError> {
        Ok(format!("<2{target_language}> {text}"))
    }

    fn tokenize_prompt(&self, prompt: &str) -> Result<Vec<u32>, JeersError> {
        Ok(self
            .tokenizer
            .encode(prompt, true)
            .map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to encode prompt: {}", e),
            })?
            .get_ids()
            .to_vec())
    }

    async fn encode_input(&self, tokens: &[u32]) -> Result<Tensor, JeersError> {
        let input_token_ids = Tensor::new(&tokens[..], &self.device)
            .and_then(|x| x.unsqueeze(0))
            .map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to unsqueeze input token ids: {}", e),
            })?;

        self.model
            .lock()
            .await
            .encode(&input_token_ids)
            .map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to encode input token ids: {}", e),
            })
    }

    async fn decode_output(
        &self,
        decoder_token_ids: &Tensor,
        encoder_output: &Tensor,
    ) -> Result<Tensor, JeersError> {
        self.model
            .lock()
            .await
            .decode(&decoder_token_ids, &encoder_output)
            .and_then(|x| x.squeeze(0))
            .map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to squeeze logits: {}", e),
            })
    }

    async fn translate(&self, text: &str, target_language: &str) -> Result<String, JeersError> {
        let mut logits_processor = self.create_logits_processor()?;

        let promt = self.create_prompt(text, target_language)?;
        let tokens = self.tokenize_prompt(&promt)?;
        let encoder_output = self.encode_input(&tokens).await?;

        let mut output_token_ids = [self
            .config
            .decoder_start_token_id
            .unwrap_or(self.config.pad_token_id) as u32]
        .to_vec();

        let mut translation_result = String::new();

        for index in 0.. {
            let decoder_token_ids = if index == 0 || !self.config.use_cache {
                Tensor::new(output_token_ids.as_slice(), &self.device).and_then(|x| x.unsqueeze(0))
            } else {
                let last_token = *output_token_ids
                    .last()
                    .ok_or(JeersError::TranslationError {
                        reason: format!("Failed to get last token"),
                    })?;
                Tensor::new(&[last_token], &self.device).and_then(|x| x.unsqueeze(0))
            }
            .map_err(|e| JeersError::TranslationError {
                reason: format!("Failed to unsqueeze decoder token ids: {}", e),
            })?;

            let logits = self
                .decode_output(&decoder_token_ids, &encoder_output)
                .await?;

            let next_token_id =
                logits_processor
                    .sample(&logits)
                    .map_err(|e| JeersError::TranslationError {
                        reason: format!("Failed to sample logits: {}", e),
                    })?;

            if next_token_id as usize == self.config.eos_token_id {
                break;
            }

            output_token_ids.push(next_token_id);

            if let Some(text) = self.tokenizer.id_to_token(next_token_id) {
                let text = text.replace('▁', " ").replace("<0x0A>", "\n");
                translation_result.push_str(&text);
            }
        }

        Ok(translation_result)
    }
}
