use crate::ui::{Button, ButtonVariant, Card, Paragraph, SectionHeader, Switch, TextInput};
use crate::{ensure_user, to_error, DEFAULT_USERNAME};
use dioxus::prelude::*;
use keikaku::application::use_cases::export_anki_pack::ExportAnkiPackUseCase;
use keikaku::settings::ApplicationEnvironment;

#[component]
pub fn Anki() -> Element {
    let file_path = use_signal(|| "deck.apkg".to_string());
    let word_tag = use_signal(|| "Word".to_string());
    let translation_tag = use_signal(|| "Translation".to_string());
    let mut dry_run = use_signal(|| true);
    let log = use_signal(Vec::<String>::new);

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            SectionHeader {
                title: "Anki импорт".to_string(),
                subtitle: Some(
                    "cli: anki_create --file path --word-tag Word --translation-tag Translation"
                        .to_string(),
                ),
                actions: None,
            }
            Card { class: Some("space-y-3".to_string()),
                TextInput {
                    label: Some("ПУТЬ К ФАЙЛУ".to_string()),
                    placeholder: Some("anki.apkg".to_string()),
                    value: Some(file_path),
                    oninput: Some(
                        EventHandler::new({
                            let mut file_path = file_path;
                            move |e: Event<FormData>| file_path.set(e.value())
                        }),
                    ),
                    class: None,
                    r#type: None,
                }
                TextInput {
                    label: Some("ТЕГ ВОПРОСА".to_string()),
                    placeholder: Some("Word".to_string()),
                    value: Some(word_tag),
                    oninput: Some(
                        EventHandler::new({
                            let mut word_tag = word_tag;
                            move |e: Event<FormData>| word_tag.set(e.value())
                        }),
                    ),
                    class: None,
                    r#type: None,
                }
                TextInput {
                    label: Some("ТЕГ ПЕРЕВОДА".to_string()),
                    placeholder: Some("Translation".to_string()),
                    value: Some(translation_tag),
                    oninput: Some(
                        EventHandler::new({
                            let mut translation_tag = translation_tag;
                            move |e: Event<FormData>| translation_tag.set(e.value())
                        }),
                    ),
                    class: None,
                    r#type: None,
                }
                Switch {
                    checked: dry_run(),
                    onchange: move |v| dry_run.set(v),
                    label: Some("Dry-run (показать без сохранения)".to_string()),
                }
                Button {
                    variant: ButtonVariant::Rainbow,
                    class: Some("w-full".to_string()),
                    onclick: {
                        let file_path = file_path;
                        let word_tag = word_tag;
                        let translation_tag = translation_tag;
                        let dry_run = dry_run;
                        let log = log;
                        move |_| {
                            let file = file_path();
                            let word = word_tag();
                            let translation = translation_tag();
                            let is_dry = dry_run();
                            let mut log = log;
                            spawn(async move {
                                let res = if is_dry {
                                    run_anki_dry(file.clone(), word.clone(), translation.clone()).await
                                } else {
                                    run_anki(file.clone(), word.clone(), translation.clone()).await
                                };
                                match res {
                                    Ok(msg) => log.write().push(msg),
                                    Err(e) => log.write().push(format!("Ошибка: {e}")),
                                }
                            });
                        }
                    },
                    "Запустить"
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

async fn run_anki_dry(
    file_path: String,
    word_tag: String,
    translation_tag: String,
) -> Result<String, String> {
    let env = ApplicationEnvironment::get();
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    let repo = env.get_repository().await.map_err(to_error)?;
    let llm = env.get_llm_service(user_id).await.map_err(to_error)?;
    let use_case = ExportAnkiPackUseCase::new(repo, &llm);
    let cards = use_case
        .extract_cards(&file_path, &word_tag, Some(translation_tag.as_str()))
        .await
        .map_err(to_error)?;
    Ok(format!("Dry-run: найдено {} карточек", cards.len()))
}

async fn run_anki(
    file_path: String,
    word_tag: String,
    translation_tag: String,
) -> Result<String, String> {
    let env = ApplicationEnvironment::get();
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let llm = env.get_llm_service(user_id).await.map_err(to_error)?;
    let use_case = ExportAnkiPackUseCase::new(repo, &llm);
    let result = use_case
        .execute(user_id, file_path, word_tag, Some(translation_tag))
        .await
        .map_err(to_error)?;
    Ok(format!(
        "Импорт Anki: создано {}, пропущено {}",
        result.total_created_count,
        result.skipped_words.len()
    ))
}
