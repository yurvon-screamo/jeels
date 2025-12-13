use dioxus::prelude::*;

use super::metric_card::MetricCard;
use crate::ui::{Card, Pill, Section, StateTone};

#[component]
pub fn OverviewMetrics(
    primary_metrics: Vec<(String, String, String, StateTone)>,
    card_status_metrics: Vec<(String, String, String, StateTone)>,
) -> Element {
    rsx! {
        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-6",
            for (title , value , hint , tone) in primary_metrics {
                MetricCard {
                    label: title,
                    value,
                    hint: Some(hint),
                    tone: Some(tone),
                }
            }
        }

        Section { title: "Карточки".to_string(),
            Card { class: "p-6",
                div { class: "space-y-4",
                    for (title , value , hint , tone) in card_status_metrics {
                        div { class: "flex items-center justify-between",
                            div { class: "flex-1",
                                h4 { class: "text-sm font-medium text-slate-700", {title} }
                                p { class: "text-xs text-slate-500", {hint} }
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
