use crate::application::{FuriganaService, user_repository::UserRepository};
use crate::domain::Card;
use crate::domain::error::JeersError;
use ulid::Ulid;

#[derive(Clone, Debug)]
pub struct SimilarCard {
    pub card: Card,
    pub furigana: String,
}

#[derive(Clone)]
pub struct FindSimilarityUseCase<'a, R: UserRepository, F: FuriganaService> {
    repository: &'a R,
    furigana_service: &'a F,
}

impl<'a, R: UserRepository, F: FuriganaService> FindSimilarityUseCase<'a, R, F> {
    pub fn new(repository: &'a R, furigana_service: &'a F) -> Self {
        Self {
            repository,
            furigana_service,
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        card_id: Ulid,
    ) -> Result<Vec<SimilarCard>, JeersError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let similar_cards = user.find_similarity(card_id)?;

        let similar_cards_with_furigana: Vec<SimilarCard> = similar_cards
            .into_iter()
            .map(|card| {
                let question_text = card.question().text();
                let furigana = self.furigana_service.get_furigana(question_text);
                SimilarCard { card, furigana }
            })
            .collect();

        Ok(similar_cards_with_furigana)
    }
}
