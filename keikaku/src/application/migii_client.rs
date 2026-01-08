use crate::domain::{
    KeikakuError,
    value_objects::{JapaneseLevel, NativeLanguage},
};

#[derive(Debug, Clone)]
pub struct MigiiWord {
    pub word: String,
    pub short_mean: String,
    pub mean: Vec<MigiiMeaning>,
}

#[derive(Debug, Clone)]
pub struct MigiiMeaning {
    pub mean: String,
}

#[async_trait::async_trait]
pub trait MigiiClient: Send + Sync {
    async fn get_words(
        &self,
        native_lang: &NativeLanguage,
        level: &JapaneseLevel,
        lesson: u32,
    ) -> Result<Vec<MigiiWord>, KeikakuError>;
}
