use dioxus::prelude::*;

use crate::domain::WordCard;
use crate::ui::Card;
use crate::ui::{Button, ButtonVariant};

#[component]
pub fn LearnCardDisplay(
    cards: Vec<super::LearnCard>,
    current_index: usize,
    current_step: super::LearnStep,
    show_furigana: bool,
    on_show_answer: EventHandler<()>,
    on_next: EventHandler<()>,
) -> Element {
    let card = cards.get(current_index).cloned();

    if let Some(card) = card {
        rsx! {
            Card {
                class: Some(
                    format!(
                        "space-y-4 transition-all duration-300 {}",
                        if current_step == super::LearnStep::Question {
                            "border-l-4 border-l-blue-400"
                        } else {
                            "border-l-4 border-l-green-400"
                        },
                    ),
                ),

                // Индикатор состояния
                div { class: "flex items-center gap-2 mb-2",
                    if current_step == super::LearnStep::Question {
                        div { class: "w-2 h-2 bg-blue-400 rounded-full animate-pulse" }
                        span { class: "text-xs text-blue-600 font-medium", "Вопрос" }
                    } else {
                        div { class: "w-2 h-2 bg-green-400 rounded-full" }
                        span { class: "text-xs text-green-600 font-medium", "Ответ" }
                    }
                }

                if current_step == super::LearnStep::Question {
                    QuestionView {
                        question: card.question,
                        show_furigana,
                        on_show_answer: move |_| on_show_answer.call(()),
                    }
                } else {
                    crate::domain::CardAnswer {
                        question: card.question,
                        answer: card.answer,
                        show_furigana,
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
pub fn QuestionView(
    question: String,
    show_furigana: bool,
    on_show_answer: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "space-y-4",
            WordCard { text: question, show_furigana }
            div { class: "space-y-2",
                Button {
                    variant: ButtonVariant::Rainbow,
                    class: Some("w-full".to_string()),
                    onclick: on_show_answer,
                    "Показать ответ (Пробел)"
                }
                div { class: "flex flex-col gap-1 text-xs text-center text-slate-400",
                    p { "Нажмите Пробел, чтобы показать ответ" }
                    p { "Нажмите Enter, чтобы перейти дальше" }
                }
            }
        }
    }
}
