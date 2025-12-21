use dioxus::prelude::*;

use crate::components::app_ui::{Card, Paragraph, SectionHeader};
use crate::components::button::{Button, ButtonVariant};
use crate::components::checkbox::Checkbox;
use crate::components::input::Input;
use crate::components::radio_group::{RadioGroup, RadioItem};
use crate::components::switch::{Switch, SwitchThumb};
use crate::{DEFAULT_USERNAME, ensure_user, to_error};
use dioxus_primitives::checkbox::CheckboxState;
use keikaku::application::use_cases::{
    export_anki_pack::ExportAnkiPackUseCase,
    export_jlpt_recommended::ExportJlptRecommendedUseCase,
    export_migii_pack::ExportMigiiPackUseCase,
    rebuild_database::{RebuildDatabaseOptions, RebuildDatabaseUseCase},
    sync_duolingo_words::SyncDuolingoWordsUseCase,
};
use keikaku::domain::value_objects::JapaneseLevel;
use keikaku::infrastructure::HttpDuolingoClient;
use keikaku::settings::ApplicationEnvironment;

#[component]
pub fn Import() -> Element {
    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            SectionHeader {
                title: "Импорт".to_string(),
                subtitle: Some(
                    "Сюда собраны все инструменты импорта/перестройки"
                        .to_string(),
                ),
                actions: None,
            }

            div { class: "space-y-6",
                ImportDuolingoTool {}
                ImportJlptTool {}
                ImportAnkiTool {}
                ImportMigiiTool {}
                ImportRebuildTool {}
            }
        }
    }
}

#[component]
fn ToolHeader(title: &'static str, subtitle: &'static str) -> Element {
    rsx! {
        div { class: "flex flex-col gap-1",
            Paragraph { class: Some("text-lg font-bold text-slate-800".to_string()), "{title}" }
            Paragraph { class: Some("text-sm text-slate-500".to_string()), "{subtitle}" }
        }
    }
}

#[component]
fn LogsCard(log: Signal<Vec<String>>) -> Element {
    rsx! {
        Card { class: Some("p-3 bg-slate-50 border border-slate-100 rounded-2xl space-y-2".to_string()),
            Paragraph { class: Some("text-sm font-semibold text-slate-700".to_string()), "Логи" }
            for entry in log().iter().rev() {
                Paragraph { class: Some("text-sm text-slate-600".to_string()), {entry.clone()} }
            }
            if log().is_empty() {
                Paragraph { class: Some("text-sm text-slate-500".to_string()), "История пустая." }
            }
        }
    }
}

#[component]
fn ImportDuolingoTool() -> Element {
    let mut question_only = use_signal(|| false);
    let log = use_signal(Vec::<String>::new);

    rsx! {
        Card { class: Some("space-y-4".to_string()),
            ToolHeader {
                title: "Duolingo синхронизация",
                subtitle: "Импорт изученных слов из Duolingo",
            }

            Paragraph { class: Some("text-sm text-slate-600".to_string()),
                "Синхронизация слов, нужен JWT в настройках CLI."
            }

            div { class: "flex items-center justify-between gap-4",
                span { class: "text-sm font-medium", "Только вопросы" }
                Switch {
                    aria_label: "Только вопросы",
                    checked: question_only(),
                    on_checked_change: move |v| question_only.set(v),
                    SwitchThumb {}
                }
            }

            Button {
                variant: ButtonVariant::Primary,
                class: "w-full",
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

            LogsCard { log }
        }
    }
}

#[component]
fn ImportJlptTool() -> Element {
    let levels = ["N5", "N4", "N3", "N2", "N1"];
    let selected = use_signal(|| vec!["N5".to_string(), "N4".to_string()]);
    let log = use_signal(Vec::<String>::new);

    rsx! {
        Card { class: Some("space-y-4".to_string()),
            ToolHeader {
                title: "JLPT импорт",
                subtitle: "Импорт слов для указанных уровней JLPT",
            }

            Paragraph { class: Some("text-sm text-slate-600".to_string()),
                "Отметьте уровни для генерации карточек."
            }

            div { class: "grid grid-cols-1 md:grid-cols-2 gap-3",
                for level in levels {
                    {
                        let level = level.to_string();
                        let checked = selected().contains(&level);
                        rsx! {
                            label { class: "flex items-center gap-3 cursor-pointer",
                                Checkbox {
                                    checked: if checked { CheckboxState::Checked } else { CheckboxState::Unchecked },
                                    on_checked_change: {
                                        let mut selected = selected;
                                        let level = level.clone();
                                        move |state: CheckboxState| {
                                            let v: bool = state.into();
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
                                }
                                span { class: "text-sm", "Уровень {level}" }
                            }
                        }
                    }
                }
            }

            Button {
                variant: ButtonVariant::Primary,
                class: "w-full",
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

            LogsCard { log }
        }
    }
}

#[component]
fn ImportAnkiTool() -> Element {
    let file_path = use_signal(|| "deck.apkg".to_string());
    let word_tag = use_signal(|| "Word".to_string());
    let translation_tag = use_signal(|| "Translation".to_string());
    let mut dry_run = use_signal(|| true);
    let log = use_signal(Vec::<String>::new);

    rsx! {
        Card { class: Some("space-y-4".to_string()),
            ToolHeader {
                title: "Anki импорт",
                subtitle: "Импорт слов из Anki файла",
            }

            div { class: "space-y-2",
                label { class: "text-sm font-medium", "ПУТЬ К ФАЙЛУ" }
                Input {
                    placeholder: "anki.apkg",
                    value: file_path(),
                    oninput: {
                        let mut file_path = file_path;
                        move |e: FormEvent| file_path.set(e.value())
                    },
                }
            }
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "ТЕГ ВОПРОСА" }
                Input {
                    placeholder: "Word",
                    value: word_tag(),
                    oninput: {
                        let mut word_tag = word_tag;
                        move |e: FormEvent| word_tag.set(e.value())
                    },
                }
            }
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "ТЕГ ПЕРЕВОДА" }
                Input {
                    placeholder: "Translation",
                    value: translation_tag(),
                    oninput: {
                        let mut translation_tag = translation_tag;
                        move |e: FormEvent| translation_tag.set(e.value())
                    },
                }
            }

            div { class: "flex items-center justify-between gap-4",
                span { class: "text-sm font-medium",
                    "Dry-run (показать без сохранения)"
                }
                Switch {
                    aria_label: "Dry-run",
                    checked: dry_run(),
                    on_checked_change: move |v| dry_run.set(v),
                    SwitchThumb {}
                }
            }

            Button {
                variant: ButtonVariant::Primary,
                class: "w-full",
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

            LogsCard { log }
        }
    }
}

#[component]
fn ImportMigiiTool() -> Element {
    let lessons = use_signal(|| "1,2,3".to_string());
    let mut question_only = use_signal(|| false);
    let log = use_signal(Vec::<String>::new);

    rsx! {
        Card { class: Some("space-y-4".to_string()),
            ToolHeader {
                title: "Migii импорт",
                subtitle: "Импорт слов из указанных уроков Migii",
            }

            div { class: "space-y-2",
                label { class: "text-sm font-medium", "УРОКИ" }
                Input {
                    placeholder: "Напр. 1,2,5",
                    value: lessons(),
                    oninput: {
                        let mut lessons = lessons;
                        move |e: FormEvent| lessons.set(e.value())
                    },
                }
            }

            div { class: "flex items-center justify-between gap-4",
                span { class: "text-sm font-medium", "Только вопросы" }
                Switch {
                    aria_label: "Только вопросы",
                    checked: question_only(),
                    on_checked_change: move |v| question_only.set(v),
                    SwitchThumb {}
                }
            }

            Button {
                variant: ButtonVariant::Primary,
                class: "w-full",
                onclick: {
                    let lessons_calc = lessons;
                    let question_only = question_only;
                    let mut log = log;
                    move |_| {
                        let lessons_str = lessons_calc();
                        let question_only = question_only();
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

            LogsCard { log }
        }
    }
}

#[component]
fn ImportRebuildTool() -> Element {
    let mut option = use_signal(|| "content".to_string());
    let log = use_signal(Vec::<String>::new);

    rsx! {
        Card { class: Some("space-y-4".to_string()),
            ToolHeader {
                title: "Пересборка базы",
                subtitle: "Пересборка базы данных",
            }

            RadioGroup {
                aria_label: "Rebuild options",
                value: Some(option()),
                on_value_change: move |v| option.set(v),
                class: "grid gap-2",
                RadioItem { index: 0usize, value: "content".to_string(),
                    span { class: "ml-2", "Только контент (ответы/примеры)" }
                }
                RadioItem { index: 1usize, value: "all".to_string(),
                    span { class: "ml-2", "Полная пересборка" }
                }
            }

            Button {
                variant: ButtonVariant::Primary,
                class: "w-full",
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

            LogsCard { log }
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
