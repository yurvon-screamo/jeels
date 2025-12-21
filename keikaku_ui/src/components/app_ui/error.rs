use dioxus::prelude::*;
use dioxus_heroicons::{Icon, solid};

use super::Card;

#[component]
pub fn ErrorCard(message: String, class: Option<String>) -> Element {
    let class_str = class.unwrap_or_else(|| "".to_string());

    rsx! {
        Card {
            class: Some(
                format!(
                    "relative overflow-hidden bg-gradient-to-br from-pink-50 via-purple-50 to-cyan-50 border border-pink-200/50 shadow-soft hover:shadow-soft-hover transition-all duration-500 animate-enter {}",
                    class_str,
                ),
            ),
            div { class: "absolute top-0 right-0 w-32 h-32 bg-accent-pink/10 rounded-full blur-2xl -translate-y-1/2 translate-x-1/2" }
            div { class: "absolute bottom-0 left-0 w-24 h-24 bg-accent-purple/10 rounded-full blur-xl translate-y-1/2 -translate-x-1/2" }

            div { class: "relative z-10 flex items-center space-x-4 p-2",
                div { class: "flex-shrink-0",
                    div { class: "w-12 h-12 rounded-2xl bg-gradient-to-br from-pink-100 to-purple-100 flex items-center justify-center shadow-md",
                        Icon {
                            icon: solid::Shape::ExclamationCircle,
                            size: 24,
                            class: Some("w-6 h-6 text-accent-pink".to_string()),
                        }
                    }
                }
                div { class: "flex-1",
                    div { class: "text-xs font-bold text-accent-purple uppercase tracking-widest mb-1",
                        "Ошибка"
                    }
                    p { class: "text-sm font-medium text-slate-700 leading-relaxed",
                        {message}
                    }
                }
            }
        }
    }
}
