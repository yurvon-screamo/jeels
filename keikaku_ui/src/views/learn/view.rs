use chrono::Utc;
use dioxus::{document::eval, prelude::*};

use super::{LearnActive, SessionState, StartFeedback, use_learn_session};
use crate::views::learn::session_manager::complete_lesson_impl;

#[component]
pub fn Learn() -> Element {
    let session = use_learn_session();

    let keyboard_handler = {
        let session = session.clone();
        move |e: KeyboardEvent| {
            use dioxus::prelude::Code;
            match e.code() {
                Code::Space => {
                    // Space - show answer on Question.
                    let session_data = (session.session_data)();
                    if session_data.current_step == super::LearnStep::Question {
                        e.prevent_default();
                        (session.show_answer)();
                    }
                }
                Code::Backspace => {
                    // Backspace - вернуться к предыдущей карточке
                    let session_data = (session.session_data)();
                    let is_first_card = session_data.current_index == 0;
                    let can_go_prev =
                        !is_first_card && session_data.current_step == super::LearnStep::Answer;
                    if can_go_prev {
                        e.prevent_default();
                        (session.prev_card)();
                    }
                }
                Code::KeyS => {
                    // S - пропустить карточку
                    e.prevent_default();
                    (session.next_card)();
                }
                Code::KeyQ => {
                    // Q - выйти из сессии
                    e.prevent_default();
                    let session_duration = Utc::now()
                        .signed_duration_since((session.session_data)().session_start_time);
                    spawn(async move {
                        let _ = complete_lesson_impl(session_duration).await;
                    });
                    (session.restart_session)();
                }
                Code::Digit1 => {
                    // 1 - оценить как "Легко"
                    e.prevent_default();
                    let session_data = (session.session_data)();
                    if session_data.current_step == super::LearnStep::Answer {
                        (session.rate_card)(crate::domain::Rating::Easy);
                    }
                }
                Code::Digit2 => {
                    // 2 - оценить как "Хорошо"
                    e.prevent_default();
                    let session_data = (session.session_data)();
                    if session_data.current_step == super::LearnStep::Answer {
                        (session.rate_card)(crate::domain::Rating::Good);
                    }
                }
                Code::Digit3 => {
                    // 3 - оценить как "Сложно"
                    e.prevent_default();
                    let session_data = (session.session_data)();
                    if session_data.current_step == super::LearnStep::Answer {
                        (session.rate_card)(crate::domain::Rating::Hard);
                    }
                }
                Code::Digit4 => {
                    // 4 - оценить как "Снова"
                    e.prevent_default();
                    let session_data = (session.session_data)();
                    if session_data.current_step == super::LearnStep::Answer {
                        (session.rate_card)(crate::domain::Rating::Again);
                    }
                }
                _ => {}
            }
        }
    };

    // Автоматический фокус при смене состояния
    use_effect(move || {
        let eval = eval(
            r#"
            setTimeout(() => {
                const element = document.querySelector('[data-learn-container]');
                if (element) {
                    element.focus();
                }
            }, 100);
        "#,
        );
        eval.send(()).unwrap();
    });

    let show_start_button = matches!((session.state)(), SessionState::Start);

    rsx! {
        div {
            class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6 focus:outline-none",
            tabindex: "0",
            "data-learn-container": "",
            onkeydown: keyboard_handler,
            if show_start_button {
                div { class: "space-y-4 max-w-xl mx-auto",
                    div { class: "flex justify-center",
                        crate::ui::Button {
                            variant: crate::ui::ButtonVariant::Rainbow,
                            class: Some("px-8 py-3".to_string()),
                            onclick: {
                                let session_clone = session.clone();
                                move |_| {
                                    (session_clone.start_session)();
                                }
                            },
                            disabled: None,
                            "Начать обучение"
                        }
                    }

                    {
                        let data = (session.session_data)();
                        match data.start_feedback {
                            StartFeedback::None => rsx! {},
                            StartFeedback::Empty => rsx! {
                                crate::ui::Card { class: Some("border-slate-200 bg-slate-50".to_string()),
                                    crate::ui::Paragraph { class: Some("text-slate-700".to_string()),
                                        "Нет карточек для обучения — похоже, вы всё выучили (или на сегодня ничего не запланировано)."
                                    }
                                }
                            },
                            StartFeedback::Error(ref msg) => rsx! {
                                crate::ui::Card { class: Some("border-red-200 bg-red-50".to_string()),
                                    crate::ui::Paragraph { class: Some("text-red-800".to_string()),
                                        "Не удалось загрузить карточки: {msg}"
                                    }
                                }
                            },
                        }
                    }
                }
            }
            {
                match (session.state)() {
                    SessionState::Loading => {
                        rsx! {
                            div { class: "flex items-center justify-center py-12",
                                crate::ui::LoadingState { message: Some("Загрузка карточек...".to_string()) }
                            }
                        }
                    }
                    SessionState::Active => {
                        let session_data = (session.session_data)();
                        let current_card = session_data
                            .cards
                            .get(session_data.current_index)
                            .cloned();
                        rsx! {
                            LearnActive {
                                current_card,
                                total_cards: session_data.cards.len(),
                                current_index: session_data.current_index,
                                current_step: session_data.current_step.clone(),
                                show_furigana: session_data.show_furigana,
                                native_language: keikaku::domain::value_objects::NativeLanguage::Russian,
                                on_next: EventHandler::new({
                                    let next_card = session.next_card.clone();
                                    move |_| next_card()
                                }),
                                on_show_answer: move |_| (session.show_answer)(),
                                on_prev: Some(EventHandler::new(move |_| (session.prev_card)())),
                                on_rate: EventHandler::new(move |rating: crate::domain::Rating| (session.rate_card)(rating)),
                                on_skip: EventHandler::new({
                                    let next_card = session.next_card.clone();
                                    move |_| next_card()
                                }),
                                on_quit: move |_| {
                                    let session_duration = Utc::now()
                                        .signed_duration_since((session.session_data)().session_start_time);
                                    spawn(async move {
                                        let _ = complete_lesson_impl(session_duration).await;
                                    });
                                    (session.restart_session)();
                                },
                            }
                        }
                    }
                    SessionState::Completed => {
                        // Automatically restart session when completed
                        let session_clone = session.clone();
                        spawn(async move {
                            (session_clone.restart_session)();
                        });
                        rsx! {}
                    }
                    _ => rsx! {},
                }
            }
        }
    }
}
