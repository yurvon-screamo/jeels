use crate::domain::Card;
use ulid::Ulid;

pub struct AppState {
    pub user_id: Ulid,
    pub cards: Vec<Card>,
    pub current_index: usize,
    pub show_answer: bool,
}

impl AppState {
    pub fn new(user_id: Ulid, cards: Vec<Card>) -> Self {
        Self {
            user_id,
            cards,
            current_index: 0,
            show_answer: false,
        }
    }

    pub fn toggle_answer(&mut self) {
        self.show_answer = !self.show_answer;
    }

    pub fn next_card(&mut self) {
        if self.current_index < self.cards.len() - 1 {
            self.current_index += 1;
            self.show_answer = false;
        }
    }

    pub fn current_card(&self) -> Option<&Card> {
        self.cards.get(self.current_index)
    }

    pub fn cards_count(&self) -> usize {
        self.cards.len()
    }

    pub fn get_current_card_id(&self) -> Ulid {
        self.cards[self.current_index].id()
    }

    pub fn update_cards(&mut self, cards: Vec<Card>) {
        self.cards = cards;
        if self.current_index >= self.cards.len() {
            self.current_index = 0;
        }
        self.show_answer = false;
    }
}
