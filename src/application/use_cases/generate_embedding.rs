use crate::application::EmbeddingService;
use crate::domain::dictionary::VOCABULARY_DB;
use crate::domain::error::JeersError;
use crate::domain::value_objects::Embedding;

const PROMPT: &str = "Represent this Japanese word for find same words";

#[derive(Clone)]
pub struct GenerateEmbeddingUseCase<'a, E: EmbeddingService> {
    embedding_service: &'a E,
}

impl<'a, E: EmbeddingService> GenerateEmbeddingUseCase<'a, E> {
    pub fn new(embedding_service: &'a E) -> Self {
        Self { embedding_service }
    }

    pub async fn generate_embedding(&self, question_text: &str) -> Result<Embedding, JeersError> {
        if let Some(embedding) = self.try_get_from_dictionary(question_text) {
            return Ok(embedding);
        }

        self.generate_with_service(question_text).await
    }

    fn try_get_from_dictionary(&self, question_text: &str) -> Option<Embedding> {
        VOCABULARY_DB.get_embedding(question_text)
    }

    async fn generate_with_service(&self, question_text: &str) -> Result<Embedding, JeersError> {
        self.embedding_service
            .generate_embedding(PROMPT, question_text)
            .await
    }
}
