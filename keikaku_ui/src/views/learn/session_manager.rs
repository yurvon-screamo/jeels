use chrono::{DateTime, Duration, Utc};
use dioxus::prelude::*;
use keikaku::application::use_cases::{
    complete_lesson::CompleteLessonUseCase, rate_card::RateCardUseCase,
};
use keikaku::settings::ApplicationEnvironment;
use std::rc::Rc;
use ulid::Ulid;

use crate::{ensure_user, to_error, DEFAULT_USERNAME};

use super::{LearnCard, LearnStep, SessionState};

#[derive(Clone, PartialEq)]
pub struct LearnSessionData {
    pub cards: Vec<LearnCard>,
    pub current_index: usize,
    pub current_step: LearnStep,
    pub show_furigana: bool,
    pub low_stability_mode: bool,
    pub limit: Option<usize>,
    pub session_start_time: DateTime<Utc>,
}

impl Default for LearnSessionData {
    fn default() -> Self {
        Self {
            cards: vec![],
            current_index: 0,
            current_step: LearnStep::Question,
            show_furigana: true,
            low_stability_mode: false,
            limit: None,
            session_start_time: Utc::now(),
        }
    }
}

pub fn use_learn_session() -> LearnSessionSignals {
    let state = use_signal(|| SessionState::Completed);
    let session_data = use_signal(LearnSessionData::default);

    LearnSessionSignals {
        state,
        session_data,
        next_card: Rc::new(move || {
            let mut state = state;
            let mut session_data = session_data;
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
            let mut state = state;
            let mut session_data = session_data;
            *session_data.write() = LearnSessionData::default();
            state.set(SessionState::Completed);
        }),
        show_answer: Rc::new(move || {
            let mut session_data = session_data;
            session_data.write().current_step = LearnStep::Answer;
        }),
        prev_card: Rc::new(move || {
            let mut session_data = session_data;
            let mut data = session_data.write();
            if data.current_index > 0 {
                data.current_index -= 1;
                data.current_step = LearnStep::Answer;
            }
        }),
        rate_card: Rc::new(move |rating: crate::domain::Rating| {
            let state = state;
            let mut session_data = session_data;

            spawn(async move {
                let data = session_data.read();
                let current_index = data.current_index;
                let cards_len = data.cards.len();
                let card_id = data.cards.get(current_index).map(|c| c.id.clone());

                if let Some(card_id_str) = card_id {
                    if let Ok(card_ulid) = ulid::Ulid::from_string(&card_id_str) {
                        // Rate the card
                        if let Err(e) = rate_card_impl(card_ulid, rating).await {
                            error!("Failed to rate card: {:?}", e);
                        }

                        // Move to next card or complete session
                        drop(data);
                        let mut data = session_data.write();
                        data.current_step = LearnStep::Completed;

                        // Auto-advance immediately
                        drop(data);
                        let mut data = session_data.write();
                        if data.current_index + 1 < cards_len {
                            data.current_index += 1;
                            data.current_step = LearnStep::Question;
                        } else {
                            let session_start_time = data.session_start_time;
                            drop(data);
                            let mut state = state;
                            state.set(SessionState::Completed);
                            // Complete lesson
                            let session_duration =
                                Utc::now().signed_duration_since(session_start_time);
                            spawn(async move {
                                if let Err(e) = complete_lesson_impl(session_duration).await {
                                    error!("Failed to complete lesson: {:?}", e);
                                }
                            });
                        }
                    }
                }
            });
        }),
    }
}

#[derive(Clone)]
pub struct LearnSessionSignals {
    pub state: Signal<SessionState>,
    pub session_data: Signal<LearnSessionData>,
    pub next_card: Rc<dyn Fn()>,
    pub restart_session: Rc<dyn Fn()>,
    pub show_answer: Rc<dyn Fn()>,
    pub prev_card: Rc<dyn Fn()>,
    pub rate_card: Rc<dyn Fn(crate::domain::Rating)>,
}

async fn rate_card_impl(card_id: Ulid, rating: crate::domain::Rating) -> Result<(), String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let srs_service = env.get_srs_service().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let rate_usecase = RateCardUseCase::new(repo, srs_service);
    let domain_rating = match rating {
        crate::domain::Rating::Easy => keikaku::domain::Rating::Easy,
        crate::domain::Rating::Good => keikaku::domain::Rating::Good,
        crate::domain::Rating::Hard => keikaku::domain::Rating::Hard,
        crate::domain::Rating::Again => keikaku::domain::Rating::Again,
    };
    rate_usecase
        .execute(user_id, card_id, domain_rating)
        .await
        .map_err(to_error)
}

pub async fn complete_lesson_impl(lesson_duration: Duration) -> Result<(), String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let complete_usecase = CompleteLessonUseCase::new(repo);
    complete_usecase
        .execute(user_id, lesson_duration)
        .await
        .map_err(to_error)
}
