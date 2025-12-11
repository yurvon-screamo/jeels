use crate::application::user_repository::UserRepository;
use crate::domain::VocabularyCard;
use crate::domain::error::JeersError;
use ulid::Ulid;

#[derive(Clone)]
pub struct DeleteCardUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> DeleteCardUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        card_id: Ulid,
    ) -> Result<VocabularyCard, JeersError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let card = user.delete_card(card_id)?;
        self.repository.save(&user).await?;

        Ok(card)
    }
}
