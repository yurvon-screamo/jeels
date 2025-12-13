use dioxus::prelude::*;

use crate::{
    ui::{Chart, Section},
    views::overview::OverviewCharts,
};

#[component]
pub fn OverviewChartsComponent(charts: OverviewCharts) -> Element {
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
