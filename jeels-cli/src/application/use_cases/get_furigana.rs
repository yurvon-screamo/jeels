use crate::application::{FuriganaService, UserRepository};
use crate::domain::error::JeersError;
use ulid::Ulid;

#[derive(Clone)]
pub struct GetFuriganaUseCase<'a, R: UserRepository, F: FuriganaService> {
    repository: &'a R,
    furigana_service: &'a F,
}

impl<'a, R: UserRepository, F: FuriganaService> GetFuriganaUseCase<'a, R, F> {
    pub fn new(repository: &'a R, furigana_service: &'a F) -> Self {
        Self {
            repository,
            furigana_service,
        }
    }

    pub async fn execute(&self, user_id: Ulid, card_id: Ulid) -> Result<String, JeersError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let card = user
            .get_card(card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        let question_text = card.question().text();
        let question_furigana = self.furigana_service.get_furigana(question_text);

        Ok(question_furigana)
    }
}
