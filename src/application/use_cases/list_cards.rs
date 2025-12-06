use crate::application::user_repository::UserRepository;
use crate::domain::VocabularyCard;
use crate::domain::error::JeersError;
use ulid::Ulid;

#[derive(Clone)]
pub struct ListCardsUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> ListCardsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Ulid) -> Result<Vec<VocabularyCard>, JeersError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let mut cards: Vec<VocabularyCard> = user.cards().values().cloned().collect();
        cards.sort_by_key(|card| card.next_review_date());

        Ok(cards)
    }
}
