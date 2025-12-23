use dioxus::prelude::*;
use keikaku::domain::LlmSettings;

use crate::components::app_ui::Paragraph;
use crate::components::input::Input;
use crate::components::select::{
    Select, SelectItemIndicator, SelectList, SelectOption, SelectTrigger, SelectValue,
};

#[derive(Debug, Clone, PartialEq)]
pub enum LlmType {
    None,
    Gemini,
    OpenAi,
}

impl std::fmt::Display for LlmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmType::None => write!(f, "Отключено"),
            LlmType::Gemini => write!(f, "Gemini"),
            LlmType::OpenAi => write!(f, "OpenAI"),
        }
    }
}

fn extract_gemini_fields(settings: &LlmSettings) -> (String, String) {
    if let LlmSettings::Gemini { temperature, model } = settings {
        (temperature.to_string(), model.clone())
    } else {
        (String::new(), String::new())
    }
}

fn extract_openai_fields(settings: &LlmSettings) -> (String, String, String, String) {
    if let LlmSettings::OpenAi {
        temperature,
        model,
        base_url,
        env_var_name,
    } = settings
    {
        (
            temperature.to_string(),
            model.clone(),
            base_url.clone(),
            env_var_name.clone(),
        )
    } else {
        (String::new(), String::new(), String::new(), String::new())
    }
}

fn create_llm_settings(
    llm_type: LlmType,
    gemini_temp: String,
    gemini_model: String,
    openai_temp: String,
    openai_model: String,
    openai_base_url: String,
    openai_env_var: String,
) -> LlmSettings {
    match llm_type {
        LlmType::None => LlmSettings::None,
        LlmType::Gemini => LlmSettings::Gemini {
            temperature: gemini_temp.parse().unwrap_or(0.0),
            model: gemini_model,
        },
        LlmType::OpenAi => LlmSettings::OpenAi {
            temperature: openai_temp.parse().unwrap_or(0.0),
            model: openai_model,
            base_url: openai_base_url,
            env_var_name: openai_env_var,
        },
    }
}

#[component]
pub fn LlmSettingsForm(settings: LlmSettings, on_change: EventHandler<LlmSettings>) -> Element {
    let mut llm_type = use_signal(|| match &settings {
        LlmSettings::None => LlmType::None,
        LlmSettings::Gemini { .. } => LlmType::Gemini,
        LlmSettings::OpenAi { .. } => LlmType::OpenAi,
    });
    let llm_value = match llm_type() {
        LlmType::None => "none",
        LlmType::Gemini => "gemini",
        LlmType::OpenAi => "openai",
    }
    .to_string();

    let (gemini_temp_init, gemini_model_init) = extract_gemini_fields(&settings);
    let gemini_temperature = use_signal(|| gemini_temp_init);
    let gemini_model = use_signal(|| gemini_model_init);

    let (openai_temp_init, openai_model_init, openai_base_url_init, openai_env_var_init) =
        extract_openai_fields(&settings);
    let openai_temperature = use_signal(|| openai_temp_init);
    let openai_model = use_signal(|| openai_model_init);
    let openai_base_url = use_signal(|| openai_base_url_init);
    let openai_env_var = use_signal(|| openai_env_var_init);

    let update_settings = {
        let llm_type = llm_type;
        let gemini_temperature = gemini_temperature;
        let gemini_model = gemini_model;
        let openai_temperature = openai_temperature;
        let openai_model = openai_model;
        let openai_base_url = openai_base_url;
        let openai_env_var = openai_env_var;
        let on_change = on_change;
        move || {
            let new_settings = create_llm_settings(
                llm_type(),
                gemini_temperature(),
                gemini_model(),
                openai_temperature(),
                openai_model(),
                openai_base_url(),
                openai_env_var(),
            );
            on_change.call(new_settings);
        }
    };

    rsx! {
        div { class: "space-y-4",
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "Тип LLM" }
                Select::<String> {
                    value: Some(Some(llm_value)),
                    on_value_change: move |v: Option<String>| {
                        if let Some(v) = v {
                            let next = match v.as_str() {
                                "gemini" => LlmType::Gemini,
                                "openai" => LlmType::OpenAi,
                                _ => LlmType::None,
                            };
                            llm_type.set(next);
                            update_settings();
                        }
                    },
                    placeholder: "Выберите...",
                    SelectTrigger { aria_label: "Тип LLM", width: "100%", SelectValue {} }
                    SelectList { aria_label: "Тип LLM",
                        SelectOption::<String> { index: 0usize, value: "none".to_string(),
                            "Отключено"
                            SelectItemIndicator {}
                        }
                        SelectOption::<String> { index: 1usize, value: "gemini".to_string(),
                            "Gemini"
                            SelectItemIndicator {}
                        }
                        SelectOption::<String> { index: 2usize, value: "openai".to_string(),
                            "OpenAI"
                            SelectItemIndicator {}
                        }
                    }
                }
            }

            match llm_type() {
                LlmType::None => rsx! {
                    Paragraph { class: Some("text-slate-500".to_string()), "LLM отключен" }
                },
                LlmType::Gemini => rsx! {
                    GeminiFields {
                        temperature: gemini_temperature,
                        model: gemini_model,
                        on_change: update_settings,
                    }
                },
                LlmType::OpenAi => rsx! {
                    OpenAiFields {
                        temperature: openai_temperature,
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
fn GeminiFields(
    temperature: Signal<String>,
    model: Signal<String>,
    on_change: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "Temperature" }
                Input {
                    value: temperature(),
                    oninput: {
                        let mut temperature = temperature;
                        let on_change = on_change;
                        move |e: FormEvent| {
                            temperature.set(e.value());
                            on_change.call(());
                        }
                    },
                }
            }
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
        }
    }
}

#[component]
fn OpenAiFields(
    temperature: Signal<String>,
    model: Signal<String>,
    base_url: Signal<String>,
    env_var: Signal<String>,
    on_change: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "Temperature" }
                Input {
                    value: temperature(),
                    oninput: {
                        let mut temperature = temperature;
                        let on_change = on_change;
                        move |e: FormEvent| {
                            temperature.set(e.value());
                            on_change.call(());
                        }
                    },
                }
            }
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
