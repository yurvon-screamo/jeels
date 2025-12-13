use dioxus::prelude::*;

use super::{LearnCardDisplay, LearnNavigation, LearnProgress};

#[component]
pub fn LearnActive(
    cards: Vec<super::LearnCard>,
    current_index: usize,
    current_step: super::LearnStep,
    show_furigana: bool,
    similarity_shown: bool,
    on_next: EventHandler<()>,
    on_show_answer: EventHandler<()>,
    on_prev: Option<EventHandler<()>>,
    on_rate: EventHandler<crate::domain::Rating>,
    on_toggle_similarity: EventHandler<()>,
    on_skip: EventHandler<()>,
    on_quit: EventHandler<()>,
) -> Element {
    let progress = {
        let current = current_index;
        let total = cards.len();
        if total > 0 {
            (current as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    };

    rsx! {
        div {
            class: "space-y-6",
            tabindex: "0",
            onkeydown: {
                let current_step_clone = current_step.clone();
                move |e: KeyboardEvent| {
                    use dioxus::prelude::Code;
                    match e.code() {
                        Code::Space => {
                            // Пробел - показать ответ или перейти дальше
                            e.prevent_default();
                            if current_step_clone == super::LearnStep::Question {
                                on_show_answer.call(());
                            } else if current_step_clone == super::LearnStep::Answer {
                                on_next.call(());
                            }
                        }
                        Code::Backspace => {
                            // Backspace - вернуться к предыдущей карточке
                            if let Some(on_prev) = on_prev.clone() {
                                let is_first_card = current_index == 0;
                                let can_go_prev = !is_first_card
                                    && current_step_clone == super::LearnStep::Answer;
                                if can_go_prev {
                                    e.prevent_default();
                                    on_prev.call(());
                                }
                            }
                        }
                        Code::KeyH => {
                            // H - переключить связанные карточки
                            e.prevent_default();
                            on_toggle_similarity.call(());
                        }
                        Code::KeyS => {
                            // S - пропустить карточку
                            e.prevent_default();
                            on_skip.call(());
                        }
                        Code::KeyQ => {
                            // Q - выйти из сессии
                            e.prevent_default();
                            on_quit.call(());
                        }
                        Code::Digit1 => {
                            // 1 - оценить как "Легко"
                            e.prevent_default();
                            if current_step_clone == super::LearnStep::Answer {
                                on_rate.call(crate::domain::Rating::Easy);
                            }
                        }
                        Code::Digit2 => {
                            // 2 - оценить как "Хорошо"
                            e.prevent_default();
                            if current_step_clone == super::LearnStep::Answer {
                                on_rate.call(crate::domain::Rating::Good);
                            }
                        }
                        Code::Digit3 => {
                            // 3 - оценить как "Сложно"
                            e.prevent_default();
                            if current_step_clone == super::LearnStep::Answer {
                                on_rate.call(crate::domain::Rating::Hard);
                            }
                        }
                        Code::Digit4 => {
                            // 4 - оценить как "Снова"
                            e.prevent_default();
                            if current_step_clone == super::LearnStep::Answer {
                                on_rate.call(crate::domain::Rating::Again);
                            }
                        }
                        _ => {}
                    }
                }
            },

            LearnProgress {
                current: current_index + 1,
                total: cards.len(),
                progress,
            }

            LearnCardDisplay {
                cards: cards.clone(),
                current_index,
                current_step: current_step.clone(),
                show_furigana,
                similarity_shown,
                on_show_answer,
                on_next,
                on_rate,
                on_toggle_similarity,
            }

            LearnNavigation {
                current_index,
                total_cards: cards.len(),
                current_step: current_step.clone(),
                on_next,
                on_prev: on_prev.clone(),
            }
        }
    }
}
