use dioxus::prelude::*;

use crate::ui::ProgressBar;

#[component]
pub fn LearnProgress(current: usize, total: usize, progress: f64) -> Element {
    rsx! {
        ProgressBar {
            title: Some("Прогресс".to_string()),
            subtitle: Some(format!("{} из {}", current, total)),
            progress,
        }
    }
}
