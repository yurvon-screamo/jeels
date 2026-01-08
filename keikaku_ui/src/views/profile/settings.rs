use dioxus::prelude::*;
use keikaku::domain::UserSettings;

use crate::components::app_ui::{Card, SectionHeader};
use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use crate::views::profile::forms::LlmSettingsForm;

#[component]
pub fn SettingsForm(
    settings: UserSettings,
    on_save: EventHandler<UserSettings>,
    loading: bool,
) -> Element {
    let mut llm_settings = use_signal(|| settings.llm().clone());
    let duolingo_token = use_signal(|| {
        settings
            .duolingo_jwt_token()
            .unwrap_or_default()
            .to_string()
    });

    rsx! {
        div { class: "space-y-6",
            Card { class: Some("space-y-4".to_string()),
                SectionHeader {
                    title: "Настройки LLM".to_string(),
                    subtitle: Some(
                        "Модель для генерации контента карточек"
                            .to_string(),
                    ),
                    actions: None,
                }

                LlmSettingsForm {
                    settings: llm_settings(),
                    on_change: move |new_settings| llm_settings.set(new_settings),
                }
            }


            Card { class: Some("space-y-4".to_string()),
                SectionHeader {
                    title: "Duolingo".to_string(),
                    subtitle: Some("JWT токен для синхронизации слов".to_string()),
                    actions: None,
                }

                div { class: "space-y-2",
                    label { class: "text-sm font-medium", "JWT Token" }
                    Input {
                        placeholder: "Введите JWT токен...",
                        value: duolingo_token(),
                        oninput: {
                            let mut duolingo_token = duolingo_token;
                            move |e: FormEvent| duolingo_token.set(e.value())
                        },
                    }
                }
            }

            div { class: "flex justify-end",
                Button {
                    variant: ButtonVariant::Primary,
                    disabled: loading,
                    onclick: move |_| {
                        let new_settings = UserSettings::new(
                            llm_settings(),
                            Some(duolingo_token()).filter(|s| !s.trim().is_empty()),
                        );
                        on_save.call(new_settings);
                    },
                    if loading {
                        "Сохранение..."
                    } else {
                        "Сохранить настройки"
                    }
                }
            }
        }
    }
}
