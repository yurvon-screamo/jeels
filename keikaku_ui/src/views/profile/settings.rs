use dioxus::prelude::*;
use keikaku::domain::UserSettings;

use crate::ui::{Button, ButtonVariant, Card, SectionHeader, TextInput};
use crate::views::profile::forms::{
    EmbeddingSettingsForm, LlmSettingsForm, TranslationSettingsForm,
};

#[component]
pub fn SettingsForm(
    settings: UserSettings,
    on_save: EventHandler<UserSettings>,
    loading: bool,
) -> Element {
    let mut llm_settings = use_signal(|| settings.llm().clone());
    let mut embedding_settings = use_signal(|| settings.embedding().clone());
    let mut translation_settings = use_signal(|| settings.translation().clone());
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
                    title: "Настройки Embedding".to_string(),
                    subtitle: Some(
                        "Модель для векторных представлений".to_string(),
                    ),
                    actions: None,
                }

                EmbeddingSettingsForm {
                    settings: embedding_settings(),
                    on_change: move |new_settings| embedding_settings.set(new_settings),
                }
            }

            Card { class: Some("space-y-4".to_string()),
                SectionHeader {
                    title: "Настройки перевода".to_string(),
                    subtitle: Some("Параметры для сервиса перевода".to_string()),
                    actions: None,
                }

                TranslationSettingsForm {
                    settings: translation_settings(),
                    on_change: move |new_settings| translation_settings.set(new_settings),
                }
            }

            Card { class: Some("space-y-4".to_string()),
                SectionHeader {
                    title: "Duolingo".to_string(),
                    subtitle: Some("JWT токен для синхронизации слов".to_string()),
                    actions: None,
                }

                TextInput {
                    label: "JWT Token",
                    value: duolingo_token,
                    placeholder: "Введите JWT токен...",
                }
            }

            div { class: "flex justify-end",
                Button {
                    variant: ButtonVariant::Rainbow,
                    onclick: move |_| {
                        let new_settings = UserSettings::new(
                            llm_settings(),
                            embedding_settings(),
                            translation_settings(),
                            Some(duolingo_token()).filter(|s| !s.trim().is_empty()),
                        );
                        on_save.call(new_settings);
                    },
                    disabled: Some(loading),
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
