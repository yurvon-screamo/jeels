use dioxus::prelude::*;
use keikaku::domain::EmbeddingSettings;

use crate::components::app_ui::Paragraph;
use crate::components::input::Input;
use crate::components::select::{
    Select, SelectItemIndicator, SelectList, SelectOption, SelectTrigger, SelectValue,
};

#[derive(Debug, Clone, PartialEq)]
pub enum EmbeddingType {
    None,
    OpenAi,
}

impl std::fmt::Display for EmbeddingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmbeddingType::None => write!(f, "Отключено"),
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
        EmbeddingSettings::OpenAi { .. } => EmbeddingType::OpenAi,
    });

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

    let embedding_type_value = match embedding_type() {
        EmbeddingType::None => "none",
        EmbeddingType::OpenAi => "openai",
    }
    .to_string();

    rsx! {
        div { class: "space-y-4",
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "Тип Embedding" }
                Select::<String> {
                    value: Some(Some(embedding_type_value)),
                    on_value_change: move |v: Option<String>| {
                        if let Some(v) = v {
                            let next = match v.as_str() {
                                "openai" => EmbeddingType::OpenAi,
                                _ => EmbeddingType::None,
                            };
                            embedding_type.set(next);
                            update_settings();
                        }
                    },
                    placeholder: "Выберите...",
                    SelectTrigger { aria_label: "Тип Embedding", width: "100%", SelectValue {} }
                    SelectList { aria_label: "Тип Embedding",
                        SelectOption::<String> { index: 0usize, value: "none".to_string(),
                            "Отключено"
                            SelectItemIndicator {}
                        }
                        SelectOption::<String> { index: 2usize, value: "openai".to_string(),
                            "OpenAI"
                            SelectItemIndicator {}
                        }
                    }
                }
            }

            match embedding_type() {
                EmbeddingType::None => rsx! {
                    Paragraph { class: Some("text-slate-500".to_string()), "Embedding отключен" }
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
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "Model" }
                Input {
                    value: model(),
                    oninput: {
                        let mut model = model;
                        let on_change = on_change;
                        move |e: FormEvent| {
                            model.set(e.value());
                            on_change.call(());
                        }
                    },
                }
            }
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "Base URL" }
                Input {
                    value: base_url(),
                    oninput: {
                        let mut base_url = base_url;
                        let on_change = on_change;
                        move |e: FormEvent| {
                            base_url.set(e.value());
                            on_change.call(());
                        }
                    },
                }
            }
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "API Key Env Var" }
                Input {
                    value: env_var(),
                    oninput: {
                        let mut env_var = env_var;
                        let on_change = on_change;
                        move |e: FormEvent| {
                            env_var.set(e.value());
                            on_change.call(());
                        }
                    },
                }
            }
        }
    }
}
