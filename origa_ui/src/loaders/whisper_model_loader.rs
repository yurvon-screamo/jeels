use origa::domain::OrigaError;
use origa::stt::WhisperTranscriber;
use tracing::info;

use crate::loaders::model_cache::ModelCache;

const WHISPER_FILE_COUNT: usize = 3;

fn to_stt_error(reason: String) -> OrigaError {
    OrigaError::SttError { reason }
}

pub struct WhisperModelFiles {
    pub encoder: Vec<u8>,
    pub decoder: Vec<u8>,
    pub tokenizer: Vec<u8>,
}

pub struct WhisperModelLoader {
    base_url: String,
    cache_name: String,
}

impl WhisperModelLoader {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            cache_name: "whisper-model-cache".to_string(),
        }
    }

    pub async fn load(&self) -> Result<WhisperModelFiles, OrigaError> {
        info!("Loading Whisper models from {}", self.base_url);

        let model_cache = ModelCache::new(&self.cache_name, to_stt_error);

        let cache = model_cache.get_cache().await?;
        let files: Vec<(&str, String)> = [
            "onnx/encoder_model.onnx",
            "onnx/decoder_model.onnx",
            "tokenizer.json",
        ]
        .iter()
        .map(|&f| (f, format!("{}/{}", self.base_url.trim_end_matches('/'), f)))
        .collect();

        if model_cache.ensure_files_cached(&cache, &files).await? {
            info!("Whisper models found in cache, loading...");
            let loaded = model_cache.load_files_from_cache(&cache, &files).await?;
            self.build_model_files(loaded)
        } else {
            info!("Whisper models not in cache, downloading...");
            let loaded = model_cache.download_and_cache_model(&cache, &files).await?;
            self.build_model_files(loaded)
        }
    }

    pub async fn init_model(files: WhisperModelFiles) -> Result<WhisperTranscriber, OrigaError> {
        WhisperTranscriber::new(&files.encoder, &files.decoder, &files.tokenizer).await
    }

    fn build_model_files(&self, mut loaded: Vec<Vec<u8>>) -> Result<WhisperModelFiles, OrigaError> {
        if loaded.len() != WHISPER_FILE_COUNT {
            return Err(OrigaError::SttError {
                reason: format!(
                    "Expected {} model files, got {}",
                    WHISPER_FILE_COUNT,
                    loaded.len()
                ),
            });
        }
        Ok(WhisperModelFiles {
            encoder: loaded.remove(0),
            decoder: loaded.remove(0),
            tokenizer: loaded.remove(0),
        })
    }
}
