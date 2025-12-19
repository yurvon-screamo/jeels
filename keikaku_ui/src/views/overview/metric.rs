use dioxus::prelude::*;

use crate::{
    ui::{Card, H2, H4, Heatmap, HeatmapDataPoint, Paragraph, Pill, Section, Size, StateTone, Tag},
    views::overview::overview::{MetricData, OverviewStats},
};

#[component]
pub fn OverviewMetrics(stats: OverviewStats, heatmap_data: Vec<HeatmapDataPoint>) -> Element {
    let card_status_metrics = build_card_status_metrics(&stats);

    rsx! {
        div { class: "grid grid-cols-1 lg:grid-cols-2 gap-8 mb-8 mt-4",
            // Левая колонка: метрики и тепловая карта
            div { class: "space-y-6",

                Card { class: "p-2",
                    H2 { class: Some("text-slate-800 flex items-center justify-between".to_string()),
                        "Всего карточек"
                        Tag { size: Some(Size::ExtraLarge),
                            {format!("{} шт.", stats.total_cards).to_string()}
                        }
                    }
                }

                Section { title: "Тепловая карта обучения".to_string(),
                    Card { class: "p-3",
                        Heatmap { data: heatmap_data }
                    }
                }
            }

            // Правая колонка: статус карточек
            Section { title: "Карточки".to_string(),
                Card { class: "p-6",
                    div { class: "space-y-1",
                        for (title , value , hint , tone) in card_status_metrics {
                            div { class: "flex items-center justify-between",
                                div { class: "flex-1",
                                    H4 { class: Some("text-slate-700".to_string()),
                                        {title}
                                    }
                                    Paragraph { class: Some("text-slate-500".to_string()),
                                        {hint}
                                    }
                                }
                                Pill {
                                    text: format!("{} шт.", value),
                                    tone: Some(tone),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn build_card_status_metrics(stats: &OverviewStats) -> Vec<MetricData> {
    vec![
        (
            "Для повторения".to_string(),
            stats.due_cards.to_string(),
            "Карточки, готовые к повторению".to_string(),
            if stats.due_cards > 0 {
                StateTone::Warning
            } else {
                StateTone::Success
            },
        ),
        (
            "Новые".to_string(),
            stats.new_cards.to_string(),
            "Карточки, которые еще не изучались".to_string(),
            StateTone::Info,
        ),
        (
            "Изучаемые".to_string(),
            stats.learning_cards.to_string(),
            "Карточки в процессе изучения".to_string(),
            StateTone::Neutral,
        ),
        (
            "Низкая стабильность".to_string(),
            stats.low_stability_cards.to_string(),
            "Карточки, требующие особого внимания".to_string(),
            StateTone::Warning,
        ),
        (
            "Изученные".to_string(),
            stats.known_cards.to_string(),
            "Карточки, которые хорошо запомнены".to_string(),
            StateTone::Success,
        ),
    ]
}
