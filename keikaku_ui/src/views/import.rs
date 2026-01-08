use dioxus::prelude::*;

use crate::components::app_ui::{Card, LoadingState, Paragraph, Pill, SectionHeader, StateTone};
use crate::components::button::{Button, ButtonVariant};
use crate::components::checkbox::Checkbox;
use crate::components::input::Input;
use crate::components::switch::{Switch, SwitchThumb};
use crate::components::tabs::{TabContent, TabList, TabTrigger, Tabs};
use crate::{DEFAULT_USERNAME, ensure_user, to_error};
use dioxus_primitives::checkbox::CheckboxState;
use keikaku::application::use_cases::{
    import_anki_pack::ExportAnkiPackUseCase, import_jlpt_recommended::ExportJlptRecommendedUseCase,
    import_migii_pack::ExportMigiiPackUseCase, sync_duolingo_words::SyncDuolingoWordsUseCase,
};
use keikaku::domain::value_objects::JapaneseLevel;
use keikaku::infrastructure::HttpDuolingoClient;
use keikaku::settings::ApplicationEnvironment;

#[derive(Clone, PartialEq)]
pub enum OperationStatus {
    Idle,
    Loading,
    Success(String),
    Error(String),
}

impl OperationStatus {
    pub fn to_pill_text(&self) -> String {
        match self {
            OperationStatus::Idle => "Готово".to_string(),
            OperationStatus::Loading => "Выполняется...".to_string(),
            OperationStatus::Success(msg) => format!("Успешно: {}", msg),
            OperationStatus::Error(msg) => format!("Ошибка: {}", msg),
        }
    }

    pub fn to_tone(&self) -> StateTone {
        match self {
            OperationStatus::Idle => StateTone::Neutral,
            OperationStatus::Loading => StateTone::Info,
            OperationStatus::Success(_) => StateTone::Success,
            OperationStatus::Error(_) => StateTone::Warning,
        }
    }
}

#[component]
pub fn Import() -> Element {
    let mut active_tab = use_signal(|| "duolingo".to_string());

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

            Tabs {
                value: Some(active_tab()),
                on_value_change: move |value| active_tab.set(value),
                default_value: "duolingo".to_string(),

                TabList { class: "grid w-full grid-cols-5",
                    TabTrigger { value: "duolingo".to_string(), index: 0usize, "Duolingo" }
                    TabTrigger { value: "jlpt".to_string(), index: 1usize, "JLPT" }
                    TabTrigger { value: "anki".to_string(), index: 2usize, "Anki" }
                    TabTrigger { value: "migii".to_string(), index: 3usize, "Migii" }
                }

                TabContent { value: "duolingo".to_string(), index: 0usize, ImportDuolingoTool {} }
                TabContent { value: "jlpt".to_string(), index: 1usize, ImportJlptTool {} }
                TabContent { value: "anki".to_string(), index: 2usize, ImportAnkiTool {} }
                TabContent { value: "migii".to_string(), index: 3usize, ImportMigiiTool {} }
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
    let log = use_signal(Vec::<String>::new);
    let status = use_signal(|| OperationStatus::Idle);

    rsx! {
        Card { class: Some("space-y-4".to_string()),
            div { class: "flex items-center justify-between",
                ToolHeader {
                    title: "Duolingo синхронизация",
                    subtitle: "Импорт изученных слов из Duolingo",
                }
                Pill {
                    text: status().to_pill_text(),
                    tone: Some(status().to_tone()),
                }
            }

            Paragraph { class: Some("text-sm text-slate-600".to_string()),
                "Синхронизация слов, нужен JWT в настройках CLI."
            }


            if matches!(status(), OperationStatus::Loading) {
                LoadingState { message: Some("Синхронизация с Duolingo...".to_string()) }
            } else {
                Button {
                    variant: ButtonVariant::Primary,
                    class: "w-full",
                    disabled: matches!(status(), OperationStatus::Loading),
                    onclick: move |_| {
                        let mut log = log;
                        let mut status = status;
                        status.set(OperationStatus::Loading);
                        spawn(async move {
                            match run_duolingo().await {
                                Ok(msg) => {
                                    status.set(OperationStatus::Success(msg.clone()));
                                    log.write().push(msg);
                                }
                                Err(e) => {
                                    let error_msg = format!("Ошибка: {e}");
                                    status.set(OperationStatus::Error(e.to_string()));
                                    log.write().push(error_msg);
                                }
                            }
                        });
                    },
                    "Синхронизировать"
                }
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
    let status = use_signal(|| OperationStatus::Idle);

    rsx! {
        Card { class: Some("space-y-4".to_string()),
            div { class: "flex items-center justify-between",
                ToolHeader {
                    title: "JLPT импорт",
                    subtitle: "Импорт слов для указанных уровней JLPT",
                }
                Pill {
                    text: status().to_pill_text(),
                    tone: Some(status().to_tone()),
                }
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
                                    disabled: matches!(status(), OperationStatus::Loading),
                                }
                                span { class: "text-sm", "Уровень {level}" }
                            }
                        }
                    }
                }
            }

            if matches!(status(), OperationStatus::Loading) {
                LoadingState { message: Some("Создание пачки JLPT...".to_string()) }
            } else {
                Button {
                    variant: ButtonVariant::Primary,
                    class: "w-full",
                    disabled: matches!(status(), OperationStatus::Loading) || selected().is_empty(),
                    onclick: move |_| {
                        if selected().is_empty() {
                            return;
                        }
                        let levels = selected();
                        let mut log = log;
                        let mut status = status;
                        status.set(OperationStatus::Loading);
                        spawn(async move {
                            match run_jlpt(levels).await {
                                Ok(msg) => {
                                    status.set(OperationStatus::Success(msg.clone()));
                                    log.write().push(msg);
                                }
                                Err(e) => {
                                    let error_msg = format!("Ошибка: {e}");
                                    status.set(OperationStatus::Error(e.to_string()));
                                    log.write().push(error_msg);
                                }
                            }
                        });
                    },
                    "Создать пачку"
                }
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
    let status = use_signal(|| OperationStatus::Idle);

    rsx! {
        Card { class: Some("space-y-4".to_string()),
            div { class: "flex items-center justify-between",
                ToolHeader {
                    title: "Anki импорт",
                    subtitle: "Импорт слов из Anki файла",
                }
                Pill {
                    text: status().to_pill_text(),
                    tone: Some(status().to_tone()),
                }
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
                    disabled: matches!(status(), OperationStatus::Loading),
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
                    disabled: matches!(status(), OperationStatus::Loading),
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
                    disabled: matches!(status(), OperationStatus::Loading),
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
                    disabled: matches!(status(), OperationStatus::Loading),
                    SwitchThumb {}
                }
            }

            if matches!(status(), OperationStatus::Loading) {
                LoadingState { message: Some("Импорт из Anki файла...".to_string()) }
            } else {
                Button {
                    variant: ButtonVariant::Primary,
                    class: "w-full",
                    disabled: matches!(status(), OperationStatus::Loading),
                    onclick: {
                        let file_path = file_path;
                        let word_tag = word_tag;
                        let translation_tag = translation_tag;
                        let dry_run = dry_run;
                        let log = log;
                        let status = status;
                        move |_| {
                            let file = file_path();
                            let word = word_tag();
                            let translation = translation_tag();
                            let is_dry = dry_run();
                            let mut log = log;
                            let mut status = status;
                            status.set(OperationStatus::Loading);
                            spawn(async move {
                                let res = if is_dry {
                                    run_anki_dry(file.clone(), word.clone(), translation.clone()).await
                                } else {
                                    run_anki(file.clone(), word.clone(), translation.clone()).await
                                };
                                match res {
                                    Ok(msg) => {
                                        status.set(OperationStatus::Success(msg.clone()));
                                        log.write().push(msg);
                                    }
                                    Err(e) => {
                                        let error_msg = format!("Ошибка: {e}");
                                        status.set(OperationStatus::Error(e.to_string()));
                                        log.write().push(error_msg);
                                    }
                                }
                            });
                        }
                    },
                    "Запустить"
                }
            }

            LogsCard { log }
        }
    }
}

#[component]
fn ImportMigiiTool() -> Element {
    let lessons = use_signal(|| "1,2,3".to_string());
    let log = use_signal(Vec::<String>::new);
    let status = use_signal(|| OperationStatus::Idle);

    rsx! {
        Card { class: Some("space-y-4".to_string()),
            div { class: "flex items-center justify-between",
                ToolHeader {
                    title: "Migii импорт",
                    subtitle: "Импорт слов из указанных уроков Migii",
                }
                Pill {
                    text: status().to_pill_text(),
                    tone: Some(status().to_tone()),
                }
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
                    disabled: matches!(status(), OperationStatus::Loading),
                }
            }

            if matches!(status(), OperationStatus::Loading) {
                LoadingState { message: Some("Импорт из Migii...".to_string()) }
            } else {
                Button {
                    variant: ButtonVariant::Primary,
                    class: "w-full",
                    disabled: matches!(status(), OperationStatus::Loading),
                    onclick: {
                        let lessons_calc = lessons;
                        let log = log;
                        let status = status;
                        move |_| {
                            let lessons_str = lessons_calc();
                            let mut log = log;
                            let mut status = status;
                            status.set(OperationStatus::Loading);
                            spawn(async move {
                                match run_migii(lessons_str).await {
                                    Ok(msg) => {
                                        status.set(OperationStatus::Success(msg.clone()));
                                        log.write().push(msg);
                                    }
                                    Err(e) => {
                                        let error_msg = format!("Ошибка: {e}");
                                        status.set(OperationStatus::Error(e.to_string()));
                                        log.write().push(error_msg);
                                    }
                                }
                            });
                        }
                    },
                    "Импортировать"
                }
            }

            LogsCard { log }
        }
    }
}

async fn run_duolingo() -> Result<String, String> {
    let env = ApplicationEnvironment::get();
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let llm = env.get_llm_service(user_id).await.map_err(to_error)?;
    let client = HttpDuolingoClient::new();
    let use_case = SyncDuolingoWordsUseCase::new(repo, &llm, &client);
    let res = use_case.execute(user_id).await.map_err(to_error)?;
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

async fn run_migii(lessons: String) -> Result<String, String> {
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
        .execute(user_id, lesson_numbers)
        .await
        .map_err(to_error)?;

    Ok(format!(
        "Migii: создано {}, пропущено {}",
        res.total_created_count,
        res.skipped_words.len()
    ))
}
