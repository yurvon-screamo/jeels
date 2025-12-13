use dioxus::prelude::*;

use crate::ui::{Button, ButtonVariant, Card, H2, Paragraph, TextInput, Switch};

#[component]
pub fn LearnSettings() -> Element {
    rsx! {
        Card { class: Some("space-y-6".to_string()),
            H2 { class: Some("text-2xl font-bold text-slate-800".to_string()),
                "Настройки сессии"
            }
            Paragraph { class: Some("text-sm text-slate-500".to_string()),
                "Настройте параметры обучения и начните сессию"
            }

            div { class: "space-y-4",
                TextInput {
                    label: Some("Лимит карточек".to_string()),
                    placeholder: Some("7".to_string()),
                    value: None,
                    oninput: None,
                    class: None,
                    r#type: None,
                }

                Switch {
                    checked: false,
                    onchange: |_| {},
                    label: Some("Показывать фуригану".to_string()),
                }
            }

            Button {
                variant: ButtonVariant::Rainbow,
                class: Some("w-full".to_string()),
                onclick: move |_| {},
                "Начать обучение"
            }
        }
    }
}
