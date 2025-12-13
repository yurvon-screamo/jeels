use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UserSettings {
    llm: LlmSettings,
    embedding: EmbeddingSettings,
    translation: TranslationSettings,
    duolingo_jwt_token: Option<String>,
}

impl UserSettings {
    pub fn new(
        llm: LlmSettings,
        embedding: EmbeddingSettings,
        translation: TranslationSettings,
        duolingo_jwt_token: Option<String>,
    ) -> Self {
        Self {
            llm,
            embedding,
            translation,
            duolingo_jwt_token,
        }
    }

    pub fn empty() -> Self {
        Self {
            duolingo_jwt_token: None,
            llm: LlmSettings::None,
            embedding: EmbeddingSettings::None,
            translation: TranslationSettings {
                temperature: 0.3,
                seed: 0,
            },
        }
    }

    pub fn llm(&self) -> &LlmSettings {
        &self.llm
    }

    pub fn set_llm(&mut self, llm: LlmSettings) {
        self.llm = llm;
    }

    pub fn embedding(&self) -> &EmbeddingSettings {
        &self.embedding
    }

    pub fn set_embedding(&mut self, embedding: EmbeddingSettings) {
        self.embedding = embedding;
    }

    pub fn translation(&self) -> &TranslationSettings {
        &self.translation
    }

    pub fn set_translation(&mut self, translation: TranslationSettings) {
        self.translation = translation;
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum EmbeddingSettings {
    #[default]
    None,
    Candle,
    OpenAi {
        model: String,
        base_url: String,
        env_var_name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TranslationSettings {
    temperature: f64,
    seed: u64,
}

impl TranslationSettings {
    pub fn new(temperature: f64, seed: u64) -> Self {
        Self { temperature, seed }
    }

    pub fn default() -> Self {
        Self {
            temperature: 0.8,
            seed: 299792458,
        }
    }

    pub fn temperature(&self) -> f64 {
        self.temperature
    }

    pub fn set_temperature(&mut self, temperature: f64) {
        self.temperature = temperature;
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
    }
}
