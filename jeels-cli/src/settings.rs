use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use crate::domain::JeersError;
use crate::infrastructure::{EmbeddingGenerator, FsrsSrsService, PoloDbUserRepository, QwenLlm};
use tokio::sync::OnceCell;

static SETTINGS: OnceLock<Settings> = OnceLock::new();

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub llm: LlmSettings,

    #[serde(skip)]
    lazy_repository: Arc<OnceCell<PoloDbUserRepository>>,
    #[serde(skip)]
    lazy_embedding_generator: Arc<OnceCell<EmbeddingGenerator>>,
    #[serde(skip)]
    lazy_qwen_llm: Arc<OnceCell<QwenLlm>>,
    #[serde(skip)]
    lazy_srs_service: Arc<OnceCell<FsrsSrsService>>,
}

#[derive(Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub path: PathBuf,
    pub namespace: String,
    pub database: String,
    pub auth: AuthSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSettings {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSettings {
    pub max_sample_len: usize,
    pub temperature: f64,
    pub seed: u64,
    pub model_repo: String,
    pub model_filename: String,
    pub model_revision: String,
    pub tokenizer_repo: String,
    pub tokenizer_filename: String,
}

impl Settings {
    pub fn from_database_path(database_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let settings = Self {
            database: DatabaseSettings {
                path: database_path,
                namespace: "default".to_string(),
                database: "default".to_string(),
                auth: AuthSettings {
                    username: "default".to_string(),
                    password: "default".to_string(),
                },
            },
            llm: LlmSettings {
                max_sample_len: 8192,
                temperature: 0.7,
                seed: 299792458,
                model_repo: "unsloth/Qwen3-1.7B-GGUF".to_string(),
                model_filename: "Qwen3-1.7B-Q4_K_M.gguf".to_string(),
                model_revision: "main".to_string(),
                tokenizer_repo: "Qwen/Qwen3-1.7B".to_string(),
                tokenizer_filename: "tokenizer.json".to_string(),
            },
            lazy_repository: Arc::new(OnceCell::new()),
            lazy_embedding_generator: Arc::new(OnceCell::new()),
            lazy_qwen_llm: Arc::new(OnceCell::new()),
            lazy_srs_service: Arc::new(OnceCell::new()),
        };

        Self::init(settings)?;
        Ok(())
    }

    pub async fn load() -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::find_config_file()?;
        let contents = std::fs::read_to_string(&config_path)?;
        let mut settings: Settings = toml::from_str(&contents)?;

        settings.lazy_repository = Arc::new(OnceCell::new());
        settings.lazy_embedding_generator = Arc::new(OnceCell::new());
        settings.lazy_qwen_llm = Arc::new(OnceCell::new());
        settings.lazy_srs_service = Arc::new(OnceCell::new());

        Self::init(settings)?;
        Ok(())
    }

    pub async fn get_repository(&self) -> Result<&PoloDbUserRepository, JeersError> {
        self.lazy_repository
            .get_or_try_init(|| async {
                PoloDbUserRepository::new(self)
                    .await
                    .map_err(|e| JeersError::SettingsError {
                        reason: e.to_string(),
                    })
            })
            .await
    }

    pub async fn get_embedding_generator(&self) -> Result<&EmbeddingGenerator, JeersError> {
        self.lazy_embedding_generator
            .get_or_try_init(|| async {
                EmbeddingGenerator::new().map_err(|e| JeersError::SettingsError {
                    reason: e.to_string(),
                })
            })
            .await
    }

    pub async fn get_llm_service(&self) -> Result<&QwenLlm, JeersError> {
        self.lazy_qwen_llm
            .get_or_try_init(|| async {
                QwenLlm::new(&self.llm).map_err(|e| JeersError::SettingsError {
                    reason: e.to_string(),
                })
            })
            .await
    }

    pub async fn get_srs_service(&self) -> Result<&FsrsSrsService, JeersError> {
        self.lazy_srs_service
            .get_or_try_init(|| async {
                FsrsSrsService::new().map_err(|e| JeersError::SettingsError {
                    reason: e.to_string(),
                })
            })
            .await
    }

    fn find_config_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let possible_paths = vec![PathBuf::from("config.toml")];

        for path in possible_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err("config.toml not found in current directory".into())
    }

    fn init(settings: Settings) -> Result<(), Box<dyn std::error::Error>> {
        SETTINGS
            .set(settings)
            .map_err(|_| "Settings already initialized".into())
    }

    pub fn get() -> &'static Settings {
        SETTINGS.get().expect("Settings not initialized")
    }
}
