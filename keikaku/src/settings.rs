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

const DB_PATH: &str = "~/.keikaku";
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

fn expand_tilde(path: &str) -> String {
    if !path.starts_with("~/") && path != "~" {
        return path.to_string();
    }

    // On Android, use app's internal files directory instead of HOME
    // HOME on Android may point to a read-only location
    if std::env::var("ANDROID_DATA").is_ok() {
        // Try to get app files directory from environment variable first
        // This is typically set by the Android runtime or framework
        let base_path = if let Ok(app_path) = std::env::var("ANDROID_APP_PATH") {
            app_path
        } else {
            // Fallback: construct path using ANDROID_DATA and package name
            // Format: /data/data/<package>/files
            let android_data =
                std::env::var("ANDROID_DATA").unwrap_or_else(|_| "/data".to_string());
            let package_name =
                std::env::var("ANDROID_PACKAGE_NAME").unwrap_or_else(|_| "keikaku".to_string());
            format!("{}/data/{}/files", android_data, package_name)
        };

        let base = PathBuf::from(&base_path);

        if path == "~" {
            base.to_string_lossy().to_string()
        } else {
            let relative_path = path.strip_prefix("~/").unwrap_or(path);
            base.join(relative_path).to_string_lossy().to_string()
        }
    } else {
        // Non-Android platforms - use standard home directory
        // Try multiple methods to get home directory for cross-platform support
        // 1. HOME - standard on Unix/Linux
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSettings {
    pub username: String,
    pub password: String,
}

impl ApplicationEnvironment {
    pub async fn get_repository(&self) -> Result<&FileSystemUserRepository, JeersError> {
        let path = expand_tilde(DB_PATH);

        self.lazy_repository
            .get_or_try_init(|| async {
                FileSystemUserRepository::new(&path)
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
