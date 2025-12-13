use dioxus::prelude::*;

use crate::domain::{CardAnswer, Rating, RatingButtons, WordCard};
use crate::ui::{Button, ButtonVariant, Paragraph};
use crate::components::{handle_key_event, KeyAction};

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
                Paragraph { class: Some("text-xs text-center text-slate-400".to_string()),
                    "Нажмите Пробел или кнопку выше"
                }
            }
        }
    }
}

#[component]
pub fn AnswerView(
    question: String,
    answer: String,
    show_furigana: bool,
    on_rate: EventHandler<u8>,
) -> Element {
    rsx! {
        div { class: "space-y-4",
            CardAnswer {
                question,
                answer,
                show_furigana,
                examples: None,
            }
            RatingSection { on_rate }
        }
    }
}

#[component]
pub fn RatingSection(on_rate: EventHandler<u8>) -> Element {
    rsx! {
        RatingButtons {
            on_rate: move |rating| {
                // Преобразовать Rating в u8
                let rating_value = match rating {
                    Rating::Easy => 1,
                    Rating::Good => 2,
                    Rating::Hard => 3,
                    Rating::Again => 4,
                };
                on_rate.call(rating_value);
            }
        }
    }
}

#[component]
pub fn QuestionCard(question: String, show_furigana: bool) -> Element {
    rsx! {
        WordCard { text: question, show_furigana }
    }
}

#[component]
pub fn AnswerCard(question: String, answer: String, show_furigana: bool) -> Element {
    rsx! {
        CardAnswer {
            question,
            answer,
            show_furigana,
            examples: None,
        }
    }
}

#[component]
pub fn RatingButton(rating: u8, label: String, color: String, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            class: "{color} text-white font-bold py-4 px-4 rounded-xl shadow-md hover:shadow-lg active:scale-95 transition-all duration-200 text-sm",
            onclick: move |e| onclick.call(e),
            div { class: "space-y-1",
                span { class: "block text-xs opacity-90", "Клавиша {rating}" }
                span { class: "block text-base", {label} }
            }
        }
    }
}

pub fn handle_key_action(action: KeyAction) {
    // Логика обработки клавиш
    match action {
        KeyAction::ShowAnswer => {
            // Показать ответ
        }
        KeyAction::RateEasy
        | KeyAction::RateGood
        | KeyAction::RateHard
        | KeyAction::RateAgain => {
            // Оценить карточку
        }
        KeyAction::Skip => {
            // Пропустить
        }
    }
}
