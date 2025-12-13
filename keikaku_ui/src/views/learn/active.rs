use dioxus::prelude::*;

use crate::ui::Card;

use super::use_cases::QuestionView;

#[component]
#[component]
pub fn LearnActive(
    cards: Signal<Vec<super::LearnCard>>,
    current_index: Signal<usize>,
    current_step: Signal<super::LearnStep>,
    show_furigana: Signal<bool>,
    on_next: EventHandler<()>,
) -> Element {
    let progress = {
        let current = current_index() + 1;
        let total = cards().len();
        if total > 0 {
            (current as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    };

    rsx! {
        div { class: "space-y-6", tabindex: "0",

            LearnProgress {
                current: current_index() + 1,
                total: cards().len(),
                progress,
            }

            LearnCardDisplay {
                cards,
                current_index,
                current_step,
                show_furigana,
                on_show_answer: move |_| current_step.set(super::LearnStep::Answer),
                on_next,
            }

            LearnNavigation {
                cards,
                current_index,
                on_next,
            }
        }
    }
}

#[component]
pub fn LearnProgress(current: usize, total: usize, progress: f64) -> Element {
    rsx! {
        Card { class: Some("space-y-3".to_string()),
            div { class: "flex items-center justify-between text-sm",
                span { class: "font-semibold text-slate-700", "Прогресс" }
                span { class: "text-slate-500", "{current} из {total}" }
            }
            div { class: "w-full h-3 bg-slate-100 rounded-full overflow-hidden",
                div {
                    class: "h-full bg-rainbow-vibrant rounded-full transition-all duration-500 ease-out",
                    style: "width: {progress}%",
                }
            }
        }
    }
}

#[component]
pub fn LearnCardDisplay(
    cards: Signal<Vec<super::LearnCard>>,
    current_index: Signal<usize>,
    current_step: Signal<super::LearnStep>,
    show_furigana: Signal<bool>,
    on_show_answer: EventHandler<()>,
    on_next: EventHandler<()>,
) -> Element {
    let card = cards().get(current_index()).cloned();

    if let Some(card) = card {
        rsx! {
            Card { class: Some("space-y-4".to_string()),
                if current_step() == super::LearnStep::Question {
                    QuestionView {
                        question: card.question,
                        show_furigana: show_furigana(),
                        on_show_answer: move |_| on_show_answer.call(()),
                    }
                } else {
                    crate::domain::CardAnswer {
                        question: card.question,
                        answer: card.answer,
                        show_furigana: show_furigana(),
                        examples: None,
                    }
                }
            }
        }
    } else {
        rsx! {
            Card { class: Some("space-y-4".to_string()),
                crate::ui::Paragraph { class: Some("text-sm text-slate-500 text-center".to_string()),
                    "Нет карточек для отображения"
                }
            }
        }
    }
}

#[component]
#[component]
pub fn LearnNavigation(
    cards: Signal<Vec<super::LearnCard>>,
    current_index: Signal<usize>,
    on_next: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "flex gap-2",
            crate::ui::Button {
                variant: crate::ui::ButtonVariant::Outline,
                class: Some("w-full".to_string()),
                onclick: move |_| on_next.call(()),
                "Далее"
            }
        }
    }
}
