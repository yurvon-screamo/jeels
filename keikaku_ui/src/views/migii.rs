use crate::ui::{Button, ButtonVariant, Card, Paragraph, SectionHeader, Switch, TextInput};
use crate::{ensure_user, to_error, DEFAULT_USERNAME};
use dioxus::prelude::*;
use keikaku::application::use_cases::export_migii_pack::ExportMigiiPackUseCase;
use keikaku::settings::ApplicationEnvironment;

#[component]
pub fn Migii() -> Element {
    let lessons = use_signal(|| "1,2,3".to_string());
    let mut question_only = use_signal(|| false);
    let log = use_signal(Vec::<String>::new);

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            SectionHeader {
                title: "Migii импорт".to_string(),
                subtitle: Some("cli: migii_create --lessons 1 2 3".to_string()),
                actions: None,
            }
            Card { class: Some("space-y-3".to_string()),
                TextInput {
                    label: Some("УРОКИ".to_string()),
                    placeholder: Some("Напр. 1,2,5".to_string()),
                    value: Some(lessons),
                    oninput: Some(
                        EventHandler::new({
                            let mut lessons = lessons;
                            move |e: Event<FormData>| lessons.set(e.value())
                        }),
                    ),
                    class: None,
                    r#type: None,
                }
                Switch {
                    checked: question_only(),
                    onchange: move |v| question_only.set(v),
                    label: Some("Только вопросы".to_string()),
                }
                Button {
                    variant: ButtonVariant::Rainbow,
                    class: Some("w-full".to_string()),
                    onclick: {
                        let lessons_calc = lessons;
                        let question_only = question_only();
                        let mut log = log;
                        move |_| {
                            let lessons_str = lessons_calc();
                            spawn(async move {
                                match run_migii(lessons_str, question_only).await {
                                    Ok(msg) => log.write().push(msg),
                                    Err(e) => log.write().push(format!("Ошибка: {e}")),
                                }
                            });
                        }
                    },
                    "Импортировать"
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

async fn run_migii(lessons: String, question_only: bool) -> Result<String, String> {
    let env = ApplicationEnvironment::get();
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let llm = env.get_llm_service(user_id).await.map_err(to_error)?;
    let migii_client = env.get_migii_client().await.map_err(to_error)?;

    let lesson_numbers: Vec<u32> = lessons
        .split(',')
        .filter_map(|s| s.trim().parse::<u32>().ok())
        .collect();
    if lesson_numbers.is_empty() {
        return Err("Укажите хотя бы один номер урока".to_string());
    }

    let res = ExportMigiiPackUseCase::new(repo, &llm, migii_client)
        .execute(user_id, lesson_numbers, question_only)
        .await
        .map_err(to_error)?;

    Ok(format!(
        "Migii: создано {}, пропущено {}",
        res.total_created_count,
        res.skipped_words.len()
    ))
}
