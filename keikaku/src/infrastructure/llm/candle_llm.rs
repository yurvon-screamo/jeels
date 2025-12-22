use std::sync::Arc;

use crate::application::LlmService;
use crate::domain::error::JeersError;

use async_trait::async_trait;
use tokio::sync::Mutex;

pub struct CandleLlm {
    // model: Arc<Mutex<Qwen3>>,
    // tokenizer: Tokenizer,
    // device: Device,
    // logits_processor: Arc<Mutex<LogitsProcessor>>,
    // eos_token: u32,
    // max_sample_len: usize,
}

impl CandleLlm {
    pub fn new(
        max_sample_len: usize,
        temperature: f32,
        seed: u64,
        model_repo: String,
        model_filename: String,
        model_revision: String,
        tokenizer_repo: String,
        tokenizer_filename: String,
    ) -> Result<Self, JeersError> {
        // let device = Device::Cpu;
        // let model = Self::load_model(&device, model_repo, model_filename, model_revision)?;
        // let tokenizer = Self::load_tokenizer(tokenizer_repo, tokenizer_filename)?;
        // let logits_processor = Self::create_logits_processor(seed, temperature)?;
        // let eos_token = Self::extract_eos_token(&tokenizer)?;

        Ok(Self {
            // model: Arc::new(Mutex::new(model)),
            // tokenizer,
            // device,
            // logits_processor: Arc::new(Mutex::new(logits_processor)),
            // eos_token,
            // max_sample_len,
        })
    }

    // fn extract_eos_token(tokenizer: &Tokenizer) -> Result<u32, JeersError> {
    //     let vocab = tokenizer.get_vocab(true);
    //     let eos_token = vocab.get("<|im_end|>").ok_or(JeersError::LlmError {
    //         reason: "EOS token not found".to_string(),
    //     })?;
    //     Ok(*eos_token)
    // }

    // fn load_model(
    //     device: &Device,
    //     model_repo: String,
    //     model_filename: String,
    //     model_revision: String,
    // ) -> Result<Qwen3, JeersError> {
    //     let model_path = Self::download_model_path(model_repo, model_filename, model_revision)?;
    //     let mut file = Self::open_model_file(&model_path)?;
    //     let gguf_content = Self::read_gguf_content(&mut file, &model_path)?;
    //     Self::load_model_from_gguf(gguf_content, &mut file, device)
    // }

    // fn download_model_path(
    //     model_repo: String,
    //     model_filename: String,
    //     model_revision: String,
    // ) -> Result<std::path::PathBuf, JeersError> {
    //     let api = Self::create_hf_hub_api()?;
    //     let repo = hf_hub::Repo::with_revision(
    //         model_repo.clone(),
    //         hf_hub::RepoType::Model,
    //         model_revision.clone(),
    //     );
    //     api.repo(repo)
    //         .get(&model_filename)
    //         .map_err(|e| JeersError::LlmError {
    //             reason: format!("Failed to get model: {}", e),
    //         })
    // }

    // fn create_hf_hub_api() -> Result<hf_hub::api::sync::Api, JeersError> {
    //     hf_hub::api::sync::Api::new().map_err(|e| JeersError::LlmError {
    //         reason: format!("Failed to create HF Hub API: {}", e),
    //     })
    // }

    // fn open_model_file(path: &std::path::Path) -> Result<std::fs::File, JeersError> {
    //     std::fs::File::open(path).map_err(|e| JeersError::LlmError {
    //         reason: format!("Failed to open model file: {}", e),
    //     })
    // }

    // fn read_gguf_content(
    //     file: &mut std::fs::File,
    //     path: &std::path::Path,
    // ) -> Result<gguf_file::Content, JeersError> {
    //     gguf_file::Content::read(file)
    //         .map_err(|e| e.with_path(path))
    //         .map_err(|e| JeersError::LlmError {
    //             reason: format!("Failed to read model file: {}", e),
    //         })
    // }

    // fn load_model_from_gguf(
    //     content: gguf_file::Content,
    //     file: &mut std::fs::File,
    //     device: &Device,
    // ) -> Result<Qwen3, JeersError> {
    //     Qwen3::from_gguf(content, file, device).map_err(|e| JeersError::LlmError {
    //         reason: format!("Failed to load model: {}", e),
    //     })
    // }

    // fn load_tokenizer(
    //     tokenizer_repo: String,
    //     tokenizer_filename: String,
    // ) -> Result<Tokenizer, JeersError> {
    //     let tokenizer_path = Self::download_tokenizer_path(tokenizer_repo, tokenizer_filename)?;
    //     Self::load_tokenizer_from_file(tokenizer_path)
    // }

    // fn download_tokenizer_path(
    //     tokenizer_repo: String,
    //     tokenizer_filename: String,
    // ) -> Result<std::path::PathBuf, JeersError> {
    //     let api = Self::create_hf_hub_api()?;
    //     api.model(tokenizer_repo.clone())
    //         .get(&tokenizer_filename)
    //         .map_err(|e| JeersError::LlmError {
    //             reason: format!("Failed to get tokenizer: {}", e),
    //         })
    // }

    // fn load_tokenizer_from_file(path: std::path::PathBuf) -> Result<Tokenizer, JeersError> {
    //     Tokenizer::from_file(path).map_err(|e| JeersError::LlmError {
    //         reason: format!("Failed to load tokenizer: {}", e),
    //     })
    // }

    // fn create_logits_processor(seed: u64, temperature: f32) -> Result<LogitsProcessor, JeersError> {
    //     Ok(LogitsProcessor::from_sampling(
    //         seed,
    //         Sampling::All {
    //             temperature: temperature as f64,
    //         },
    //     ))
    // }
}

#[async_trait]
impl LlmService for CandleLlm {
    async fn generate_text(&self, question: &str) -> Result<String, JeersError> {
        // let prompt = Self::format_prompt(question);
        // let input_tokens = Self::encode_prompt(&self.tokenizer, &prompt)?;
        // let first_token = Self::generate_first_token(self, &input_tokens).await?;
        // let all_tokens = Self::generate_remaining_tokens(self, &input_tokens, first_token).await?;
        // let raw_response = Self::decode_tokens(&self.tokenizer, &all_tokens)?;
        // let response = Self::clean_think_tag(&raw_response)?;
        // Ok(response)
        Err(JeersError::LlmError {
            reason: "Not implemented".to_string(),
        })
    }
}

impl CandleLlm {
    // fn format_prompt(question: &str) -> String {
    //     format!("<|im_start|>user\n{question}/no_think<|im_end|>\n<|im_start|>assistant\n")
    // }

    // fn encode_prompt(tokenizer: &Tokenizer, prompt: &str) -> Result<Vec<u32>, JeersError> {
    //     let encoding = tokenizer
    //         .encode(prompt, true)
    //         .map_err(|e| JeersError::LlmError {
    //             reason: format!("Failed to encode prompt: {}", e),
    //         })?;
    //     Ok(encoding.get_ids().to_vec())
    // }

    // fn create_input_tensor(tokens: &[u32], device: &Device) -> Result<Tensor, JeersError> {
    //     Tensor::from_slice(tokens, (tokens.len(),), device)
    //         .map_err(|e| JeersError::LlmError {
    //             reason: format!("Failed to create input tensor: {}", e),
    //         })?
    //         .unsqueeze(0)
    //         .map_err(|e| JeersError::LlmError {
    //             reason: format!("Failed to unsqueeze input tensor: {}", e),
    //         })
    // }

    // async fn forward_model(
    //     model: &Arc<Mutex<Qwen3>>,
    //     input: &Tensor,
    //     pos: usize,
    // ) -> Result<Tensor, JeersError> {
    //     let mut model = model.lock().await;
    //     model.forward(input, pos).map_err(|e| JeersError::LlmError {
    //         reason: format!("Failed to forward input tensor: {}", e),
    //     })
    // }

    // fn squeeze_logits(logits: Tensor) -> Result<Tensor, JeersError> {
    //     logits.squeeze(0).map_err(|e| JeersError::LlmError {
    //         reason: format!("Failed to squeeze logits tensor: {}", e),
    //     })
    // }

    // async fn sample_token(
    //     logits_processor: &Arc<Mutex<LogitsProcessor>>,
    //     logits: &Tensor,
    // ) -> Result<u32, JeersError> {
    //     let mut processor = logits_processor.lock().await;
    //     processor.sample(logits).map_err(|e| JeersError::LlmError {
    //         reason: format!("Failed to sample logits: {}", e),
    //     })
    // }

    // async fn generate_first_token(&self, input_tokens: &[u32]) -> Result<u32, JeersError> {
    //     let input = Self::create_input_tensor(input_tokens, &self.device)?;
    //     let logits = Self::forward_model(&self.model, &input, 0).await?;
    //     let logits = Self::squeeze_logits(logits)?;
    //     Self::sample_token(&self.logits_processor, &logits).await
    // }

    // async fn generate_next_token(&self, token: u32, position: usize) -> Result<u32, JeersError> {
    //     let input = Self::create_input_tensor(&[token], &self.device)?;
    //     let logits = Self::forward_model(&self.model, &input, position).await?;
    //     let logits = Self::squeeze_logits(logits)?;
    //     Self::sample_token(&self.logits_processor, &logits).await
    // }

    // async fn generate_remaining_tokens(
    //     &self,
    //     input_tokens: &[u32],
    //     first_token: u32,
    // ) -> Result<Vec<u32>, JeersError> {
    //     let mut all_tokens = vec![first_token];
    //     let to_sample = self.max_sample_len.saturating_sub(1);

    //     for index in 0..to_sample {
    //         let position = input_tokens.len() + index;
    //         let next_token =
    //             Self::generate_next_token(self, all_tokens[all_tokens.len() - 1], position).await?;
    //         all_tokens.push(next_token);

    //         if next_token == self.eos_token {
    //             break;
    //         }
    //     }

    //     Ok(all_tokens)
    // }

    // fn decode_tokens(tokenizer: &Tokenizer, tokens: &[u32]) -> Result<String, JeersError> {
    //     tokenizer
    //         .decode(tokens, true)
    //         .map_err(|e| JeersError::LlmError {
    //             reason: format!("Failed to decode tokens: {}", e),
    //         })
    // }

    // fn clean_think_tag(response: &str) -> Result<String, JeersError> {
    //     let response = response
    //         .split_once("</think>")
    //         .ok_or(JeersError::LlmError {
    //             reason: "Response does not contain think tag".to_string(),
    //         })?;

    //     Ok(response.1.trim().trim_end_matches('.').to_string())
    // }
}
