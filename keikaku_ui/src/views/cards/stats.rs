use dioxus::prelude::*;

use crate::components::app_ui::{MetricTone, StatCard};

#[component]
pub fn CardsStats(total_count: usize, due_count: usize, filtered_count: usize) -> Element {
    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
            StatCard {
                title: Some("Всего карточек".to_string()),
                value: total_count.to_string(),
                label: "".to_string(), // В расширенном режиме label не используется, но требуется
                tone: Some(MetricTone::Neutral),
            }
            StatCard {
                title: Some("К повторению".to_string()),
                value: due_count.to_string(),
                label: "".to_string(),
                tone: Some(MetricTone::Warning),
            }
            StatCard {
                title: Some("Показано".to_string()),
                value: filtered_count.to_string(),
                label: "".to_string(),
                tone: Some(MetricTone::Neutral),
            }
        }
    }
}
