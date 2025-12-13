use dioxus::prelude::*;

use super::{use_learn_session, LearnActive, LearnCompleted, LearnSettings, SessionState};

#[component]
pub fn Learn() -> Element {
    let session = use_learn_session();

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            {
                match (session.state)() {
                    SessionState::Settings => {
                        rsx! {
                            LearnSettings {
                                limit: (session.session_data)().limit.clone().unwrap_or_else(|| "7".to_string()),
                                show_furigana: (session.session_data)().show_furigana,
                                new_cards_force: (session.session_data)().new_cards_force,
                                similarity_force: (session.session_data)().similarity_force,
                                loop_mod: (session.session_data)().loop_mod,
                                loading: false,
                                on_start: move |
                                    (
                                        // Complete lesson and reset
                                        // Import and call complete_lesson_impl
                                        limit_opt,
                                        show_furigana_val,
                                        similarity_shown_val,
                                        new_cards_force_val,
                                        similarity_force_val,
                                        loop_mod_val,
                                    ): (Option<String>, bool, bool, bool, bool, bool)|
                                {
                                    let limit_val = limit_opt.and_then(|s| s.parse::<usize>().ok());
                                    (session
                                        .start_session)(
                                        limit_val,
                                        show_furigana_val,
                                        similarity_shown_val,
                                        new_cards_force_val,
                                        similarity_force_val,
                                        loop_mod_val,
                                    );
                                },
                            }
                        }
                    }
                    SessionState::Loading => {
                        rsx! {
                            div { class: "flex items-center justify-center py-12",
                                crate::ui::LoadingState { message: Some("Загрузка карточек...".to_string()) }
                            }
                        }
                    }
                    SessionState::Active => {
                        rsx! {
                            LearnActive {
                                cards: (session.session_data)().cards.clone(),
                                current_index: (session.session_data)().current_index,
                                current_step: (session.session_data)().current_step.clone(),
                                show_furigana: (session.session_data)().show_furigana,
                                similarity_shown: (session.session_data)().similarity_shown,
                                on_next: EventHandler::new({
                                    let next_card = session.next_card.clone();
                                    move |_| next_card()
                                }),
                                on_show_answer: move |_| (session.show_answer)(),
                                on_prev: Some(EventHandler::new(move |_| (session.prev_card)())),
                                on_rate: EventHandler::new(move |rating: crate::domain::Rating| (session.rate_card)(rating)),
                                on_toggle_similarity: move |_| (session.toggle_similarity)(),
                                on_skip: EventHandler::new({
                                    let next_card = session.next_card.clone();
                                    move |_| next_card()
                                }),
                                on_quit: move |_| {
                                    spawn(async move {
                                        use crate::views::learn::session_manager::complete_lesson_impl;
                                        let _ = complete_lesson_impl().await;
                                    });
                                    (session.restart_session)();
                                },
                            }
                        }
                    }
                    SessionState::Completed => {
                        rsx! {
                            LearnCompleted {
                                total_cards: (session.session_data)().cards.len(),
                                on_restart: move |_| (session.restart_session)(),
                            }
                        }
                    }
                }
            }
        }
    }
}
