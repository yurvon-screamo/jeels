use dioxus::prelude::*;

use crate::ui::Paragraph;

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
            Rating::Easy => "bg-green-500 hover:bg-green-600",
            Rating::Good => "bg-blue-500 hover:bg-blue-600",
            Rating::Hard => "bg-orange-500 hover:bg-orange-600",
            Rating::Again => "bg-red-500 hover:bg-red-600",
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
        div { class: "space-y-3",
            Paragraph { class: Some("text-xs text-center text-slate-500 font-semibold mb-2".to_string()),
                "Как хорошо вы знали ответ?"
            }
            div { class: "grid grid-cols-2 gap-2",
                RatingButton {
                    rating: Rating::Easy,
                    onclick: move |_| on_rate.call(Rating::Easy),
                }
                RatingButton {
                    rating: Rating::Good,
                    onclick: move |_| on_rate.call(Rating::Good),
                }
                RatingButton {
                    rating: Rating::Hard,
                    onclick: move |_| on_rate.call(Rating::Hard),
                }
                RatingButton {
                    rating: Rating::Again,
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
        button {
            class: "{rating.color()} text-white font-bold py-4 px-4 rounded-xl shadow-md hover:shadow-lg active:scale-95 transition-all duration-200 text-sm",
            onclick: move |e| onclick.call(e),
            div { class: "space-y-1",
                span { class: "block text-xs opacity-90", "Клавиша {rating.key_hint()}" }
                span { class: "block text-base", {rating.label()} }
            }
        }
    }
}
