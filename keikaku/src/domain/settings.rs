use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UserSettings {
    llm: LlmSettings,
    embedding: EmbeddingSettings,
    translation: TranslationSettings,
    duolingo_jwt_token: Option<String>,
    #[serde(default)]
    learn: LearnSettings,
}

impl UserSettings {
    pub fn new(
        llm: LlmSettings,
        embedding: EmbeddingSettings,
        translation: TranslationSettings,
        duolingo_jwt_token: Option<String>,
        learn: LearnSettings,
    ) -> Self {
        Self {
            llm,
            embedding,
            translation,
            duolingo_jwt_token,
            learn,
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
            learn: LearnSettings::default(),
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

    pub fn learn(&self) -> &LearnSettings {
        &self.learn
    }

    pub fn set_learn(&mut self, learn: LearnSettings) {
        self.learn = learn;
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LearnSettings {
    limit: Option<usize>,
    show_furigana: bool,
    low_stability_mode: bool,
    force_new_cards: bool,
}

impl LearnSettings {
    pub fn new(
        limit: Option<usize>,
        show_furigana: bool,
        low_stability_mode: bool,
        force_new_cards: bool,
    ) -> Self {
        Self {
            limit,
            show_furigana,
            low_stability_mode,
            force_new_cards,
        }
    }

    pub fn default() -> Self {
        Self {
            limit: Some(30),
            show_furigana: true,
            low_stability_mode: false,
            force_new_cards: false,
        }
    }

    pub fn limit(&self) -> Option<usize> {
        self.limit
    }

    pub fn set_limit(&mut self, limit: Option<usize>) {
        self.limit = limit;
    }

    pub fn show_furigana(&self) -> bool {
        self.show_furigana
    }

    pub fn set_show_furigana(&mut self, show_furigana: bool) {
        self.show_furigana = show_furigana;
    }

    pub fn low_stability_mode(&self) -> bool {
        self.low_stability_mode
    }

    pub fn set_low_stability_mode(&mut self, low_stability_mode: bool) {
        self.low_stability_mode = low_stability_mode;
    }

    pub fn force_new_cards(&self) -> bool {
        self.force_new_cards
    }

    pub fn set_force_new_cards(&mut self, force_new_cards: bool) {
        self.force_new_cards = force_new_cards;
    }
}
