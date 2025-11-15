use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    #[serde(default)]
    pub llm: LlmSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(default = "default_model_path")]
    pub model_path: PathBuf,
}

impl Default for LlmSettings {
    fn default() -> Self {
        Self {
            model_path: default_model_path(),
        }
    }
}

fn default_model_path() -> PathBuf {
    PathBuf::from("qwen3-0.6b.gguf")
}

impl Settings {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::find_config_file()?;
        let contents = std::fs::read_to_string(&config_path)?;
        let settings: Settings = toml::from_str(&contents)?;
        Ok(settings)
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
}
