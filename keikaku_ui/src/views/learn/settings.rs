use dioxus::prelude::*;

use crate::ui::{Button, ButtonVariant, Card, Paragraph, Switch, TextInput, H2};

#[component]
pub fn LearnSettings(
    limit: Signal<String>,
    show_furigana: Signal<bool>,
    loading: Signal<bool>,
    on_start: EventHandler<()>,
) -> Element {
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
                    value: Some(limit),
                    oninput: Some(EventHandler::new(move |e: Event<FormData>| limit.set(e.value()))),
                    class: None,
                    r#type: None,
                }

                Switch {
                    checked: show_furigana(),
                    onchange: move |v| show_furigana.set(v),
                    label: Some("Показывать фуригану".to_string()),
                }
            }

            Button {
                variant: ButtonVariant::Rainbow,
                class: Some("w-full".to_string()),
                onclick: move |_| on_start.call(()),
                disabled: Some(loading()),
                if loading() {
                    "Загрузка..."
                } else {
                    "Начать обучение"
                }
            }
        }
    }
}
