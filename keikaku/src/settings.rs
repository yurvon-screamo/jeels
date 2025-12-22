use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};

use crate::application::UserRepository;
use crate::domain::{JeersError, LlmSettings};
use crate::infrastructure::{
    CandleLlm, CandleTranslationService, EmbeddedMigiiClient, FileSystemUserRepository,
    FsrsSrsService, GeminiLlm, LlmServiceInvoker, OpenAiLlm,
};
use tokio::sync::OnceCell;

static SETTINGS: LazyLock<ApplicationEnvironment> = LazyLock::new(|| ApplicationEnvironment {
    lazy_repository: Arc::new(OnceCell::new()),
    lazy_srs_service: Arc::new(OnceCell::new()),
    lazy_migii_client: Arc::new(OnceCell::new()),
});

pub struct ApplicationEnvironment {
    lazy_repository: Arc<OnceCell<FileSystemUserRepository>>,
    lazy_srs_service: Arc<OnceCell<FsrsSrsService>>,
    lazy_migii_client: Arc<OnceCell<EmbeddedMigiiClient>>,
}

fn expand_tilde() -> PathBuf {
    if std::env::var("ANDROID_DATA").is_ok() {
        PathBuf::from(format!("/data/data/{}/files", "net.uwuwu.keikaku"))
    } else {
        // Non-Android platforms - use standard home directory
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE")) // Windows
            .or_else(|_| {
                // Windows fallback: HOMEPATH is relative to HOMEDRIVE
                std::env::var("HOMEPATH")
                    .and_then(|hp| std::env::var("HOMEDRIVE").map(|hd| format!("{}{}", hd, hp)))
            })
            .unwrap_or_else(|_| "~".to_string());

        let home_path = PathBuf::from(&home);
        home_path.join(".keikaku")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSettings {
    pub username: String,
    pub password: String,
}

impl ApplicationEnvironment {
    pub async fn get_repository(&self) -> Result<&FileSystemUserRepository, JeersError> {
        let path = expand_tilde();

        self.lazy_repository
            .get_or_try_init(|| async {
                FileSystemUserRepository::new(path)
                    .await
                    .map_err(|e| JeersError::SettingsError {
                        reason: e.to_string(),
                    })
            })
            .await
    }

    pub async fn get_llm_service(
        &self,
        user_id: ulid::Ulid,
    ) -> Result<LlmServiceInvoker, JeersError> {
        let repository = self.get_repository().await?;
        let user = repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;
        let llm_settings = user.settings().llm();

        let service = match llm_settings {
            LlmSettings::Gemini { temperature, model } => {
                LlmServiceInvoker::Gemini(GeminiLlm::new(*temperature, model.clone()).map_err(
                    |e| JeersError::SettingsError {
                        reason: e.to_string(),
                    },
                )?)
            }
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
            LlmSettings::None => LlmServiceInvoker::None,
        };
        Ok(service)
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

    pub async fn get_translation_service(
        &self,
        user_id: ulid::Ulid,
    ) -> Result<CandleTranslationService, JeersError> {
        let repository = self.get_repository().await?;
        let user = repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;
        let translation_settings = user.settings().translation();

        Ok(CandleTranslationService {})
    }

    pub async fn get_migii_client(&self) -> Result<&EmbeddedMigiiClient, JeersError> {
        self.lazy_migii_client
            .get_or_try_init(|| async { Ok(EmbeddedMigiiClient::new()) })
            .await
    }

    pub fn get() -> &'static ApplicationEnvironment {
        &SETTINGS
    }
}
