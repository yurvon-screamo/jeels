use crate::ui::{Button, ButtonVariant, Card, Paragraph, Radio, SectionHeader};
use crate::{DEFAULT_USERNAME, ensure_user, to_error};
use dioxus::prelude::*;
use keikaku::application::use_cases::rebuild_database::{
    RebuildDatabaseOptions, RebuildDatabaseUseCase,
};
use keikaku::settings::ApplicationEnvironment;

#[component]
pub fn Rebuild() -> Element {
    let mut option = use_signal(|| "content".to_string());
    let mut is_content = use_signal(|| true);
    let mut is_all = use_signal(|| false);
    let log = use_signal(Vec::<String>::new);

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            SectionHeader {
                title: "Пересборка базы".to_string(),
                subtitle: Some("cli: rebuild_database --options <all|content>".to_string()),
                actions: None,
            }
            Card { class: Some("grid grid-cols-1 md:grid-cols-3 gap-4".to_string()),
                div { class: "space-y-3",
                    Radio {
                        checked: is_content(),
                        onchange: move |_| {
                            option.set("content".to_string());
                            is_content.set(true);
                            is_all.set(false);
                        },
                        name: "rebuild".to_string(),
                        label: Some("Только контент (ответы/примеры)".to_string()),
                    }
                    Radio {
                        checked: is_all(),
                        onchange: move |_| {
                            option.set("all".to_string());
                            is_content.set(false);
                            is_all.set(true);
                        },
                        name: "rebuild".to_string(),
                        label: Some("Полная пересборка".to_string()),
                    }
                    Button {
                        variant: ButtonVariant::Rainbow,
                        class: Some("w-full".to_string()),
                        onclick: move |_| {
                            let option = option();
                            let mut log = log;
                            spawn(async move {
                                match run_rebuild(&option).await {
                                    Ok(msg) => log.write().push(msg),
                                    Err(e) => log.write().push(format!("Ошибка: {e}")),
                                }
                            });
                        },
                        "Пересобрать"
                    }
                }
                Card { class: Some("p-3 bg-slate-50 border border-slate-100 rounded-2xl space-y-2".to_string()),
                    Paragraph { class: Some("text-sm font-semibold text-slate-700".to_string()),
                        "Логи"
                    }
                    for entry in log().iter().rev() {
                        Paragraph { class: Some("text-sm text-slate-600".to_string()), {entry.clone()} }
                    }
                    if log().is_empty() {
                        Paragraph { class: Some("text-sm text-slate-500".to_string()),
                            "История пустая."
                        }
                    }
                }
            }
        }
    }
}

async fn run_rebuild(option: &str) -> Result<String, String> {
    let env = ApplicationEnvironment::get();
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let llm = env.get_llm_service(user_id).await.map_err(to_error)?;

    let opt = match option.to_lowercase().as_str() {
        "all" => RebuildDatabaseOptions::All,
        _ => RebuildDatabaseOptions::Content,
    };

    let count = RebuildDatabaseUseCase::new(repo, &llm)
        .execute(user_id, opt)
        .await
        .map_err(to_error)?;

    Ok(format!(
        "Пересборка завершена, обработано карточек: {}",
        count
    ))
}
