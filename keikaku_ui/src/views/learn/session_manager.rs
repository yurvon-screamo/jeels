use crate::views::learn::learn_session::CardType;
use chrono::{DateTime, Duration, Utc};
use dioxus::prelude::*;
use keikaku::application::UserRepository;
use keikaku::application::use_cases::{
    complete_lesson::CompleteLessonUseCase, rate_card::RateCardUseCase,
    select_cards_to_fixation::SelectCardsToFixationUseCase,
    select_cards_to_lesson::SelectCardsToLessonUseCase,
};
use keikaku::domain::knowledge::Card;
use keikaku::settings::ApplicationEnvironment;
use std::collections::HashMap;
use std::rc::Rc;
use ulid::Ulid;

use crate::{DEFAULT_USERNAME, ensure_user, to_error};

use super::{LearnCard, LearnStep, SessionState, StartFeedback};

#[derive(Clone, PartialEq)]
pub struct LearnSessionData {
    pub cards: Vec<LearnCard>,
    pub current_index: usize,
    pub current_step: LearnStep,
    pub show_furigana: bool,
    pub limit: Option<usize>,
    pub session_start_time: DateTime<Utc>,
    pub start_feedback: StartFeedback,
}

impl Default for LearnSessionData {
    fn default() -> Self {
        Self {
            cards: vec![],
            current_index: 0,
            current_step: LearnStep::Question,
            show_furigana: true,
            limit: None,
            session_start_time: Utc::now(),
            start_feedback: StartFeedback::None,
        }
    }
}

pub fn use_learn_session() -> LearnSessionSignals {
    let state = use_signal(|| SessionState::Start);
    let session_data = use_signal(LearnSessionData::default);

    LearnSessionSignals {
        state,
        session_data,
        start_session: Rc::new(move || {
            let mut state = state;
            let mut session_data = session_data;

            spawn(async move {
                state.set(SessionState::Loading);
                session_data.write().start_feedback = StartFeedback::None;

                match fetch_cards_to_learn().await {
                    Ok(items) => {
                        if items.is_empty() {
                            state.set(SessionState::Start);
                            session_data.write().cards = vec![];
                            session_data.write().start_feedback = StartFeedback::Empty;
                        } else {
                            let learn_cards = items
                                .into_iter()
                                .map(map_study_item_to_learn_card)
                                .collect::<Vec<_>>();
                            session_data.write().cards = learn_cards;
                            session_data.write().current_index = 0;
                            session_data.write().current_step = LearnStep::Question;
                            session_data.write().session_start_time = Utc::now();

                            let env = ApplicationEnvironment::get();
                            if let Ok(repo) = env.get_repository().await
                                && let Ok(user_id) = ensure_user(env, DEFAULT_USERNAME).await
                                && let Ok(user) = repo.find_by_id(user_id).await
                                && let Some(_user) = user
                            {
                                session_data.write().show_furigana = true;
                            }

                            state.set(SessionState::Active);
                        }
                    }
                    Err(e) => {
                        session_data.write().cards = vec![];
                        state.set(SessionState::Start);
                        session_data.write().current_step = LearnStep::Question;
                        session_data.write().start_feedback = StartFeedback::Error(e.clone());
                        error!("learn fetch error: {}", e);
                    }
                }
            });
        }),
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
            state.set(SessionState::Start);
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
        start_high_difficulty_session: Rc::new(move || {
            let mut state = state;
            let mut session_data = session_data;

            spawn(async move {
                state.set(SessionState::Loading);
                session_data.write().start_feedback = StartFeedback::None;

                match fetch_high_difficulty_cards().await {
                    Ok(items) => {
                        if items.is_empty() {
                            state.set(SessionState::Start);
                            session_data.write().cards = vec![];
                            session_data.write().start_feedback = StartFeedback::Empty;
                        } else {
                            let learn_cards = items
                                .into_iter()
                                .map(map_card_to_learn_card)
                                .collect::<Vec<_>>();
                            session_data.write().cards = learn_cards;
                            session_data.write().current_index = 0;
                            session_data.write().current_step = LearnStep::Question;
                            session_data.write().session_start_time = Utc::now();

                            let env = ApplicationEnvironment::get();
                            if let Ok(repo) = env.get_repository().await
                                && let Ok(user_id) = ensure_user(env, DEFAULT_USERNAME).await
                                && let Ok(user) = repo.find_by_id(user_id).await
                                && let Some(_user) = user
                            {
                                session_data.write().show_furigana = true;
                            }

                            state.set(SessionState::Active);
                        }
                    }
                    Err(e) => {
                        session_data.write().cards = vec![];
                        state.set(SessionState::Start);
                        session_data.write().current_step = LearnStep::Question;
                        session_data.write().start_feedback = StartFeedback::Error(e.clone());
                        error!("high difficulty fetch error: {}", e);
                    }
                }
            });
        }),
        rate_card: Rc::new(move |rating: crate::domain::Rating| {
            let state = state;
            let mut session_data = session_data;

            spawn(async move {
                let data = session_data.read();
                let current_index = data.current_index;
                let cards_len = data.cards.len();
                let card_id = data.cards.get(current_index).map(|c| c.id.clone());

                if let Some(card_id_str) = card_id
                    && let Ok(card_ulid) = ulid::Ulid::from_string(&card_id_str)
                {
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
                        let session_duration = Utc::now().signed_duration_since(session_start_time);
                        spawn(async move {
                            if let Err(e) = complete_lesson_impl(session_duration).await {
                                error!("Failed to complete lesson: {:?}", e);
                            }
                        });
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
    pub start_session: Rc<dyn Fn()>,
    pub next_card: Rc<dyn Fn()>,
    pub restart_session: Rc<dyn Fn()>,
    pub show_answer: Rc<dyn Fn()>,
    pub prev_card: Rc<dyn Fn()>,
    pub rate_card: Rc<dyn Fn(crate::domain::Rating)>,
    pub start_high_difficulty_session: Rc<dyn Fn()>,
}

async fn fetch_cards_to_learn() -> Result<HashMap<Ulid, Card>, String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    SelectCardsToLessonUseCase::new(repo)
        .execute(user_id)
        .await
        .map_err(to_error)
}

async fn fetch_high_difficulty_cards() -> Result<Vec<Card>, String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    SelectCardsToFixationUseCase::new(repo)
        .execute(user_id)
        .await
        .map_err(to_error)
}

fn map_card_to_learn_card(card: Card) -> LearnCard {
    match card {
        Card::Vocabulary(v) => LearnCard {
            id: ulid::Ulid::new().to_string(), // Generate a temporary ID
            card_type: CardType::Vocabulary,
            question: v.word().text().to_string(),
            answer: v.meaning().text().to_string(),
            example_phrases: v.example_phrases().to_vec(),
            kanji_info: v
                .get_kanji_cards(&keikaku::domain::value_objects::JapaneseLevel::N5)
                .into_iter()
                .cloned()
                .collect(), // TODO: Use proper level
            example_words: vec![],
            radicals: vec![],
            jlpt_level: keikaku::domain::value_objects::JapaneseLevel::N5, // TODO: Add proper level
            markdown_description: None,
        },
        Card::Kanji(k) => LearnCard {
            id: ulid::Ulid::new().to_string(), // Generate a temporary ID
            card_type: CardType::Kanji,
            question: k.kanji().text().to_string(),
            answer: k.description().text().to_string(),
            example_phrases: vec![],
            kanji_info: vec![],
            example_words: k.example_words().to_vec(),
            radicals: k
                .radicals_info()
                .unwrap_or_default()
                .into_iter()
                .cloned()
                .collect(),
            jlpt_level: k.jlpt(),
            markdown_description: None,
        },
        Card::Grammar(g) => LearnCard {
            id: ulid::Ulid::new().to_string(), // Generate a temporary ID
            card_type: CardType::Grammar,
            question: g.title().text().to_string(),
            answer: g.description().text().to_string(),
            example_phrases: vec![],
            kanji_info: vec![],
            example_words: vec![],
            radicals: vec![],
            jlpt_level: keikaku::domain::value_objects::JapaneseLevel::N5, // TODO: Grammar rules don't have JLPT level
            markdown_description: Some(g.description().text().to_string()),
        },
    }
}

fn map_study_item_to_learn_card((card_id, card): (Ulid, Card)) -> LearnCard {
    match card {
        Card::Vocabulary(v) => LearnCard {
            id: card_id.to_string(),
            card_type: CardType::Vocabulary,
            question: v.word().text().to_string(),
            answer: v.meaning().text().to_string(),
            example_phrases: v.example_phrases().to_vec(),
            kanji_info: v
                .get_kanji_cards(&keikaku::domain::value_objects::JapaneseLevel::N5)
                .into_iter()
                .cloned()
                .collect(), // TODO: Use proper level
            example_words: vec![],
            radicals: vec![],
            jlpt_level: keikaku::domain::value_objects::JapaneseLevel::N5, // TODO: Add proper level
            markdown_description: None,
        },
        Card::Kanji(k) => LearnCard {
            id: card_id.to_string(),
            card_type: CardType::Kanji,
            question: k.kanji().text().to_string(),
            answer: k.description().text().to_string(),
            example_phrases: vec![],
            kanji_info: vec![],
            example_words: k.example_words().to_vec(),
            radicals: k
                .radicals_info()
                .unwrap_or_default()
                .into_iter()
                .cloned()
                .collect(),
            jlpt_level: k.jlpt(),
            markdown_description: None,
        },
        Card::Grammar(g) => LearnCard {
            id: card_id.to_string(),
            card_type: CardType::Grammar,
            question: g.title().text().to_string(),
            answer: g.description().text().to_string(),
            example_phrases: vec![],
            kanji_info: vec![],
            example_words: vec![],
            radicals: vec![],
            jlpt_level: keikaku::domain::value_objects::JapaneseLevel::N5, // TODO: Grammar rules don't have JLPT level
            markdown_description: Some(g.description().text().to_string()),
        },
    }
}

async fn rate_card_impl(card_id: Ulid, rating: crate::domain::Rating) -> Result<(), String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let srs_service = env.get_srs_service().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let rate_usecase = RateCardUseCase::new(repo, srs_service);
    // Convert UI Rating to domain Rating
    let domain_rating = match rating {
        crate::domain::Rating::Easy => keikaku::domain::Rating::Easy,
        crate::domain::Rating::Good => keikaku::domain::Rating::Good,
        crate::domain::Rating::Hard => keikaku::domain::Rating::Hard,
        crate::domain::Rating::Again => keikaku::domain::Rating::Again,
    };
    rate_usecase
        .execute(
            user_id,
            card_id,
            keikaku::application::srs_service::RateMode::Standard,
            domain_rating,
        )
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
