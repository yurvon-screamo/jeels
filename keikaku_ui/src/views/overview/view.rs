use dioxus::prelude::*;

use crate::ui::{Card, Chart, ChartDataPoint, MetricCard, MetricTone, Paragraph, Section, H1};

#[component]
pub fn Overview() -> Element {
    // Заглушка для данных
    let username = "Пользователь".to_string();
    let metrics = vec![
        (
            "Карточек".to_string(),
            "42".to_string(),
            "Всего карточек".to_string(),
            MetricTone::Info,
        ),
        (
            "К повторению".to_string(),
            "7".to_string(),
            "Сегодня".to_string(),
            MetricTone::Warning,
        ),
        (
            "Сессий".to_string(),
            "15".to_string(),
            "За неделю".to_string(),
            MetricTone::Success,
        ),
    ];

    let charts = OverviewCharts {
        stability_data: vec![
            ChartDataPoint {
                label: "Пн".to_string(),
                value: 85.0,
            },
            ChartDataPoint {
                label: "Вт".to_string(),
                value: 90.0,
            },
            ChartDataPoint {
                label: "Ср".to_string(),
                value: 88.0,
            },
            ChartDataPoint {
                label: "Чт".to_string(),
                value: 92.0,
            },
            ChartDataPoint {
                label: "Пт".to_string(),
                value: 95.0,
            },
            ChartDataPoint {
                label: "Сб".to_string(),
                value: 93.0,
            },
            ChartDataPoint {
                label: "Вс".to_string(),
                value: 96.0,
            },
        ],
        words_progress_data: vec![
            ChartDataPoint {
                label: "Неделя 1".to_string(),
                value: 10.0,
            },
            ChartDataPoint {
                label: "Неделя 2".to_string(),
                value: 25.0,
            },
            ChartDataPoint {
                label: "Неделя 3".to_string(),
                value: 45.0,
            },
            ChartDataPoint {
                label: "Неделя 4".to_string(),
                value: 70.0,
            },
        ],
        lessons_data: vec![
            ChartDataPoint {
                label: "Пн".to_string(),
                value: 2.0,
            },
            ChartDataPoint {
                label: "Вт".to_string(),
                value: 1.0,
            },
            ChartDataPoint {
                label: "Ср".to_string(),
                value: 3.0,
            },
            ChartDataPoint {
                label: "Чт".to_string(),
                value: 2.0,
            },
            ChartDataPoint {
                label: "Пт".to_string(),
                value: 1.0,
            },
            ChartDataPoint {
                label: "Сб".to_string(),
                value: 0.0,
            },
            ChartDataPoint {
                label: "Вс".to_string(),
                value: 1.0,
            },
        ],
    };

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8",
            OverviewHeader { username }
            OverviewMetrics { metrics }
            OverviewChartsComponent { charts }
        }
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

#[derive(Clone, PartialEq)]
pub struct OverviewCharts {
    pub stability_data: Vec<ChartDataPoint>,
    pub words_progress_data: Vec<ChartDataPoint>,
    pub lessons_data: Vec<ChartDataPoint>,
}
