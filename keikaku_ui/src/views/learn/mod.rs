mod view;
pub use view::Learn;

mod settings;
pub use settings::LearnSettings;

mod active;
pub use active::LearnActive;

mod completed;
pub use completed::LearnCompleted;

mod use_cases;
pub use use_cases::learn_session::{LearnCard, LearnStep, SessionState};

use crate::{ensure_user, init_env, to_error, DEFAULT_USERNAME};
use dioxus::prelude::*;
use keikaku::application::use_cases::select_cards_to_learn::SelectCardsToLearnUseCase;
use keikaku::domain::study_session::StudySessionItem;

#[derive(Clone, PartialEq)]
pub struct UseLearnSession {
    pub state: Signal<SessionState>,
    pub cards: Signal<Vec<LearnCard>>,
    pub current_index: Signal<usize>,
    pub current_step: Signal<LearnStep>,
    pub limit: Signal<String>,
    pub show_furigana: Signal<bool>,
    pub loading: Signal<bool>,
}

pub async fn fetch_cards_to_learn(limit: usize) -> Result<Vec<StudySessionItem>, String> {
    let env = init_env().await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    SelectCardsToLearnUseCase::new(repo)
        .execute(user_id, false, false, Some(limit))
        .await
        .map_err(to_error)
}

pub fn map_study_item_to_learn_card(item: StudySessionItem) -> LearnCard {
    match item {
        StudySessionItem::Vocabulary(v) => LearnCard {
            id: v.card_id().to_string(),
            question: v.word().to_string(),
            answer: v.meaning().to_string(),
        },
        StudySessionItem::Kanji(k) => LearnCard {
            id: k.card_id().to_string(),
            question: k.kanji().to_string(),
            answer: k.description().to_string(),
        },
    }
}

impl UseLearnSession {
    pub fn current_card(&self) -> Option<LearnCard> {
        let cards = (self.cards)();
        let index = (self.current_index)();
        cards.get(index).cloned()
    }

    pub fn progress(&self) -> f64 {
        let cards = (self.cards)();
        if cards.is_empty() {
            0.0
        } else {
            let index = (self.current_index)();
            (index as f64 + 1.0) / cards.len() as f64 * 100.0
        }
    }

    pub fn has_cards(&self) -> bool {
        !(self.cards)().is_empty()
    }

    pub fn is_completed(&self) -> bool {
        let index = (self.current_index)();
        let cards_len = (self.cards)().len();
        index >= cards_len.saturating_sub(1)
    }
}
