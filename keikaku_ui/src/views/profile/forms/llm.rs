use dioxus::prelude::*;
use keikaku::domain::LlmSettings;

use crate::ui::{Paragraph, Select, TextInput};

#[derive(Debug, Clone, PartialEq)]
pub enum LlmType {
    None,
    Gemini,
    OpenAi,
    Candle,
}

impl std::fmt::Display for LlmType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmType::None => write!(f, "Отключено"),
            LlmType::Gemini => write!(f, "Gemini"),
            LlmType::OpenAi => write!(f, "OpenAI"),
            LlmType::Candle => write!(f, "Candle"),
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

fn extract_candle_fields(
    settings: &LlmSettings,
) -> (
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
) {
    if let LlmSettings::Candle {
        max_sample_len,
        temperature,
        seed,
        model_repo,
        model_filename,
        model_revision,
        tokenizer_repo,
        tokenizer_filename,
    } = settings
    {
        (
            max_sample_len.to_string(),
            temperature.to_string(),
            seed.to_string(),
            model_repo.clone(),
            model_filename.clone(),
            model_revision.clone(),
            tokenizer_repo.clone(),
            tokenizer_filename.clone(),
        )
    } else {
        (
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        )
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
    candle_max_len: String,
    candle_temp: String,
    candle_seed: String,
    candle_model_repo: String,
    candle_model_filename: String,
    candle_model_revision: String,
    candle_tokenizer_repo: String,
    candle_tokenizer_filename: String,
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
        LlmType::Candle => LlmSettings::Candle {
            max_sample_len: candle_max_len.parse().unwrap_or(0),
            temperature: candle_temp.parse().unwrap_or(0.0),
            seed: candle_seed.parse().unwrap_or(0),
            model_repo: candle_model_repo,
            model_filename: candle_model_filename,
            model_revision: candle_model_revision,
            tokenizer_repo: candle_tokenizer_repo,
            tokenizer_filename: candle_tokenizer_filename,
        },
    }
}

#[component]
pub fn LlmSettingsForm(settings: LlmSettings, on_change: EventHandler<LlmSettings>) -> Element {
    let mut llm_type = use_signal(|| match &settings {
        LlmSettings::None => LlmType::None,
        LlmSettings::Gemini { .. } => LlmType::Gemini,
        LlmSettings::OpenAi { .. } => LlmType::OpenAi,
        LlmSettings::Candle { .. } => LlmType::Candle,
    });

    let (gemini_temp_init, gemini_model_init) = extract_gemini_fields(&settings);
    let gemini_temperature = use_signal(|| gemini_temp_init);
    let gemini_model = use_signal(|| gemini_model_init);

    let (openai_temp_init, openai_model_init, openai_base_url_init, openai_env_var_init) =
        extract_openai_fields(&settings);
    let openai_temperature = use_signal(|| openai_temp_init);
    let openai_model = use_signal(|| openai_model_init);
    let openai_base_url = use_signal(|| openai_base_url_init);
    let openai_env_var = use_signal(|| openai_env_var_init);

    let (
        candle_max_len_init,
        candle_temp_init,
        candle_seed_init,
        candle_model_repo_init,
        candle_model_filename_init,
        candle_model_revision_init,
        candle_tokenizer_repo_init,
        candle_tokenizer_filename_init,
    ) = extract_candle_fields(&settings);
    let candle_max_sample_len = use_signal(|| candle_max_len_init);
    let candle_temperature = use_signal(|| candle_temp_init);
    let candle_seed = use_signal(|| candle_seed_init);
    let candle_model_repo = use_signal(|| candle_model_repo_init);
    let candle_model_filename = use_signal(|| candle_model_filename_init);
    let candle_model_revision = use_signal(|| candle_model_revision_init);
    let candle_tokenizer_repo = use_signal(|| candle_tokenizer_repo_init);
    let candle_tokenizer_filename = use_signal(|| candle_tokenizer_filename_init);

    let update_settings = {
        let llm_type = llm_type;
        let gemini_temperature = gemini_temperature;
        let gemini_model = gemini_model;
        let openai_temperature = openai_temperature;
        let openai_model = openai_model;
        let openai_base_url = openai_base_url;
        let openai_env_var = openai_env_var;
        let candle_max_sample_len = candle_max_sample_len;
        let candle_temperature = candle_temperature;
        let candle_seed = candle_seed;
        let candle_model_repo = candle_model_repo;
        let candle_model_filename = candle_model_filename;
        let candle_model_revision = candle_model_revision;
        let candle_tokenizer_repo = candle_tokenizer_repo;
        let candle_tokenizer_filename = candle_tokenizer_filename;
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
                candle_max_sample_len(),
                candle_temperature(),
                candle_seed(),
                candle_model_repo(),
                candle_model_filename(),
                candle_model_revision(),
                candle_tokenizer_repo(),
                candle_tokenizer_filename(),
            );
            on_change.call(new_settings);
        }
    };

    rsx! {
        div { class: "space-y-4",
            Select {
                label: Some("Тип LLM".to_string()),
                options: vec![LlmType::None, LlmType::Gemini, LlmType::OpenAi, LlmType::Candle],
                selected: use_signal(|| Some(llm_type())),
                onselect: move |new_type: LlmType| {
                    llm_type.set(new_type.clone());
                    update_settings();
                },
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
                LlmType::Candle => rsx! {
                    CandleFields {
                        max_sample_len: candle_max_sample_len,
                        temperature: candle_temperature,
                        seed: candle_seed,
                        model_repo: candle_model_repo,
                        model_filename: candle_model_filename,
                        model_revision: candle_model_revision,
                        tokenizer_repo: candle_tokenizer_repo,
                        tokenizer_filename: candle_tokenizer_filename,
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
            TextInput {
                label: Some("Temperature".to_string()),
                value: Some(temperature),
                placeholder: None,
                oninput: Some(
                    EventHandler::new({
                        let mut temperature = temperature;
                        let on_change = on_change;
                        move |e: Event<FormData>| {
                            temperature.set(e.value());
                            on_change.call(());
                        }
                    }),
                ),
                class: None,
                r#type: None,
            }
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
            TextInput {
                label: Some("Temperature".to_string()),
                value: Some(temperature),
                placeholder: None,
                oninput: Some(
                    EventHandler::new({
                        let mut temperature = temperature;
                        let on_change = on_change;
                        move |e: Event<FormData>| {
                            temperature.set(e.value());
                            on_change.call(());
                        }
                    }),
                ),
                class: None,
                r#type: None,
            }
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

#[component]
fn CandleFields(
    max_sample_len: Signal<String>,
    temperature: Signal<String>,
    seed: Signal<String>,
    model_repo: Signal<String>,
    model_filename: Signal<String>,
    model_revision: Signal<String>,
    tokenizer_repo: Signal<String>,
    tokenizer_filename: Signal<String>,
    on_change: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
            TextInput {
                label: Some("Max Sample Length".to_string()),
                value: Some(max_sample_len),
                placeholder: None,
                oninput: Some(
                    EventHandler::new({
                        let mut max_sample_len = max_sample_len;
                        let on_change = on_change;
                        move |e: Event<FormData>| {
                            max_sample_len.set(e.value());
                            on_change.call(());
                        }
                    }),
                ),
                class: None,
                r#type: None,
            }
            TextInput {
                label: Some("Temperature".to_string()),
                value: Some(temperature),
                placeholder: None,
                oninput: Some(
                    EventHandler::new({
                        let mut temperature = temperature;
                        let on_change = on_change;
                        move |e: Event<FormData>| {
                            temperature.set(e.value());
                            on_change.call(());
                        }
                    }),
                ),
                class: None,
                r#type: None,
            }
            TextInput {
                label: Some("Seed".to_string()),
                value: Some(seed),
                placeholder: None,
                oninput: Some(
                    EventHandler::new({
                        let mut seed = seed;
                        let on_change = on_change;
                        move |e: Event<FormData>| {
                            seed.set(e.value());
                            on_change.call(());
                        }
                    }),
                ),
                class: None,
                r#type: None,
            }
            TextInput {
                label: Some("Model Repo".to_string()),
                value: Some(model_repo),
                placeholder: None,
                oninput: Some(
                    EventHandler::new({
                        let mut model_repo = model_repo;
                        let on_change = on_change;
                        move |e: Event<FormData>| {
                            model_repo.set(e.value());
                            on_change.call(());
                        }
                    }),
                ),
                class: None,
                r#type: None,
            }
            TextInput {
                label: Some("Model Filename".to_string()),
                value: Some(model_filename),
                placeholder: None,
                oninput: Some(
                    EventHandler::new({
                        let mut model_filename = model_filename;
                        let on_change = on_change;
                        move |e: Event<FormData>| {
                            model_filename.set(e.value());
                            on_change.call(());
                        }
                    }),
                ),
                class: None,
                r#type: None,
            }
            TextInput {
                label: Some("Model Revision".to_string()),
                value: Some(model_revision),
                placeholder: None,
                oninput: Some(
                    EventHandler::new({
                        let mut model_revision = model_revision;
                        let on_change = on_change;
                        move |e: Event<FormData>| {
                            model_revision.set(e.value());
                            on_change.call(());
                        }
                    }),
                ),
                class: None,
                r#type: None,
            }
            TextInput {
                label: Some("Tokenizer Repo".to_string()),
                value: Some(tokenizer_repo),
                placeholder: None,
                oninput: Some(
                    EventHandler::new({
                        let mut tokenizer_repo = tokenizer_repo;
                        let on_change = on_change;
                        move |e: Event<FormData>| {
                            tokenizer_repo.set(e.value());
                            on_change.call(());
                        }
                    }),
                ),
                class: None,
                r#type: None,
            }
            TextInput {
                label: Some("Tokenizer Filename".to_string()),
                value: Some(tokenizer_filename),
                placeholder: None,
                oninput: Some(
                    EventHandler::new({
                        let mut tokenizer_filename = tokenizer_filename;
                        let on_change = on_change;
                        move |e: Event<FormData>| {
                            tokenizer_filename.set(e.value());
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
