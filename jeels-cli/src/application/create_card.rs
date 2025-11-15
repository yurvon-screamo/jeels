use crate::domain::card::Card;
use crate::domain::repositories::CardRepository;
use std::sync::Arc;

pub struct CreateCardUseCase {
    card_repository: Arc<dyn CardRepository>,
}

impl CreateCardUseCase {
    pub fn new(card_repository: Arc<dyn CardRepository>) -> Self {
        CreateCardUseCase { card_repository }
    }

    pub fn execute(&self, question: String, answer: String) -> Result<Card, Box<dyn std::error::Error>> {
        let card = Card::new(question, answer);
        self.card_repository.save(&card)?;
        Ok(card)
    }
}
