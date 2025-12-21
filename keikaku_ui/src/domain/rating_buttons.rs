use dioxus::prelude::*;

use crate::components::app_ui::Paragraph;
use crate::components::button::{Button, ButtonVariant};

#[derive(Clone, PartialEq)]
pub enum Rating {
    Easy,
    Good,
    Hard,
    Again,
}

impl Rating {
    fn label(&self) -> &'static str {
        match self {
            Rating::Easy => "Легко",
            Rating::Good => "Хорошо",
            Rating::Hard => "Сложно",
            Rating::Again => "Снова",
        }
    }

    fn color(&self) -> &'static str {
        match self {
            Rating::Easy => "bg-status-perfect hover:opacity-90 text-white shadow-glow-perfect",
            Rating::Good => {
                "bg-status-good-soft hover:bg-emerald-100 text-status-good border border-emerald-200"
            }
            Rating::Hard => {
                "bg-slate-100 hover:bg-slate-200 text-status-neutral border border-slate-300"
            }
            Rating::Again => {
                "bg-status-error-soft hover:bg-red-100 text-status-error border border-red-200"
            }
        }
    }

    fn key_hint(&self) -> &'static str {
        match self {
            Rating::Easy => "1",
            Rating::Good => "2",
            Rating::Hard => "3",
            Rating::Again => "4",
        }
    }
}

#[component]
pub fn RatingButtons(on_rate: EventHandler<Rating>) -> Element {
    rsx! {
        AnswerActionButtons { on_rate }
    }
}

#[component]
pub fn QuestionActionButtons(on_show_answer: EventHandler<()>) -> Element {
    rsx! {
        div { class: "space-y-3 pt-6 justify-center flex flex-col h-full",
            div { class: "flex flex-col gap-2 flex-1",
                ActionButton {
                    label: "Показать ответ",
                    key_hint: "Пробел",
                    color_class: None,
                    class: Some("h-full".to_string()),
                    onclick: move |_| on_show_answer.call(()),
                }
            }
        }
    }
}

#[component]
pub fn AnswerActionButtons(on_rate: EventHandler<Rating>) -> Element {
    rsx! {
        div { class: "space-y-3",
            Paragraph { class: Some("text-xs text-center text-slate-500 font-semibold mb-2".to_string()),
                "Как хорошо вы знали ответ?"
            }
            div { class: "flex flex-col gap-2",
                ActionButton {
                    label: "Легко",
                    key_hint: "1",
                    color_class: Some("bg-status-perfect hover:opacity-90 text-white shadow-glow-perfect"),
                    class: None,
                    onclick: move |_| on_rate.call(Rating::Easy),
                }
                ActionButton {
                    label: "Хорошо",
                    key_hint: "2",
                    color_class: Some(
                        "bg-status-good-soft hover:bg-emerald-100 text-status-good border border-emerald-200",
                    ),
                    class: None,
                    onclick: move |_| on_rate.call(Rating::Good),
                }
                ActionButton {
                    label: "Сложно",
                    key_hint: "3",
                    color_class: Some("bg-slate-100 hover:bg-slate-200 text-status-neutral border border-slate-300"),
                    class: None,
                    onclick: move |_| on_rate.call(Rating::Hard),
                }
                ActionButton {
                    label: "Снова",
                    key_hint: "4",
                    color_class: Some(
                        "bg-status-error-soft hover:bg-red-100 text-status-error border border-red-200",
                    ),
                    class: None,
                    onclick: move |_| on_rate.call(Rating::Again),
                }
            }
            Paragraph { class: Some("text-xs text-center text-slate-400".to_string()),
                "Используйте клавиши 1-4 для оценки"
            }
        }
    }
}

#[component]
fn RatingButton(rating: Rating, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        ActionButton {
            label: rating.label(),
            key_hint: rating.key_hint(),
            color_class: Some(rating.color()),
            class: None,
            onclick,
        }
    }
}

#[component]
fn ActionButton(
    label: &'static str,
    key_hint: &'static str,
    color_class: Option<&'static str>,
    class: Option<String>,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let color_class_str = color_class.unwrap_or("");
    let additional_class = class.unwrap_or_else(|| "".to_string());
    rsx! {
        Button {
            variant: ButtonVariant::Outline,
            class: "{color_class_str} text-left {additional_class}",
            onclick: Some(onclick),
            div { class: "space-y-1",
                span { class: "block text-xs opacity-90", "Клавиша {key_hint}" }
                span { class: "block text-base", {label} }
            }
        }
    }
}
