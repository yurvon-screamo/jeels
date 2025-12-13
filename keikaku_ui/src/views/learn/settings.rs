use dioxus::prelude::*;

use crate::ui::{Button, ButtonVariant, Card, Checkbox, Paragraph, Switch, TextInput, H2};

#[component]
pub fn LearnSettings(
    limit: String,
    show_furigana: bool,
    new_cards_force: bool,
    similarity_force: bool,
    loop_mod: bool,
    loading: bool,
    on_start: EventHandler<(Option<String>, bool, bool, bool, bool, bool)>,
) -> Element {
    let mut limit_signal = use_signal(|| limit.clone());
    let mut limit_enabled_signal = use_signal(|| true);
    let mut show_furigana_signal = use_signal(|| show_furigana);
    let mut new_cards_force_signal = use_signal(|| new_cards_force);
    let mut similarity_force_signal = use_signal(|| similarity_force);
    let mut loop_mod_signal = use_signal(|| loop_mod);
    rsx! {
        Card { class: Some("space-y-6".to_string()),
            H2 { class: Some("text-2xl font-bold text-slate-800".to_string()),
                "Настройки сессии"
            }
            Paragraph { class: Some("text-sm text-slate-500".to_string()),
                "Настройте параметры обучения и начните сессию"
            }

            div { class: "space-y-4",
                div { class: "space-y-2",
                    Checkbox {
                        checked: limit_enabled_signal(),
                        onchange: move |v| limit_enabled_signal.set(v),
                        label: Some("Ограничить количество карточек".to_string()),
                    }
                    if limit_enabled_signal() {
                        TextInput {
                            label: Some("Лимит карточек".to_string()),
                            placeholder: Some("7".to_string()),
                            value: Some(limit_signal),
                            oninput: Some(EventHandler::new(move |e: Event<FormData>| limit_signal.set(e.value()))),
                            class: None,
                            r#type: None,
                        }
                    }
                }

                Switch {
                    checked: show_furigana_signal(),
                    onchange: move |v| show_furigana_signal.set(v),
                    label: Some("Показывать фуригану".to_string()),
                }

                Switch {
                    checked: new_cards_force_signal(),
                    onchange: move |v| new_cards_force_signal.set(v),
                    label: Some("Только новые карточки".to_string()),
                }

                Switch {
                    checked: similarity_force_signal(),
                    onchange: move |v| similarity_force_signal.set(v),
                    label: Some("Показывать связанные карточки".to_string()),
                }

                Switch {
                    checked: loop_mod_signal(),
                    onchange: move |v| loop_mod_signal.set(v),
                    label: Some(
                        "Режим цикла (повторять после завершения)"
                            .to_string(),
                    ),
                }
            }

            Button {
                variant: ButtonVariant::Rainbow,
                class: Some("w-full".to_string()),
                onclick: move |_| {
                    let limit_value = if limit_enabled_signal() {
                        Some(limit_signal())
                    } else {
                        None
                    };
                    on_start
                        .call((
                            limit_value,
                            show_furigana_signal(),
                            similarity_force_signal(),
                            new_cards_force_signal(),
                            similarity_force_signal(),
                            loop_mod_signal(),
                        ))
                },
                disabled: Some(loading),
                if loading {
                    "Загрузка..."
                } else {
                    "Начать обучение"
                }
            }
        }
    }
}
