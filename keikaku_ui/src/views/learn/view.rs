use dioxus::prelude::*;
use keikaku::application::use_cases::select_cards_to_learn::SelectCardsToLearnUseCase;
use keikaku::domain::study_session::StudySessionItem;

use crate::ui::SectionHeader;
use crate::{ensure_user, init_env, to_error, DEFAULT_USERNAME};

use super::{
    LearnActive, LearnCompleted, LearnSettings, SessionState, LearnCard, LearnStep,
};

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

#[component]
pub fn Learn() -> Element {
    let mut state = use_signal(|| SessionState::Settings);
    let mut cards = use_signal(Vec::<LearnCard>::new);
    let mut current_index = use_signal(|| 0);
    let mut current_step = use_signal(|| LearnStep::Question);
    let mut limit = use_signal(|| "7".to_string());
    let mut show_furigana = use_signal(|| false);
    let mut loading = use_signal(|| false);

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            SectionHeader {
                title: "Обучение".to_string(),
                subtitle: Some("Изучайте и повторяйте материал".to_string()),
                actions: None,
            }

            {
                match state() {
                    SessionState::Settings => rsx! {
                        LearnSettings {
                            limit,
                            show_furigana,
                            loading,
                            on_start: move |_| {
                                let limit_val = limit().parse::<usize>().unwrap_or(7);
                                let mut state = state.clone();
                                let mut cards = cards.clone();
                                let mut current_index = current_index.clone();
                                let mut current_step = current_step.clone();
                                let mut loading = loading.clone();

                                spawn(async move {
                                    loading.set(true);
                                    state.set(SessionState::Loading);

                                    match fetch_cards_to_learn(limit_val).await {
                                        Ok(items) => {
                                            if items.is_empty() {
                                                state.set(SessionState::Settings);
                                                cards.set(vec![]);
                                            } else {
                                                let learn_cards = items.into_iter().map(map_study_item_to_learn_card).collect::<Vec<_>>();
                                                cards.set(learn_cards);
                                                current_index.set(0);
                                                current_step.set(LearnStep::Question);
                                                state.set(SessionState::Active);
                                            }
                                            loading.set(false);
                                        }
                                        Err(e) => {
                                            cards.set(vec![]);
                                            state.set(SessionState::Settings);
                                            current_step.set(LearnStep::Question);
                                            loading.set(false);
                                            error!("learn fetch error: {}", e);
                                        }
                                    }
                                });
                            }
                        }
                    },
                    SessionState::Loading => rsx! {
                        div { "Загрузка..." }
                    },
                    SessionState::Active => rsx! {
                        LearnActive {
                            cards,
                            current_index,
                            current_step,
                            show_furigana,
                            on_next: move |_| {
                                let current_index_val = current_index();
                                let cards_len = cards().len();
                                if current_index_val + 1 < cards_len {
                                    current_index.set(current_index_val + 1);
                                    current_step.set(LearnStep::Question);
                                } else {
                                    state.set(SessionState::Completed);
                                }
                            }
                        }
                    },
                    SessionState::Completed => rsx! {
                        LearnCompleted {
                            on_restart: move |_| {
                                cards.set(vec![]);
                                current_index.set(0);
                                current_step.set(LearnStep::Question);
                                state.set(SessionState::Settings);
                            }
                        }
                    },
                }
            }
        }
    }
}
