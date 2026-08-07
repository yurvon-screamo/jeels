use origa::domain::OrigaError;
use origa::ocr::{ModelConfig, ModelFiles};
use tracing::info;

use crate::loaders::model_cache::ModelCache;

pub use crate::loaders::model_cache::ProgressCallback;

const MODEL_FILE_COUNT: usize = 5;

fn to_ocr_error(reason: String) -> OrigaError {
    OrigaError::OcrError { reason }
}

pub struct ModelLoader {
    config: ModelConfig,
    on_progress: Option<ProgressCallback>,
}

impl ModelLoader {
    pub fn new(config: ModelConfig) -> Self {
        Self {
            config,
            on_progress: None,
        }
    }

    pub fn with_progress_callback(mut self, callback: ProgressCallback) -> Self {
        self.on_progress = Some(callback);
        self
    }

    pub async fn load_or_download_model(&self) -> Result<ModelFiles, OrigaError> {
        info!(
            "Loading NDLOCR-Lite models from {}",
            self.config.ndlocr_base_url
        );

        let model_cache = ModelCache::new(&self.config.ndlocr_cache_dir, to_ocr_error);
        let model_cache = match &self.on_progress {
            Some(cb) => model_cache.with_progress_callback(cb.clone()),
            None => model_cache,
        };

        let cache = model_cache.get_cache().await?;
        let files: Vec<(&str, String)> = ModelConfig::ndlocr_file_names()
            .iter()
            .map(|&f| (f, self.config.model_url(f)))
            .collect();

        if model_cache.ensure_files_cached(&cache, &files).await? {
            info!("NDLOCR-Lite models found in cache, loading...");
            let loaded = model_cache.load_files_from_cache(&cache, &files).await?;
            self.build_model_files(loaded)
        } else {
            info!("NDLOCR-Lite models not found in cache, downloading...");
            let loaded = model_cache.download_and_cache_model(&cache, &files).await?;
            self.build_model_files(loaded)
        }
    }

    fn build_model_files(&self, mut loaded: Vec<Vec<u8>>) -> Result<ModelFiles, OrigaError> {
        if loaded.len() != MODEL_FILE_COUNT {
            return Err(OrigaError::OcrError {
                reason: format!(
                    "Expected {} model files, got {}",
                    MODEL_FILE_COUNT,
                    loaded.len()
                ),
            });
        }
        Ok(ModelFiles {
            deim: loaded.remove(0),
            parseq30: loaded.remove(0),
            parseq50: loaded.remove(0),
            parseq100: loaded.remove(0),
            vocab: loaded.remove(0),
        })
    }
}
