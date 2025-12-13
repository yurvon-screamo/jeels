use crate::ui::{Button, ButtonVariant, Card, Checkbox, Paragraph, SectionHeader};
use crate::{ensure_user, to_error, DEFAULT_USERNAME};
use dioxus::prelude::*;
use keikaku::application::use_cases::export_jlpt_recommended::ExportJlptRecommendedUseCase;
use keikaku::domain::value_objects::JapaneseLevel;
use keikaku::settings::ApplicationEnvironment;

#[component]
pub fn Jlpt() -> Element {
    let levels = ["N5", "N4", "N3", "N2", "N1"];
    let selected = use_signal(|| vec!["N5".to_string(), "N4".to_string()]);
    let log = use_signal(Vec::<String>::new);

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            SectionHeader {
                title: "JLPT импорт".to_string(),
                subtitle: Some("cli: jlpt_create --levels N5 N4 ...".to_string()),
                actions: None,
            }
            Card { class: Some("grid grid-cols-1 md:grid-cols-3 gap-4".to_string()),
                div { class: "space-y-3",
                    Paragraph { class: Some("text-sm text-slate-600".to_string()),
                        "Отметьте уровни для генерации карточек."
                    }
                    for level in levels {
                        Checkbox {
                            checked: use_signal({
                                let level = level.to_string();
                                let selected = selected;
                                move || selected().contains(&level)
                            }),
                            onchange: {
                                let level = level.to_string();
                                let mut selected = selected;
                                move |v| {
                                    let mut list = selected();
                                    if v {
                                        if !list.contains(&level) {
                                            list.push(level.clone());
                                        }
                                    } else {
                                        list.retain(|l| l != &level);
                                    }
                                    selected.set(list);
                                }
                            },
                            label: Some(format!("Уровень {level}")),
                        }
                    }
                    Button {
                        variant: ButtonVariant::Rainbow,
                        class: Some("w-full".to_string()),
                        onclick: move |_| {
                            if selected().is_empty() {
                                return;
                            }
                            let levels = selected();
                            let mut log = log;
                            spawn(async move {
                                match run_jlpt(levels).await {
                                    Ok(msg) => log.write().push(msg),
                                    Err(e) => log.write().push(format!("Ошибка: {e}")),
                                }
                            });
                        },
                        "Создать пачку"
                    }
                }
                Card { class: Some("p-4 bg-slate-50 border border-slate-100 rounded-2xl space-y-2".to_string()),
                    Paragraph { class: Some("text-sm font-semibold text-slate-700".to_string()),
                        "Логи"
                    }
                    if log().is_empty() {
                        Paragraph { class: Some("text-sm text-slate-500".to_string()),
                            "Операций пока нет."
                        }
                    } else {
                        for entry in log().iter().rev() {
                            Paragraph { class: Some("text-sm text-slate-600".to_string()),
                                {entry.clone()}
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn run_jlpt(levels: Vec<String>) -> Result<String, String> {
    let env = ApplicationEnvironment::get();
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let llm = env.get_llm_service(user_id).await.map_err(to_error)?;

    let parsed_levels = levels
        .into_iter()
        .map(|l| l.parse::<JapaneseLevel>())
        .collect::<Result<Vec<_>, _>>()?;

    let res = ExportJlptRecommendedUseCase::new(repo, &llm)
        .execute(user_id, parsed_levels)
        .await
        .map_err(to_error)?;

    Ok(format!(
        "JLPT: создано {}, пропущено {}",
        res.total_created_count,
        res.skipped_words.len()
    ))
}
