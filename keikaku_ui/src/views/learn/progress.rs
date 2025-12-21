use dioxus::prelude::*;

use crate::components::app_ui::Card;
use crate::components::progress::{Progress, ProgressIndicator};

#[component]
pub fn LearnProgress(current: usize, total: usize, progress: f64) -> Element {
    rsx! {
        Card { class: Some("py-3 px-4".to_string()),
            div { class: "flex items-center justify-between text-sm",
                div { class: "flex items-center gap-2",
                    span { class: "font-semibold text-slate-700", "Прогресс" }
                    span { class: "text-xs bg-slate-100 text-slate-600 px-2 py-0.5 rounded-full",
                        "{(progress as usize).min(100)}%"
                    }
                }
                span { class: "text-slate-500 font-medium", "{current} из {total}" }
            }
            Progress {
                aria_label: "Прогресс обучения",
                value: progress,
                class: "mt-2",
                ProgressIndicator {}
            }
        }
    }
}
