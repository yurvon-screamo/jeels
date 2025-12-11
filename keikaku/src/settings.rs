use serde::{Deserialize, Deserializer, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use crate::domain::JeersError;
use crate::infrastructure::{
    CandleEmbeddingService, CandleLlm, CandleTranslationService, EmbeddedMigiiClient,
    EmbeddingServiceInvoker, FileSystemUserRepository, FsrsSrsService, GeminiLlm,
    LlmServiceInvoker, OpenAiEmbeddingService, OpenAiLlm,
};
use tokio::sync::OnceCell;

static SETTINGS: OnceLock<ApplicationEnvironment> = OnceLock::new();

pub struct ApplicationEnvironment {
    settings: Settings,

    lazy_repository: Arc<OnceCell<FileSystemUserRepository>>,
    lazy_embedding_service: Arc<OnceCell<EmbeddingServiceInvoker>>,
    lazy_srs_service: Arc<OnceCell<FsrsSrsService>>,
    lazy_translation_service: Arc<OnceCell<CandleTranslationService>>,
    lazy_migii_client: Arc<OnceCell<EmbeddedMigiiClient>>,

    lazy_llm: Arc<OnceCell<LlmServiceInvoker>>,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub llm: LlmSettings,
    pub embedding: EmbeddingSettings,
    pub translation: TranslationSettings,
}

#[derive(Serialize, Deserialize)]
struct DatabaseSettingsHelper {
    path: String,
}

#[derive(Serialize)]
pub struct DatabaseSettings {
    pub path: PathBuf,
}

impl<'de> Deserialize<'de> for DatabaseSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = DatabaseSettingsHelper::deserialize(deserializer)?;
        let expanded_path = expand_tilde(&helper.path);
        Ok(DatabaseSettings {
            path: PathBuf::from(expanded_path),
        })
    }
}

fn expand_tilde(path: &str) -> String {
    if !path.starts_with("~/") && path != "~" {
        return path.to_string();
    }

    // Try multiple methods to get home directory for cross-platform support
    // 1. HOME - standard on Unix/Linux/Android (Termux)
    // 2. USERPROFILE - standard on Windows
    // 3. HOMEDRIVE + HOMEPATH - alternative Windows method
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE")) // Windows
        .or_else(|_| {
            // Windows fallback: HOMEPATH is relative to HOMEDRIVE
            std::env::var("HOMEPATH")
                .and_then(|hp| std::env::var("HOMEDRIVE").map(|hd| format!("{}{}", hd, hp)))
        })
        .unwrap_or_else(|_| "~".to_string());

    if path == "~" {
        home
    } else {
        // Use PathBuf for proper path joining across platforms
        // This handles Windows backslashes and Unix forward slashes correctly
        let home_path = PathBuf::from(&home);
        let relative_path = path.strip_prefix("~/").unwrap_or(path);
        home_path.join(relative_path).to_string_lossy().to_string()
    }
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
pub enum EmbeddingSettings {
    #[serde(rename = "candle")]
    Candle,
    #[serde(rename = "openai")]
    OpenAi {
        model: String,
        base_url: String,
        env_var_name: String,
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
            llm: LlmSettings::OpenAi {
                temperature: 0.3,
                model: "Qwen/Qwen3-4B-Instruct-2507".to_string(),
                base_url: "http://10.2.11.6:8001/v1".to_string(),
                env_var_name: "OPENROUTER_API_KEY".to_string(),
            },
            embedding: EmbeddingSettings::OpenAi {
                model: "Qwen/Qwen3-Embedding-0.6B".to_string(),
                base_url: "http://10.3.168.177:8003/v1".to_string(),
                env_var_name: "OPENROUTER_API_KEY".to_string(),
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

    pub async fn get_embedding_service(&self) -> Result<&EmbeddingServiceInvoker, JeersError> {
        self.lazy_embedding_service
            .get_or_try_init(|| async {
                let service = match &self.settings.embedding {
                    EmbeddingSettings::Candle => EmbeddingServiceInvoker::Candle(
                        CandleEmbeddingService::new().map_err(|e| JeersError::SettingsError {
                            reason: e.to_string(),
                        })?,
                    ),
                    EmbeddingSettings::OpenAi {
                        model,
                        base_url,
                        env_var_name,
                    } => EmbeddingServiceInvoker::OpenAi(
                        OpenAiEmbeddingService::new(
                            model.clone(),
                            base_url.clone(),
                            env_var_name.clone(),
                        )
                        .map_err(|e| JeersError::SettingsError {
                            reason: e.to_string(),
                        })?,
                    ),
                };
                Ok(service)
            })
            .await
    }

    pub async fn get_llm_service(&self) -> Result<&LlmServiceInvoker, JeersError> {
        self.lazy_llm
            .get_or_try_init(|| async {
                let service = match &self.settings.llm {
                    LlmSettings::Gemini { temperature, model } => LlmServiceInvoker::Gemini(
                        GeminiLlm::new(*temperature, model.clone()).map_err(|e| {
                            JeersError::SettingsError {
                                reason: e.to_string(),
                            }
                        })?,
                    ),
                    LlmSettings::OpenAi {
                        temperature,
                        model,
                        base_url,
                        env_var_name,
                    } => LlmServiceInvoker::OpenAi(
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
                    } => LlmServiceInvoker::Candle(
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
                Ok(service)
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

    pub async fn get_migii_client(&self) -> Result<&EmbeddedMigiiClient, JeersError> {
        self.lazy_migii_client
            .get_or_try_init(|| async { Ok(EmbeddedMigiiClient::new()) })
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
            lazy_embedding_service: Arc::new(OnceCell::new()),
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
