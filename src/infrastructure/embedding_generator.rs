use hf_hub::api::sync::{ApiBuilder, ApiRepo};
use hf_hub::{Repo, RepoType};
use serde::Deserialize;
use std::cmp::max;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokenizers::{Encoding, Tokenizer};
use tokio::sync::Mutex;

use text_embeddings_backend_candle::CandleBackend;
use text_embeddings_backend_core::{Backend, Batch, Embedding, Embeddings, ModelType, Pool};

use crate::application::embedding_service::EmbeddingService;
use crate::domain::error::JeersError;
use crate::domain::value_objects::Embedding as TraitEmbedding;

const MODEL_ID: &str = "Qwen/Qwen3-Embedding-0.6B";

pub struct CandleEmbeddingService {
    backend: Arc<Mutex<CandleBackend>>,
    tokenizer: Tokenizer,
}

impl CandleEmbeddingService {
    pub fn new() -> Result<Self, JeersError> {
        let (model_root, _) = download_artifacts(MODEL_ID, None, None)?;
        let tokenizer = load_tokenizer(&model_root)?;

        let backend = CandleBackend::new(
            &model_root,
            "float32".to_string(),
            ModelType::Embedding(Pool::LastToken),
            None,
        )
        .map_err(|e| JeersError::EmbeddingError {
            reason: format!("Failed to create CandleBackend: {}", e),
        })?;

        Ok(Self {
            backend: Arc::new(Mutex::new(backend)),
            tokenizer,
        })
    }

    fn encode_inputs(&self, inputs: &[String]) -> Result<Vec<Encoding>, JeersError> {
        inputs
            .iter()
            .map(|input| {
                let instruction = self.create_instruction(input);
                self.tokenizer
                    .encode(instruction.as_str(), true)
                    .map_err(|e| JeersError::EmbeddingError {
                        reason: format!("Failed to encode input: {}", e),
                    })
            })
            .collect()
    }

    fn create_batch(&self, encodings: Vec<Encoding>) -> Batch {
        let pooled_indices: Vec<u32> = (0..encodings.len() as u32).collect();
        batch(encodings, pooled_indices, vec![])
    }

    fn process_embeddings(
        &self,
        embeddings: Embeddings,
    ) -> Result<Vec<TraitEmbedding>, JeersError> {
        let pooled_embeddings = sort_embeddings(embeddings);
        Ok(pooled_embeddings)
    }

    fn create_instruction(&self, word: &str) -> String {
        format!(
            "Instruct: Retrieve Japanese words that belong to the same semantic category.\nQuery: {}",
            word
        )
    }
}

impl EmbeddingService for CandleEmbeddingService {
    async fn generate_embedding(&self, input: &str) -> Result<TraitEmbedding, JeersError> {
        let encodings = self.encode_inputs(&[input.to_string()])?;
        let batch = self.create_batch(encodings);
        let embeddings = {
            let backend = self.backend.lock().await;
            backend
                .embed(batch)
                .map_err(|e| JeersError::EmbeddingError {
                    reason: format!("Failed to generate embedding: {}", e),
                })?
        };

        let pooled_embeddings = self.process_embeddings(embeddings)?;
        if let Some(embedding) = pooled_embeddings.into_iter().next() {
            return Ok(embedding);
        }

        Err(JeersError::EmbeddingError {
            reason: "No embedding found".to_string(),
        })
    }

    async fn generate_embeddings(
        &self,
        inputs: &[String],
    ) -> Result<Vec<TraitEmbedding>, JeersError> {
        let encodings = self.encode_inputs(inputs)?;
        let batch = self.create_batch(encodings);
        let embeddings = {
            let backend = self.backend.lock().await;
            backend
                .embed(batch)
                .map_err(|e| JeersError::EmbeddingError {
                    reason: format!("Failed to generate embeddings: {}", e),
                })?
        };
        self.process_embeddings(embeddings)
    }
}

fn sort_embeddings(embeddings: Embeddings) -> Vec<TraitEmbedding> {
    let mut pooled_embeddings = Vec::new();

    for (_, embedding) in embeddings {
        match embedding {
            Embedding::Pooled(e) => pooled_embeddings.push(TraitEmbedding(e)),
            Embedding::All(_) => {}
        }
    }

    pooled_embeddings
}

pub fn download_artifacts(
    model_id: &'static str,
    revision: Option<&'static str>,
    dense_path: Option<&'static str>,
) -> Result<(PathBuf, Option<Vec<String>>), JeersError> {
    let api_repo = create_api_repo(model_id, revision)?;

    api_repo
        .get("config.json")
        .map_err(|e| JeersError::EmbeddingError {
            reason: format!("Failed to get config.json: {}", e),
        })?;
    api_repo
        .get("tokenizer.json")
        .map_err(|e| JeersError::EmbeddingError {
            reason: format!("Failed to get tokenizer.json: {}", e),
        })?;

    let model_files = download_model_files(&api_repo)?;
    let dense_paths = download_dense_modules(&api_repo, dense_path)?;

    let first_file = model_files
        .first()
        .ok_or_else(|| JeersError::EmbeddingError {
            reason: "No model files downloaded".to_string(),
        })?;
    let model_root = first_file
        .parent()
        .ok_or_else(|| JeersError::EmbeddingError {
            reason: "Model file has no parent directory".to_string(),
        })?
        .to_path_buf();
    Ok((model_root, dense_paths))
}

fn create_api_repo(
    model_id: &'static str,
    revision: Option<&'static str>,
) -> Result<ApiRepo, JeersError> {
    let mut builder = ApiBuilder::from_env().with_progress(false);

    if let Ok(token) = std::env::var("HF_TOKEN") {
        builder = builder.with_token(Some(token));
    }

    if let Some(cache_dir) = std::env::var_os("HUGGINGFACE_HUB_CACHE") {
        builder = builder.with_cache_dir(cache_dir.into());
    }

    let api = builder.build().map_err(|e| JeersError::EmbeddingError {
        reason: format!("Failed to build API: {}", e),
    })?;
    let api_repo = if let Some(revision) = revision {
        api.repo(Repo::with_revision(
            model_id.to_string(),
            RepoType::Model,
            revision.to_string(),
        ))
    } else {
        api.repo(Repo::new(model_id.to_string(), RepoType::Model))
    };

    Ok(api_repo)
}

fn download_model_files(api: &ApiRepo) -> Result<Vec<PathBuf>, JeersError> {
    match download_safetensors(api) {
        Ok(files) => Ok(files),
        Err(_) => {
            tracing::warn!(
                "safetensors weights not found. Using `pytorch_model.bin` instead. Model loading will be significantly slower."
            );
            tracing::info!("Downloading `pytorch_model.bin`");
            let file = api
                .get("pytorch_model.bin")
                .map_err(|e| JeersError::EmbeddingError {
                    reason: format!("Failed to get pytorch_model.bin: {}", e),
                })?;
            Ok(vec![file])
        }
    }
}

fn download_dense_modules(
    api: &ApiRepo,
    dense_path: Option<&'static str>,
) -> Result<Option<Vec<String>>, JeersError> {
    let modules_path = match api.get("modules.json") {
        Ok(path) => path,
        Err(_) => return Ok(None),
    };

    let paths = parse_dense_paths_from_modules(&modules_path)?;
    match paths.len() {
        0 => Ok(None),
        1 => {
            let path = dense_path.unwrap_or(&paths[0]).to_string();
            download_dense_module(api, &path)?;
            Ok(Some(vec![path]))
        }
        _ => {
            for path in &paths {
                download_dense_module(api, path)?;
            }
            Ok(Some(paths))
        }
    }
}

fn download_safetensors(api: &ApiRepo) -> Result<Vec<PathBuf>, JeersError> {
    tracing::info!("Downloading `model.safetensors`");
    match api.get("model.safetensors") {
        Ok(file) => return Ok(vec![file]),
        Err(err) => tracing::warn!("Could not download `model.safetensors`: {}", err),
    };

    download_sharded_safetensors(api)
}

fn download_sharded_safetensors(api: &ApiRepo) -> Result<Vec<PathBuf>, JeersError> {
    tracing::info!("Downloading `model.safetensors.index.json`");
    let index_file =
        api.get("model.safetensors.index.json")
            .map_err(|e| JeersError::EmbeddingError {
                reason: format!("Failed to get index file: {}", e),
            })?;
    let index_content =
        std::fs::read_to_string(index_file).map_err(|e| JeersError::EmbeddingError {
            reason: format!("Failed to read index file: {}", e),
        })?;
    let json: serde_json::Value =
        serde_json::from_str(&index_content).map_err(|e| JeersError::EmbeddingError {
            reason: format!("Failed to parse index file: {}", e),
        })?;

    let weight_map = extract_weight_map(&json)?;
    let filenames = collect_safetensors_filenames(weight_map);

    let mut files = Vec::new();
    for filename in filenames {
        tracing::info!("Downloading `{}`", filename);
        files.push(api.get(&filename).map_err(|e| JeersError::EmbeddingError {
            reason: format!("Failed to get {}: {}", filename, e),
        })?);
    }

    Ok(files)
}

fn extract_weight_map(
    json: &serde_json::Value,
) -> Result<&serde_json::Map<String, serde_json::Value>, JeersError> {
    match json.get("weight_map") {
        Some(serde_json::Value::Object(map)) => Ok(map),
        _ => Err(JeersError::EmbeddingError {
            reason: "model.safetensors.index.json is corrupted: missing weight_map".to_string(),
        }),
    }
}

fn collect_safetensors_filenames(
    weight_map: &serde_json::Map<String, serde_json::Value>,
) -> std::collections::HashSet<String> {
    let mut filenames = std::collections::HashSet::new();
    for value in weight_map.values() {
        if let Some(file) = value.as_str() {
            filenames.insert(file.to_string());
        }
    }
    filenames
}

fn parse_dense_paths_from_modules(modules_path: &PathBuf) -> Result<Vec<String>, JeersError> {
    let content =
        std::fs::read_to_string(modules_path).map_err(|e| JeersError::EmbeddingError {
            reason: format!("Failed to read modules.json: {}", e),
        })?;
    let modules: Vec<ModuleConfig> =
        serde_json::from_str(&content).map_err(|err| JeersError::EmbeddingError {
            reason: format!("Failed to parse modules.json: {}", err),
        })?;

    Ok(modules
        .into_iter()
        .filter(|module| module.module_type == ModuleType::Dense)
        .map(|module| module.path)
        .collect())
}

fn download_dense_module(api: &ApiRepo, dense_path: &str) -> Result<PathBuf, JeersError> {
    let config_file = format!("{}/config.json", dense_path);
    tracing::info!("Downloading `{}`", config_file);
    let config_path = api
        .get(&config_file)
        .map_err(|e| JeersError::EmbeddingError {
            reason: format!("Failed to get config file: {}", e),
        })?;

    download_dense_weights(api, dense_path)?;

    let parent = config_path
        .parent()
        .ok_or_else(|| JeersError::EmbeddingError {
            reason: "Config file has no parent directory".to_string(),
        })?;
    Ok(parent.to_path_buf())
}

fn download_dense_weights(api: &ApiRepo, dense_path: &str) -> Result<(), JeersError> {
    let safetensors_file = format!("{}/model.safetensors", dense_path);
    tracing::info!("Downloading `{}`", safetensors_file);
    match api.get(&safetensors_file) {
        Ok(_) => Ok(()),
        Err(err) => {
            tracing::warn!("Could not download `{}`: {}", safetensors_file, err);
            let pytorch_file = format!("{}/pytorch_model.bin", dense_path);
            tracing::info!("Downloading `{}`", pytorch_file);
            api.get(&pytorch_file)
                .map_err(|e| JeersError::EmbeddingError {
                    reason: format!("Failed to get {}: {}", pytorch_file, e),
                })?;
            Ok(())
        }
    }
}

fn load_tokenizer(model_root: &Path) -> Result<Tokenizer, JeersError> {
    let tokenizer_path = model_root.join("tokenizer.json");
    let mut tokenizer =
        Tokenizer::from_file(tokenizer_path).map_err(|e| JeersError::EmbeddingError {
            reason: format!("Failed to load tokenizer: {}", e),
        })?;
    tokenizer.with_padding(None);
    Ok(tokenizer)
}

fn batch(encodings: Vec<Encoding>, pooled_indices: Vec<u32>, raw_indices: Vec<u32>) -> Batch {
    let mut input_ids = Vec::new();
    let mut token_type_ids = Vec::new();
    let mut position_ids = Vec::new();
    let mut cumulative_seq_lengths = Vec::with_capacity(encodings.len() + 1);
    cumulative_seq_lengths.push(0);

    let mut max_length = 0;
    let mut cumulative_length = 0;

    for encoding in encodings.iter() {
        let encoding_length = encoding.len() as u32;
        input_ids.extend(encoding.get_ids().to_vec());
        token_type_ids.extend(encoding.get_type_ids().to_vec());
        position_ids.extend(0..encoding_length);
        cumulative_length += encoding_length;
        cumulative_seq_lengths.push(cumulative_length);
        max_length = max(max_length, encoding_length);
    }

    Batch {
        input_ids,
        token_type_ids,
        position_ids,
        cumulative_seq_lengths,
        max_length,
        pooled_indices,
        raw_indices,
    }
}

#[derive(Deserialize, PartialEq)]
enum ModuleType {
    #[serde(rename = "sentence_transformers.models.Dense")]
    Dense,
    #[serde(rename = "sentence_transformers.models.Normalize")]
    Normalize,
    #[serde(rename = "sentence_transformers.models.Pooling")]
    Pooling,
    #[serde(rename = "sentence_transformers.models.Transformer")]
    Transformer,
}

#[derive(Deserialize)]
struct ModuleConfig {
    #[allow(dead_code)]
    idx: usize,
    #[allow(dead_code)]
    name: String,
    path: String,
    #[serde(rename = "type")]
    module_type: ModuleType,
}
