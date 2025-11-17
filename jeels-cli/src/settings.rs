use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::infrastructure::{EmbeddingGenerator, FsrsSrsService, PoloDbUserRepository, QwenLlm};

static SETTINGS: OnceLock<Settings> = OnceLock::new();

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub llm: LlmSettings,

    #[serde(skip)]
    lazy_repository: Option<PoloDbUserRepository>,
    #[serde(skip)]
    lazy_embedding_generator: Option<EmbeddingGenerator>,
    #[serde(skip)]
    lazy_qwen_llm: Option<QwenLlm>,
    #[serde(skip)]
    lazy_srs_service: Option<FsrsSrsService>,
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
    pub fn from_database_path(database_path: PathBuf) -> Self {
        Self {
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
            lazy_repository: None,
            lazy_embedding_generator: None,
            lazy_qwen_llm: None,
            lazy_srs_service: None,
        }
    }
}

impl Settings {
    pub async fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::find_config_file()?;
        let contents = std::fs::read_to_string(&config_path)?;
        let mut settings: Settings = toml::from_str(&contents)?;

        settings.lazy_repository = Some(PoloDbUserRepository::new(&settings).await?);
        settings.lazy_embedding_generator = Some(EmbeddingGenerator::new()?);
        settings.lazy_qwen_llm = Some(QwenLlm::new(&settings.llm)?);
        settings.lazy_srs_service = Some(FsrsSrsService::new()?);

        Ok(settings)
    }

    pub fn get_repository(&self) -> &PoloDbUserRepository {
        self.lazy_repository.as_ref().expect("Repository not built")
    }

    pub fn get_embedding_generator(&self) -> &EmbeddingGenerator {
        self.lazy_embedding_generator
            .as_ref()
            .expect("Embedding generator not built")
    }

    pub fn get_llm(&self) -> &QwenLlm {
        self.lazy_qwen_llm.as_ref().expect("LLM not built")
    }

    pub fn get_srs_service(&self) -> &FsrsSrsService {
        self.lazy_srs_service
            .as_ref()
            .expect("SRS service not built")
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

    pub fn init(settings: Settings) -> Result<(), Box<dyn std::error::Error>> {
        SETTINGS
            .set(settings)
            .map_err(|_| "Settings already initialized".into())
    }

    pub fn get() -> &'static Settings {
        SETTINGS.get().expect("Settings not initialized")
    }
}
