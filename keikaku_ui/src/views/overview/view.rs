use dioxus::prelude::*;
use keikaku::application::use_cases::{
    get_user_info::{GetUserInfoUseCase, UserProfile},
    list_cards::ListCardsUseCase,
};
use keikaku::domain::VocabularyCard;

use crate::ui::{Card, Chart, ChartDataPoint, MetricCard, MetricTone, Paragraph, Section, H1};
use crate::{ensure_user, init_env, to_error, DEFAULT_USERNAME};

use super::{build_charts, build_metrics, calculate_stats, OverviewCharts};

async fn fetch_profile() -> Result<UserProfile, String> {
    let env = init_env().await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    GetUserInfoUseCase::new(repo)
        .execute(user_id)
        .await
        .map_err(to_error)
}

async fn fetch_cards() -> Result<Vec<VocabularyCard>, String> {
    let env = init_env().await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    ListCardsUseCase::new(repo)
        .execute(user_id)
        .await
        .map_err(to_error)
}

#[component]
pub fn Overview() -> Element {
    let profile_resource = use_resource(fetch_profile);
    let cards_resource = use_resource(fetch_cards);

    // Read resources once and store results
    let profile_read = profile_resource.read();
    let cards_read = cards_resource.read();

    match (profile_read.as_ref(), cards_read.as_ref()) {
        (Some(Err(profile_err)), _) => rsx! {
            div { class: "bg-bg min-h-screen text-text-main px-6 py-8",
                "Ошибка загрузки профиля: {profile_err}"
            }
        },
        (_, Some(Err(cards_err))) => rsx! {
            div { class: "bg-bg min-h-screen text-text-main px-6 py-8",
                "Ошибка загрузки карточек: {cards_err}"
            }
        },
        (Some(Ok(profile)), Some(Ok(cards))) => {
            let stats = calculate_stats(Some(profile), Some(cards));
            let metrics = build_metrics(&stats);
            let charts = build_charts(&profile.lesson_history[..]);

            rsx! {
                div { class: "bg-bg min-h-screen text-text-main px-6 py-8",
                    OverviewHeader { username: stats.username }
                    OverviewMetrics { metrics }
                    OverviewChartsComponent { charts }
                }
            }
        }
        _ => rsx! {
            div { class: "bg-bg min-h-screen text-text-main px-6 py-8", "Загрузка..." }
        },
    }
}

#[component]
fn OverviewHeader(username: String) -> Element {
    rsx! {
        header { class: "flex flex-col md:flex-row md:items-center md:justify-between gap-3 mb-6",
            div {
                H1 { class: Some("text-3xl font-extrabold text-slate-800".to_string()),
                    "Keikaku — панель"
                }
                Paragraph { class: Some("text-slate-500".to_string()),
                    "Быстрый обзор прогресса и действий CLI."
                }
            }
            Card { class: Some("px-4 py-3 rounded-2xl bg-white/80 shadow-soft".to_string()),
                div { class: "text-xs text-slate-500 uppercase tracking-widest", "Аккаунт" }
                div { class: "text-sm font-semibold text-slate-700", {username} }
            }
        }
    }
}

#[component]
fn OverviewMetrics(metrics: Vec<(String, String, String, MetricTone)>) -> Element {
    rsx! {
        Section {
            title: "Метрики".to_string(),
            description: Some(
                "Статистика из CLI: карточки, повторы, streak."
                    .to_string(),
            ),
            div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4",
                for (title , value , hint , tone) in metrics {
                    MetricCard {
                        label: title,
                        value,
                        hint: Some(hint),
                        tone: Some(tone),
                    }
                }
            }
        }
    }
}

#[component]
fn OverviewChartsComponent(charts: OverviewCharts) -> Element {
    rsx! {
        Section {
            title: "Графики прогресса".to_string(),
            description: Some("Визуализация обучения по времени".to_string()),
            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6 min-w-0",
                Chart {
                    title: "Стабильность знаний".to_string(),
                    data: charts.stability_data,
                    color: Some("#10b981".to_string()), // emerald-500
                    delay: Some("100".to_string()),
                }
                Chart {
                    title: "Общее количество слов".to_string(),
                    data: charts.words_progress_data,
                    color: Some("#3b82f6".to_string()), // blue-500
                    delay: Some("200".to_string()),
                }
                Chart {
                    title: "Завершенные уроки".to_string(),
                    data: charts.lessons_data,
                    color: Some("#f59e0b".to_string()), // amber-500
                    delay: Some("300".to_string()),
                }
            }
        }
    }
}
