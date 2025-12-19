use crate::ui::{Button, ButtonVariant, Card, Paragraph, SectionHeader, Switch};
use crate::{DEFAULT_USERNAME, ensure_user, to_error};
use dioxus::prelude::*;
use keikaku::application::use_cases::sync_duolingo_words::SyncDuolingoWordsUseCase;
use keikaku::infrastructure::HttpDuolingoClient;
use keikaku::settings::ApplicationEnvironment;

#[component]
pub fn Duolingo() -> Element {
    let mut question_only = use_signal(|| false);
    let log = use_signal(Vec::<String>::new);

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            SectionHeader {
                title: "Duolingo синхронизация".to_string(),
                subtitle: Some("cli: duolingo_sync --question-only".to_string()),
                actions: None,
            }
            Card { class: Some("space-y-3".to_string()),
                Paragraph { class: Some("text-sm text-slate-600".to_string()),
                    "Синхронизация слов, нужен JWT в настройках CLI."
                }
                Switch {
                    checked: question_only(),
                    onchange: move |v| question_only.set(v),
                    label: Some("Только вопросы".to_string()),
                }
                Button {
                    variant: ButtonVariant::Rainbow,
                    class: Some("w-full".to_string()),
                    onclick: move |_| {
                        let question_only = question_only();
                        let mut log = log;
                        spawn(async move {
                            match run_duolingo(question_only).await {
                                Ok(msg) => log.write().push(msg),
                                Err(e) => log.write().push(format!("Ошибка: {e}")),
                            }
                        });
                    },
                    "Синхронизировать"
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

async fn run_duolingo(question_only: bool) -> Result<String, String> {
    let env = ApplicationEnvironment::get();
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let llm = env.get_llm_service(user_id).await.map_err(to_error)?;
    let client = HttpDuolingoClient::new();
    let use_case = SyncDuolingoWordsUseCase::new(repo, &llm, &client);
    let res = use_case
        .execute(user_id, question_only)
        .await
        .map_err(to_error)?;
    Ok(format!(
        "Duolingo: создано {}, пропущено {}",
        res.total_created_count,
        res.skipped_words.len()
    ))
}
