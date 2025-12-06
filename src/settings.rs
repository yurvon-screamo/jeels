use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use crate::application::LlmService;
use crate::domain::JeersError;
use crate::infrastructure::{
    CandleEmbeddingService, CandleLlm, CandleTranslationService, FileSystemUserRepository,
    FsrsSrsService, GeminiLlm, HttpMigiiClient, OpenAiLlm,
};
use tokio::sync::OnceCell;

static SETTINGS: OnceLock<ApplicationEnvironment> = OnceLock::new();

pub struct ApplicationEnvironment {
    settings: Settings,

    lazy_repository: Arc<OnceCell<FileSystemUserRepository>>,
    lazy_embedding_generator: Arc<OnceCell<CandleEmbeddingService>>,
    lazy_srs_service: Arc<OnceCell<FsrsSrsService>>,
    lazy_translation_service: Arc<OnceCell<CandleTranslationService>>,
    lazy_migii_client: Arc<OnceCell<HttpMigiiClient>>,

    lazy_llm: Arc<OnceCell<Box<dyn LlmService>>>,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub llm: LlmSettings,
    pub translation: TranslationSettings,
}

#[derive(Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSettings {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmSettings {
    #[serde(rename = "gemini")]
    Gemini { temperature: f32, model: String },
    #[serde(rename = "openai")]
    OpenAi {
        temperature: f32,
        model: String,
        base_url: String,
        env_var_name: String,
    },
    #[serde(rename = "candle")]
    Candle {
        max_sample_len: usize,
        temperature: f32,
        seed: u64,
        model_repo: String,
        model_filename: String,
        model_revision: String,
        tokenizer_repo: String,
        tokenizer_filename: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationSettings {
    pub temperature: f64,
    pub seed: u64,
}

impl ApplicationEnvironment {
    pub fn from_database_path(database_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let settings = Settings {
            database: DatabaseSettings {
                path: database_path,
            },
            llm: LlmSettings::Candle {
                max_sample_len: 8192,
                temperature: 0.7,
                seed: 299792458,
                model_repo: "unsloth/Qwen3-1.7B-GGUF".to_string(),
                model_filename: "Qwen3-1.7B-Q4_K_M.gguf".to_string(),
                model_revision: "main".to_string(),
                tokenizer_repo: "Qwen/Qwen3-1.7B".to_string(),
                tokenizer_filename: "tokenizer.json".to_string(),
            },
            translation: TranslationSettings {
                temperature: 0.8,
                seed: 299792458,
            },
        };

        Self::init(settings)?;
        Ok(())
    }

    pub async fn load() -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::find_config_file()?;
        let contents = std::fs::read_to_string(&config_path)?;
        let settings: Settings = toml::from_str(&contents)?;
        Self::init(settings)?;
        Ok(())
    }

    pub async fn get_repository(&self) -> Result<&FileSystemUserRepository, JeersError> {
        self.lazy_repository
            .get_or_try_init(|| async {
                FileSystemUserRepository::new(self.settings.database.path.to_str().unwrap())
                    .await
                    .map_err(|e| JeersError::SettingsError {
                        reason: e.to_string(),
                    })
            })
            .await
    }

    pub async fn get_embedding_generator(&self) -> Result<&CandleEmbeddingService, JeersError> {
        self.lazy_embedding_generator
            .get_or_try_init(|| async {
                CandleEmbeddingService::new().map_err(|e| JeersError::SettingsError {
                    reason: e.to_string(),
                })
            })
            .await
    }

    pub async fn get_llm_service(&self) -> Result<&Box<dyn LlmService>, JeersError> {
        let llm = self
            .lazy_llm
            .get_or_try_init(|| async {
                let boxed_service: Box<dyn LlmService> = match &self.settings.llm {
                    LlmSettings::Gemini { temperature, model } => {
                        Box::new(GeminiLlm::new(*temperature, model.clone()).map_err(|e| {
                            JeersError::SettingsError {
                                reason: e.to_string(),
                            }
                        })?)
                    }
                    LlmSettings::OpenAi {
                        temperature,
                        model,
                        base_url,
                        env_var_name,
                    } => Box::new(
                        OpenAiLlm::new(
                            *temperature,
                            model.clone(),
                            base_url.clone(),
                            env_var_name.clone(),
                        )
                        .map_err(|e| JeersError::SettingsError {
                            reason: e.to_string(),
                        })?,
                    ),
                    LlmSettings::Candle {
                        max_sample_len,
                        temperature,
                        seed,
                        model_repo,
                        model_filename,
                        model_revision,
                        tokenizer_repo,
                        tokenizer_filename,
                    } => Box::new(
                        CandleLlm::new(
                            *max_sample_len,
                            *temperature,
                            *seed,
                            model_repo.clone(),
                            model_filename.clone(),
                            model_revision.clone(),
                            tokenizer_repo.clone(),
                            tokenizer_filename.clone(),
                        )
                        .map_err(|e| JeersError::SettingsError {
                            reason: e.to_string(),
                        })?,
                    ),
                };

                Ok(boxed_service)
            })
            .await?;

        Ok(llm)
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

    pub async fn get_translation_service(&self) -> Result<&CandleTranslationService, JeersError> {
        let temperature = self.settings.translation.temperature;
        let seed = self.settings.translation.seed;
        self.lazy_translation_service
            .get_or_try_init(|| async {
                CandleTranslationService::new(temperature, seed).map_err(|e| {
                    JeersError::SettingsError {
                        reason: e.to_string(),
                    }
                })
            })
            .await
    }

    pub async fn get_migii_client(&self) -> Result<&HttpMigiiClient, JeersError> {
        self.lazy_migii_client
            .get_or_try_init(|| async { Ok(HttpMigiiClient::new()) })
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
        let environment = ApplicationEnvironment {
            settings,
            lazy_repository: Arc::new(OnceCell::new()),
            lazy_embedding_generator: Arc::new(OnceCell::new()),
            lazy_srs_service: Arc::new(OnceCell::new()),
            lazy_translation_service: Arc::new(OnceCell::new()),
            lazy_migii_client: Arc::new(OnceCell::new()),
            lazy_llm: Arc::new(OnceCell::new()),
        };

        SETTINGS
            .set(environment)
            .map_err(|_| "Settings already initialized".into())
    }

    pub fn get() -> &'static ApplicationEnvironment {
        SETTINGS.get().expect("Settings not initialized")
    }
}

#[async_trait]
impl LlmService for Box<dyn LlmService> {
    async fn generate_text(&self, question: &str) -> Result<String, JeersError> {
        (**self).generate_text(question).await
    }
}
