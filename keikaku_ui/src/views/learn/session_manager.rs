use dioxus::prelude::*;
use keikaku::application::use_cases::select_cards_to_learn::SelectCardsToLearnUseCase;
use keikaku::domain::study_session::StudySessionItem;
use std::rc::Rc;

use crate::{ensure_user, init_env, to_error, DEFAULT_USERNAME};

use super::{LearnCard, LearnStep, SessionState};

#[derive(Clone, PartialEq)]
pub struct LearnSessionData {
    pub cards: Vec<LearnCard>,
    pub current_index: usize,
    pub current_step: LearnStep,
    pub limit: String,
    pub show_furigana: bool,
}

impl Default for LearnSessionData {
    fn default() -> Self {
        Self {
            cards: vec![],
            current_index: 0,
            current_step: LearnStep::Question,
            limit: "7".to_string(),
            show_furigana: false,
        }
    }
}

pub fn use_learn_session() -> LearnSessionSignals {
    let state = use_signal(|| SessionState::Settings);
    let session_data = use_signal(LearnSessionData::default);

    LearnSessionSignals {
        state: state.clone(),
        session_data: session_data.clone(),
        start_session: Rc::new(move |limit: usize, show_furigana: bool| {
            let mut state = state.clone();
            let mut session_data = session_data.clone();

            spawn(async move {
                state.set(SessionState::Loading);

                match fetch_cards_to_learn(limit).await {
                    Ok(items) => {
                        if items.is_empty() {
                            state.set(SessionState::Settings);
                            session_data.write().cards = vec![];
                        } else {
                            let learn_cards = items
                                .into_iter()
                                .map(map_study_item_to_learn_card)
                                .collect::<Vec<_>>();
                            session_data.write().cards = learn_cards;
                            session_data.write().current_index = 0;
                            session_data.write().current_step = LearnStep::Question;
                            session_data.write().limit = limit.to_string();
                            session_data.write().show_furigana = show_furigana;
                            state.set(SessionState::Active);
                        }
                    }
                    Err(e) => {
                        session_data.write().cards = vec![];
                        state.set(SessionState::Settings);
                        session_data.write().current_step = LearnStep::Question;
                        error!("learn fetch error: {}", e);
                    }
                }
            });
        }),
        next_card: Rc::new(move || {
            let mut state = state.clone();
            let mut session_data = session_data.clone();
            let mut data = session_data.write();
            let current_index = data.current_index;
            let cards_len = data.cards.len();

            if current_index + 1 < cards_len {
                data.current_index = current_index + 1;
                data.current_step = LearnStep::Question;
            } else {
                drop(data);
                state.set(SessionState::Completed);
            }
        }),
        restart_session: Rc::new(move || {
            let mut state = state.clone();
            let mut session_data = session_data.clone();
            *session_data.write() = LearnSessionData::default();
            state.set(SessionState::Settings);
        }),
        show_answer: Rc::new(move || {
            let mut session_data = session_data.clone();
            session_data.write().current_step = LearnStep::Answer;
        }),
        prev_card: Rc::new(move || {
            let mut session_data = session_data.clone();
            let mut data = session_data.write();
            if data.current_index > 0 {
                data.current_index -= 1;
                data.current_step = LearnStep::Answer;
            }
        }),
    }
}

#[derive(Clone)]
pub struct LearnSessionSignals {
    pub state: Signal<SessionState>,
    pub session_data: Signal<LearnSessionData>,
    pub start_session: Rc<dyn Fn(usize, bool)>,
    pub next_card: Rc<dyn Fn()>,
    pub restart_session: Rc<dyn Fn()>,
    pub show_answer: Rc<dyn Fn()>,
    pub prev_card: Rc<dyn Fn()>,
}

async fn fetch_cards_to_learn(limit: usize) -> Result<Vec<StudySessionItem>, String> {
    let env = init_env().await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    SelectCardsToLearnUseCase::new(repo)
        .execute(user_id, false, false, Some(limit))
        .await
        .map_err(to_error)
}

fn map_study_item_to_learn_card(item: StudySessionItem) -> LearnCard {
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
