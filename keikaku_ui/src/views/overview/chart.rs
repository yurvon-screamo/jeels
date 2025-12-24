use dioxus::prelude::*;

use crate::{
    components::app_ui::{Card, Chart, H4, Paragraph},
    views::overview::OverviewCharts,
};

#[component]
pub fn OverviewChartsComponent(charts: OverviewCharts) -> Element {
    rsx! {
        Card { class: Some("p-6 min-w-0".to_string()),
            div { class: "flex flex-col gap-1 mb-4",
                H4 { class: Some("text-slate-800".to_string()), "Графики прогресса" }
                Paragraph { class: Some("text-slate-500 text-sm".to_string()),
                    "Визуализация обучения по времени"
                }
            }
            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6 min-w-0",
                Chart {
                    title: "Стабильность знаний".to_string(),
                    data: charts.stability_data,
                    color: Some("#10b981".to_string()), // emerald-500
                    delay: Some("100".to_string()),
                }
                Chart {
                    title: "Сложность знаний".to_string(),
                    data: charts.difficulty_data,
                    color: Some("#ef4444".to_string()), // red-500
                    delay: Some("100".to_string()),
                }
                Chart {
                    title: "Изученных слов".to_string(),
                    data: charts.learned_words_data,
                    color: Some("#f59e0b".to_string()), // amber-500
                    delay: Some("300".to_string()),
                }
                Chart {
                    title: "В процессе изучения".to_string(),
                    data: charts.in_progress_words_data,
                    color: Some("#8b5cf6".to_string()), // violet-500
                    delay: Some("500".to_string()),
                }
                Chart {
                    title: "Низкая стабильность".to_string(),
                    data: charts.low_stability_words_data,
                    color: Some("#ec4899".to_string()), // pink-500
                    delay: Some("600".to_string()),
                }
                Chart {
                    title: "Высокая сложность".to_string(),
                    data: charts.high_difficulty_words_data,
                    color: Some("#f59e0b".to_string()), // amber-500
                    delay: Some("700".to_string()),
                }
                Chart {
                    title: "Новых слов".to_string(),
                    data: charts.new_words_data,
                    color: Some("#3b82f6".to_string()), // blue-500
                    delay: Some("200".to_string()),
                }
            }
        }
    }
}
