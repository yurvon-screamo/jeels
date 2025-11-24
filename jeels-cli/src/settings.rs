use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use crate::domain::JeersError;
use crate::infrastructure::{
    AutorubyFuriganaGenerator, CandleEmbeddingService, CandleLlm, FsrsSrsService, GeminiLlm,
    OpenRouterLlm, PoloDbUserRepository,
};
use tokio::sync::OnceCell;

static SETTINGS: OnceLock<ApplicationEnvironment> = OnceLock::new();

pub struct ApplicationEnvironment {
    pub settings: Settings,

    lazy_repository: Arc<OnceCell<PoloDbUserRepository>>,
    lazy_embedding_generator: Arc<OnceCell<CandleEmbeddingService>>,
    lazy_srs_service: Arc<OnceCell<FsrsSrsService>>,
    lazy_furigana_service: Arc<OnceCell<AutorubyFuriganaGenerator>>,

    lazy_gemini_llm: Arc<OnceCell<GeminiLlm>>,
    _lazy_openrouter_llm: Arc<OnceCell<OpenRouterLlm>>,
    _lazy_candle_llm: Arc<OnceCell<CandleLlm>>,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub llm: LlmSettings,
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
pub enum LlmSettings {
    #[serde(rename = "gemini")]
    Gemini { temperature: f32, model: String },
    #[serde(rename = "openrouter")]
    OpenRouter { temperature: f64, model: String },
    #[serde(rename = "candle")]
    Candle {
        max_sample_len: usize,
        temperature: f64,
        seed: u64,
        model_repo: String,
        model_filename: String,
        model_revision: String,
        tokenizer_repo: String,
        tokenizer_filename: String,
    },
}

impl ApplicationEnvironment {
    pub fn from_database_path(database_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let settings = Settings {
            database: DatabaseSettings {
                path: database_path,
                namespace: "default".to_string(),
                database: "default".to_string(),
                auth: AuthSettings {
                    username: "default".to_string(),
                    password: "default".to_string(),
                },
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

    pub async fn get_embedding_generator(&self) -> Result<&CandleEmbeddingService, JeersError> {
        self.lazy_embedding_generator
            .get_or_try_init(|| async {
                CandleEmbeddingService::new().map_err(|e| JeersError::SettingsError {
                    reason: e.to_string(),
                })
            })
            .await
    }

    pub async fn get_llm_service(&self) -> Result<&GeminiLlm, JeersError> {
        match &self.settings.llm {
            LlmSettings::Gemini { temperature, model } => {
                self.lazy_gemini_llm
                    .get_or_try_init(|| async {
                        GeminiLlm::new(*temperature, model.clone()).map_err(|e| {
                            JeersError::SettingsError {
                                reason: e.to_string(),
                            }
                        })
                    })
                    .await
            }
            LlmSettings::OpenRouter {
                temperature: _,
                model: _,
            } => {
                // self.lazy_openrouter_llm
                //     .get_or_try_init(|| async {
                //         OpenRouterLlm::new(*temperature, model.clone()).map_err(|e| {
                //             JeersError::SettingsError {
                //                 reason: e.to_string(),
                //             }
                //         })
                //     })
                //     .await
                Err(JeersError::SettingsError {
                    reason: "OpenRouter not supported yet".to_string(),
                })
            }
            LlmSettings::Candle {
                max_sample_len: _,
                temperature: _,
                seed: _,
                model_repo: _,
                model_filename: _,
                model_revision: _,
                tokenizer_repo: _,
                tokenizer_filename: _,
            } => {
                // self.lazy_candle_llm
                //     .get_or_try_init(|| async {
                //         CandleLlm::new(
                //             *max_sample_len,
                //             *temperature,
                //             *seed,
                //             model_repo.clone(),
                //             model_filename.clone(),
                //             model_revision.clone(),
                //             tokenizer_repo.clone(),
                //             tokenizer_filename.clone(),
                //         )
                //         .map_err(|e| JeersError::SettingsError {
                //             reason: e.to_string(),
                //         })
                //     })
                //     .await
                Err(JeersError::SettingsError {
                    reason: "Candle not supported yet".to_string(),
                })
            }
        }
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

    pub async fn get_furigana_service(&self) -> Result<&AutorubyFuriganaGenerator, JeersError> {
        self.lazy_furigana_service
            .get_or_try_init(|| async {
                AutorubyFuriganaGenerator::new().map_err(|e| JeersError::SettingsError {
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
        let environment = ApplicationEnvironment {
            settings,
            lazy_repository: Arc::new(OnceCell::new()),
            lazy_embedding_generator: Arc::new(OnceCell::new()),
            lazy_srs_service: Arc::new(OnceCell::new()),
            lazy_furigana_service: Arc::new(OnceCell::new()),
            lazy_gemini_llm: Arc::new(OnceCell::new()),
            _lazy_openrouter_llm: Arc::new(OnceCell::new()),
            _lazy_candle_llm: Arc::new(OnceCell::new()),
        };

        SETTINGS
            .set(environment)
            .map_err(|_| "Settings already initialized".into())
    }

    pub fn get() -> &'static ApplicationEnvironment {
        SETTINGS.get().expect("Settings not initialized")
    }
}
