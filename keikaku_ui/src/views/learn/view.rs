use dioxus::prelude::*;

use crate::ui::SectionHeader;

use super::{use_learn_session, LearnActive, LearnCompleted, LearnSettings, SessionState};

#[component]
pub fn Learn() -> Element {
    let session = use_learn_session();

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            SectionHeader {
                title: "Обучение".to_string(),
                subtitle: Some("Изучайте и повторяйте материал".to_string()),
                actions: None,
            }

            {
                match (session.state)() {
                    SessionState::Settings => rsx! {
                        LearnSettings {
                            limit: (session.session_data)().limit.clone(),
                            show_furigana: (session.session_data)().show_furigana,
                            loading: false,
                            on_start: move |(limit_str, show_furigana_val): (String, bool)| {
                                let limit_val = limit_str.parse::<usize>().unwrap_or(7);
                                (session.start_session)(limit_val, show_furigana_val);
                            },
                        }
                    },
                    SessionState::Loading => rsx! {
                        div { class: "flex items-center justify-center py-12",
                            crate::ui::LoadingState { message: Some("Загрузка карточек...".to_string()) }
                        }
                    },
                    SessionState::Active => rsx! {
                        LearnActive {
                            cards: (session.session_data)().cards.clone(),
                            current_index: (session.session_data)().current_index,
                            current_step: (session.session_data)().current_step.clone(),
                            show_furigana: (session.session_data)().show_furigana,
                            on_next: move |_| (session.next_card)(),
                            on_show_answer: move |_| (session.show_answer)(),
                            on_prev: Some(EventHandler::new(move |_| (session.prev_card)())),
                        }
                    },
                    SessionState::Completed => rsx! {
                        LearnCompleted {
                            total_cards: (session.session_data)().cards.len(),
                            on_restart: move |_| (session.restart_session)(),
                        }
                    },
                }
            }
        }
    }
}
