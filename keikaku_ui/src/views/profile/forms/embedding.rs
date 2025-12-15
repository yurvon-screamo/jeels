use dioxus::prelude::*;
use keikaku::domain::EmbeddingSettings;

use crate::ui::{Paragraph, Select, TextInput};

#[derive(Debug, Clone, PartialEq)]
pub enum EmbeddingType {
    None,
    Candle,
    OpenAi,
}

impl std::fmt::Display for EmbeddingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmbeddingType::None => write!(f, "Отключено"),
            EmbeddingType::Candle => write!(f, "Candle"),
            EmbeddingType::OpenAi => write!(f, "OpenAI"),
        }
    }
}

fn extract_openai_fields(settings: &EmbeddingSettings) -> (String, String, String) {
    if let EmbeddingSettings::OpenAi {
        model,
        base_url,
        env_var_name,
    } = settings
    {
        (model.clone(), base_url.clone(), env_var_name.clone())
    } else {
        (String::new(), String::new(), String::new())
    }
}

fn create_embedding_settings(
    embedding_type: EmbeddingType,
    openai_model: String,
    openai_base_url: String,
    openai_env_var: String,
) -> EmbeddingSettings {
    match embedding_type {
        EmbeddingType::None => EmbeddingSettings::None,
        EmbeddingType::Candle => EmbeddingSettings::Candle,
        EmbeddingType::OpenAi => EmbeddingSettings::OpenAi {
            model: openai_model,
            base_url: openai_base_url,
            env_var_name: openai_env_var,
        },
    }
}

#[component]
pub fn EmbeddingSettingsForm(
    settings: EmbeddingSettings,
    on_change: EventHandler<EmbeddingSettings>,
) -> Element {
    let mut embedding_type = use_signal(|| match &settings {
        EmbeddingSettings::None => EmbeddingType::None,
        EmbeddingSettings::Candle => EmbeddingType::Candle,
        EmbeddingSettings::OpenAi { .. } => EmbeddingType::OpenAi,
    });
    let selected_embedding_type = use_memo(move || Some(embedding_type()));

    let (openai_model_init, openai_base_url_init, openai_env_var_init) =
        extract_openai_fields(&settings);
    let openai_model = use_signal(|| openai_model_init);
    let openai_base_url = use_signal(|| openai_base_url_init);
    let openai_env_var = use_signal(|| openai_env_var_init);

    let update_settings = {
        let embedding_type = embedding_type;
        let openai_model = openai_model;
        let openai_base_url = openai_base_url;
        let openai_env_var = openai_env_var;
        let on_change = on_change;
        move || {
            let new_settings = create_embedding_settings(
                embedding_type(),
                openai_model(),
                openai_base_url(),
                openai_env_var(),
            );
            on_change.call(new_settings);
        }
    };

    rsx! {
        div { class: "space-y-4",
            Select {
                label: Some("Тип Embedding".to_string()),
                options: vec![EmbeddingType::None, EmbeddingType::Candle, EmbeddingType::OpenAi],
                selected: selected_embedding_type,
                onselect: move |new_type: EmbeddingType| {
                    embedding_type.set(new_type.clone());
                    update_settings();
                },
            }

            match embedding_type() {
                EmbeddingType::None => rsx! {
                    Paragraph { class: Some("text-slate-500".to_string()), "Embedding отключен" }
                },
                EmbeddingType::Candle => rsx! {
                    Paragraph { class: Some("text-slate-500".to_string()), "Используется Candle embedding" }
                },
                EmbeddingType::OpenAi => rsx! {
                    OpenAiEmbeddingFields {
                        model: openai_model,
                        base_url: openai_base_url,
                        env_var: openai_env_var,
                        on_change: update_settings,
                    }
                },
            }
        }
    }
}

#[component]
fn OpenAiEmbeddingFields(
    model: Signal<String>,
    base_url: Signal<String>,
    env_var: Signal<String>,
    on_change: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
            TextInput {
                label: Some("Model".to_string()),
                value: Some(model),
                placeholder: None,
                oninput: Some(
                    EventHandler::new({
                        let mut model = model;
                        let on_change = on_change;
                        move |e: Event<FormData>| {
                            model.set(e.value());
                            on_change.call(());
                        }
                    }),
                ),
                class: None,
                r#type: None,
            }
            TextInput {
                label: Some("Base URL".to_string()),
                value: Some(base_url),
                placeholder: None,
                oninput: Some(
                    EventHandler::new({
                        let mut base_url = base_url;
                        let on_change = on_change;
                        move |e: Event<FormData>| {
                            base_url.set(e.value());
                            on_change.call(());
                        }
                    }),
                ),
                class: None,
                r#type: None,
            }
            TextInput {
                label: Some("API Key Env Var".to_string()),
                value: Some(env_var),
                placeholder: None,
                oninput: Some(
                    EventHandler::new({
                        let mut env_var = env_var;
                        let on_change = on_change;
                        move |e: Event<FormData>| {
                            env_var.set(e.value());
                            on_change.call(());
                        }
                    }),
                ),
                class: None,
                r#type: None,
            }
        }
    }
}
