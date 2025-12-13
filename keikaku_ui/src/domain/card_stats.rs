use dioxus::prelude::*;

use crate::ui::Card;

#[component]
pub fn CardStats(total_count: usize, due_count: usize, filtered_count: usize) -> Element {
    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
            Card { class: Some("p-4".to_string()),
                div { class: "flex items-center justify-between",
                    div {
                        span { class: "text-xs font-semibold text-slate-500 uppercase",
                            "Всего карточек"
                        }
                        div { class: "text-2xl font-bold text-slate-800 mt-1", "{total_count}" }
                    }
                }
            }
            Card { class: Some("p-4".to_string()),
                div { class: "flex items-center justify-between",
                    div {
                        span { class: "text-xs font-semibold text-slate-500 uppercase",
                            "К повторению"
                        }
                        div { class: "text-2xl font-bold text-amber-600 mt-1", "{due_count}" }
                    }
                }
            }
            Card { class: Some("p-4".to_string()),
                div { class: "flex items-center justify-between",
                    div {
                        span { class: "text-xs font-semibold text-slate-500 uppercase",
                            "Показано"
                        }
                        div { class: "text-2xl font-bold text-slate-800 mt-1", "{filtered_count}" }
                    }
                }
            }
        }
    }
}
