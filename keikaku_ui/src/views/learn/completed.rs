use dioxus::prelude::*;

use crate::ui::{Button, ButtonVariant, Card, H2, Paragraph};

#[component]
pub fn LearnCompleted() -> Element {
    rsx! {
        Card { class: Some("space-y-6 text-center py-12".to_string()),
            div { class: "text-6xl mb-4", "🎉" }
            H2 { class: Some("text-3xl font-bold text-slate-800".to_string()),
                "Сессия завершена!"
            }
            Paragraph { class: Some("text-slate-600".to_string()),
                "Вы прошли все карточки в этой сессии"
            }
            Button {
                variant: ButtonVariant::Rainbow,
                class: Some("w-full max-w-xs mx-auto".to_string()),
                onclick: move |_| {},
                "Начать новую сессию"
            }
        }
    }
}
