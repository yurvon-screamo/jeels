use crate::keikaku_api::{ensure_user, init_env, to_error, DEFAULT_USERNAME};
use dioxus::prelude::*;
use keikaku::application::use_cases::select_cards_to_learn::SelectCardsToLearnUseCase;
use keikaku::domain::study_session::StudySessionItem;

#[derive(Clone, PartialEq)]
pub enum SessionState {
    Settings,
    Loading,
    Active,
    Completed,
}

#[derive(Clone, PartialEq)]
pub enum LearnStep {
    Question,
    Answer,
}

#[derive(Clone, PartialEq)]
pub struct LearnCard {
    pub id: String,
    pub question: String,
    pub answer: String,
}

pub fn use_learn_session() -> UseLearnSession {
    use_hook(|| UseLearnSession {
        state: use_signal(|| SessionState::Settings),
        cards: use_signal(Vec::new),
        current_index: use_signal(|| 0),
        current_step: use_signal(|| LearnStep::Question),
        limit: use_signal(|| "7".to_string()),
        show_furigana: use_signal(|| false),
        loading: use_signal(|| false),
    })
}

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

    pub fn start_session(&self) {
        let limit_str = (self.limit)();
        let mut session = self.clone();

        spawn(async move {
            session.loading.set(true);
            session.state.set(SessionState::Loading);

            let limit_val = limit_str.parse::<usize>().ok();
            match session.fetch_cards(limit_val).await {
                Ok(new_cards) => {
                    if new_cards.is_empty() {
                        session.state.set(SessionState::Settings);
                        session.cards.set(vec![]);
                    } else {
                        session.cards.set(new_cards);
                        session.current_index.set(0);
                        session.current_step.set(LearnStep::Question);
                        session.state.set(SessionState::Active);
                    }
                    session.loading.set(false);
                }
                Err(e) => {
                    session.cards.set(vec![]);
                    session.state.set(SessionState::Settings);
                    session.current_step.set(LearnStep::Question);
                    session.loading.set(false);
                    error!("learn fetch error: {}", e);
                }
            }
        });
    }

    pub fn next_card(&mut self) {
        let current_index = (self.current_index)();
        let cards_len = (self.cards)().len();
        if current_index + 1 < cards_len {
            self.current_index.set(current_index + 1);
            self.current_step.set(LearnStep::Question);
        } else {
            self.state.set(SessionState::Completed);
        }
    }

    pub fn reset_session(&mut self) {
        self.cards.set(vec![]);
        self.current_index.set(0);
        self.current_step.set(LearnStep::Question);
        self.state.set(SessionState::Settings);
    }

    pub async fn fetch_cards(&self, limit: Option<usize>) -> Result<Vec<LearnCard>, String> {
        let env = init_env().await?;
        let repo = env.get_repository().await.map_err(to_error)?;
        let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
        let items = SelectCardsToLearnUseCase::new(repo)
            .execute(user_id, false, false, limit)
            .await
            .map_err(to_error)?;
        Ok(items.into_iter().filter_map(map_item).collect::<Vec<_>>())
    }
}

fn map_item(item: StudySessionItem) -> Option<LearnCard> {
    match item {
        StudySessionItem::Vocabulary(v) => Some(LearnCard {
            id: v.card_id().to_string(),
            question: v.word().to_string(),
            answer: v.meaning().to_string(),
        }),
        StudySessionItem::Kanji(k) => Some(LearnCard {
            id: k.card_id().to_string(),
            question: k.kanji().to_string(),
            answer: k.description().to_string(),
        }),
    }
}
