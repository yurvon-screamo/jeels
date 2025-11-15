use crate::application::user_repository::UserRepository;
use crate::domain::{Card, JeersError};
use ulid::Ulid;

#[derive(Clone)]
pub struct GetHintsUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

#[derive(Debug, Clone)]
pub struct Hint {
    pub card: Card,
    pub similarity_score: f32,
}

impl<'a, R: UserRepository> GetHintsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        card_id: Ulid,
        limit: usize,
    ) -> Result<Vec<Hint>, JeersError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let similar_cards = user.find_similar_cards(card_id, limit)?;

        let hints: Vec<Hint> = similar_cards
            .into_iter()
            .map(|(card, similarity_score)| Hint {
                card,
                similarity_score,
            })
            .collect();

        Ok(hints)
    }
}
