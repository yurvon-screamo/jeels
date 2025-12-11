use crate::application::user_repository::UserRepository;
use crate::domain::error::JeersError;
use crate::domain::value_objects::{Answer, Question};
use ulid::Ulid;

#[derive(Clone)]
pub struct ViewCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> ViewCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        card_id: Ulid,
    ) -> Result<(Question, Answer), JeersError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let card = user
            .get_card(card_id)
            .ok_or(JeersError::CardNotFound { card_id })?;

        Ok((card.word().clone(), card.meaning().clone()))
    }
}
