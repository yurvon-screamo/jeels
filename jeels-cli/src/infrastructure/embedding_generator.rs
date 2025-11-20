use crate::application::EmbeddingService;
use crate::domain::error::JeersError;
use candle_core::{Device, Tensor};
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use hf_hub::{Repo, RepoType, api::sync::Api};
use tokenizers::Tokenizer;

pub struct EmbeddingGenerator {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl EmbeddingGenerator {
    pub fn new() -> Result<Self, JeersError> {
        let device = Device::Cpu;
        let model_id = "sentence-transformers/paraphrase-multilingual-mpnet-base-v2";
        let revision = "main";

        let repo = Repo::with_revision(model_id.to_string(), RepoType::Model, revision.to_string());
        let api = Api::new().map_err(|e| JeersError::EmbeddingError {
            reason: format!("Failed to create HF Hub API: {}", e),
        })?;

        let api = api.repo(repo);

        let config_filename = api
            .get("config.json")
            .map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to download config: {}", e),
            })?;

        let tokenizer_filename =
            api.get("tokenizer.json")
                .map_err(|e| JeersError::EmbeddingError {
                    reason: format!("Failed to download tokenizer: {}", e),
                })?;

        let weights_filename =
            api.get("model.safetensors")
                .map_err(|e| JeersError::EmbeddingError {
                    reason: format!("Failed to download weights: {}", e),
                })?;

        let config_str =
            std::fs::read_to_string(config_filename).map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to read config: {}", e),
            })?;

        let config: Config =
            serde_json::from_str(&config_str).map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to parse config: {}", e),
            })?;

        let tokenizer =
            Tokenizer::from_file(tokenizer_filename).map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to load tokenizer: {}", e),
            })?;

        let vb = unsafe {
            candle_nn::VarBuilder::from_mmaped_safetensors(&[weights_filename], DTYPE, &device)
                .map_err(|e| JeersError::EmbeddingError {
                    reason: format!("Failed to load weights: {}", e),
                })?
        };

        let model = BertModel::load(vb, &config).map_err(|e| JeersError::EmbeddingError {
            reason: format!("Failed to load model: {}", e),
        })?;

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    fn normalize_l2(&self, v: &Tensor) -> Result<Tensor, candle_core::Error> {
        // Compute L2 norm: sqrt(sum of squares) along the last dimension
        // v shape: [batch, dim] -> norm shape: [batch, 1]
        let norm = v.sqr()?.sum_keepdim(1)?.sqrt()?;
        // Add small epsilon to avoid division by zero
        let epsilon = Tensor::new(&[1e-8f32], v.device())?;
        let norm_safe = norm.broadcast_add(&epsilon)?;
        v.broadcast_div(&norm_safe)
    }
}

impl EmbeddingService for EmbeddingGenerator {
    fn embed(&self, text: &str) -> Result<Vec<f32>, JeersError> {
        let mut tokenizer = self.tokenizer.clone();
        let tokenizer = tokenizer
            .with_padding(None)
            .with_truncation(None)
            .map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to configure tokenizer: {}", e),
            })?;

        let tokens = tokenizer
            .encode(text, true)
            .map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to encode text: {}", e),
            })?;

        let token_ids = tokens.get_ids().to_vec();
        let attention_mask = tokens.get_attention_mask().to_vec();

        let token_ids =
            Tensor::new(&token_ids[..], &self.device).map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to create token tensor: {}", e),
            })?;

        let token_ids = token_ids
            .unsqueeze(0)
            .map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to unsqueeze tokens: {}", e),
            })?;

        let attention_mask = Tensor::new(&attention_mask[..], &self.device).map_err(|e| {
            JeersError::EmbeddingError {
                reason: format!("Failed to create attention mask: {}", e),
            }
        })?;

        let attention_mask =
            attention_mask
                .unsqueeze(0)
                .map_err(|e| JeersError::EmbeddingError {
                    reason: format!("Failed to unsqueeze attention mask: {}", e),
                })?;

        let token_type_ids = token_ids
            .zeros_like()
            .map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to create token type ids: {}", e),
            })?;

        let embeddings = self
            .model
            .forward(&token_ids, &token_type_ids, Some(&attention_mask))
            .map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to generate embeddings: {}", e),
            })?;

        // For MPNet models, use mean pooling with attention mask
        // Convert attention mask to proper dtype and shape for broadcasting
        let attention_mask_float =
            attention_mask
                .to_dtype(DTYPE)
                .map_err(|e| JeersError::EmbeddingError {
                    reason: format!("Failed to convert attention mask dtype: {}", e),
                })?;

        // Expand attention mask to match embeddings shape: [batch, seq_len] -> [batch, seq_len, 1]
        let attention_mask_expanded =
            attention_mask_float
                .unsqueeze(2)
                .map_err(|e| JeersError::EmbeddingError {
                    reason: format!("Failed to expand attention mask: {}", e),
                })?;

        // Apply attention mask: multiply embeddings by mask (0 for padding tokens)
        // embeddings shape: [batch, seq_len, hidden_dim]
        // attention_mask_expanded shape: [batch, seq_len, 1]
        let masked_embeddings =
            embeddings
                .broadcast_mul(&attention_mask_expanded)
                .map_err(|e| JeersError::EmbeddingError {
                    reason: format!("Failed to apply attention mask: {}", e),
                })?;

        // Sum embeddings along sequence dimension (dim=1)
        // Result shape: [batch, hidden_dim]
        let sum_embeddings = masked_embeddings
            .sum(1)
            .map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to sum embeddings: {}", e),
            })?;

        // Calculate sum of attention mask for averaging
        // Sum along sequence dimension: [batch, seq_len, 1] -> [batch, 1]
        let sum_mask = attention_mask_expanded
            .sum(1)
            .map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to sum attention mask: {}", e),
            })?;

        // Average pooling: divide by number of non-padding tokens
        // Add small epsilon to avoid division by zero
        let epsilon =
            Tensor::new(&[1e-9f32], &self.device).map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to create epsilon tensor: {}", e),
            })?;
        let sum_mask_safe =
            sum_mask
                .broadcast_add(&epsilon)
                .map_err(|e| JeersError::EmbeddingError {
                    reason: format!("Failed to add epsilon: {}", e),
                })?;

        let pooled = sum_embeddings.broadcast_div(&sum_mask_safe).map_err(|e| {
            JeersError::EmbeddingError {
                reason: format!("Failed to average embeddings: {}", e),
            }
        })?;

        // L2 normalization
        let normalized = self
            .normalize_l2(&pooled)
            .map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to normalize: {}", e),
            })?;

        // Remove batch dimension (squeeze from [1, 384] to [384])
        let normalized = normalized
            .squeeze(0)
            .map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to squeeze tensor: {}", e),
            })?;

        let embedding_vec =
            normalized
                .to_vec1::<f32>()
                .map_err(|e| JeersError::EmbeddingError {
                    reason: format!("Failed to convert tensor to vector: {}", e),
                })?;

        Ok(embedding_vec)
    }
}
