use dioxus::prelude::*;

use super::{LearnCardDisplay, LearnNavigation, LearnProgress};

#[component]
pub fn LearnActive(
    cards: Vec<super::LearnCard>,
    current_index: usize,
    current_step: super::LearnStep,
    show_furigana: bool,
    on_next: EventHandler<()>,
    on_show_answer: EventHandler<()>,
    on_prev: Option<EventHandler<()>>,
) -> Element {
    let progress = {
        let current = current_index + 1;
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
                            } else {
                                on_next.call(());
                            }
                        }
                        Code::Enter => {
                            // Enter - перейти дальше
                            e.prevent_default();
                            on_next.call(());
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
                on_show_answer,
                on_next,
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
