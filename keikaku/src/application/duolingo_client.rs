use crate::domain::KeikakuError;

#[derive(Debug, Clone)]
pub struct DuolingoWord {
    pub text: String,
    pub translations: Vec<String>,
}

#[async_trait::async_trait]
pub trait DuolingoClient: Send + Sync {
    async fn get_words(&self, jwt_token: &str) -> Result<Vec<DuolingoWord>, KeikakuError>;
}
