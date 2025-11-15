use crate::domain::Card;
use ulid::Ulid;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    MainMenu,
    StudySession,
    CardList,
    CardView { card_id: Ulid },
    CardEdit { card_id: Ulid, question: String, answer: String },
    CardCreate { question: String, answer: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuOption {
    Study,
    ManageCards,
    CreateCard,
}

pub struct AppState {
    pub user_id: Ulid,
    pub screen: Screen,
    pub cards: Vec<Card>,
    pub all_cards: Vec<Card>,
    pub current_index: usize,
    pub show_answer: bool,
    pub selected_menu_option: MenuOption,
    pub card_list_index: usize,
    pub input_mode: bool,
    pub input_field: InputField,
}

#[derive(Debug, Clone)]
pub enum InputField {
    None,
    Question,
    Answer,
}

impl AppState {
    pub fn new(user_id: Ulid, cards: Vec<Card>) -> Self {
        Self {
            user_id,
            screen: Screen::MainMenu,
            cards,
            all_cards: Vec::new(),
            current_index: 0,
            show_answer: false,
            selected_menu_option: MenuOption::Study,
            card_list_index: 0,
            input_mode: false,
            input_field: InputField::None,
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

    pub fn update_all_cards(&mut self, cards: Vec<Card>) {
        self.all_cards = cards;
        if self.card_list_index >= self.all_cards.len() {
            self.card_list_index = 0;
        }
    }

    pub fn next_menu_option(&mut self) {
        self.selected_menu_option = match self.selected_menu_option {
            MenuOption::Study => MenuOption::ManageCards,
            MenuOption::ManageCards => MenuOption::CreateCard,
            MenuOption::CreateCard => MenuOption::Study,
        };
    }

    pub fn prev_menu_option(&mut self) {
        self.selected_menu_option = match self.selected_menu_option {
            MenuOption::Study => MenuOption::CreateCard,
            MenuOption::ManageCards => MenuOption::Study,
            MenuOption::CreateCard => MenuOption::ManageCards,
        };
    }

    pub fn next_card_in_list(&mut self) {
        if self.card_list_index < self.all_cards.len().saturating_sub(1) {
            self.card_list_index += 1;
        }
    }

    pub fn prev_card_in_list(&mut self) {
        if self.card_list_index > 0 {
            self.card_list_index -= 1;
        }
    }

    pub fn current_card_in_list(&self) -> Option<&Card> {
        self.all_cards.get(self.card_list_index)
    }
}
