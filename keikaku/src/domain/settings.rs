use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UserSettings {
    llm: LlmSettings,
    duolingo_jwt_token: Option<String>,
}

impl UserSettings {
    pub fn new(llm: LlmSettings, duolingo_jwt_token: Option<String>) -> Self {
        Self {
            llm,
            duolingo_jwt_token,
        }
    }

    pub fn empty() -> Self {
        Self {
            duolingo_jwt_token: None,
            llm: LlmSettings::None,
        }
    }

    pub fn llm(&self) -> &LlmSettings {
        &self.llm
    }

    pub fn set_llm(&mut self, llm: LlmSettings) {
        self.llm = llm;
    }

    pub fn duolingo_jwt_token(&self) -> Option<&str> {
        self.duolingo_jwt_token.as_deref()
    }

    pub fn set_duolingo_jwt_token(&mut self, token: Option<String>) {
        self.duolingo_jwt_token = token;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum LlmSettings {
    #[default]
    None,
    Gemini {
        temperature: f32,
        model: String,
    },
    OpenAi {
        temperature: f32,
        model: String,
        base_url: String,
        env_var_name: String,
    },
}
