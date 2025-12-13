use dioxus::prelude::*;

use crate::ui::Card;

#[component]
pub fn LearnProgress(current: usize, total: usize, progress: f64) -> Element {
    let percentage = (progress as usize).min(100);

    rsx! {
        Card { class: Some("py-3 px-4".to_string()),
            div { class: "flex items-center justify-between text-sm",
                div { class: "flex items-center gap-2",
                    span { class: "font-semibold text-slate-700", "Прогресс" }
                    span { class: "text-xs bg-slate-100 text-slate-600 px-2 py-0.5 rounded-full",
                        "{percentage}%"
                    }
                }
                span { class: "text-slate-500 font-medium", "{current} из {total}" }
            }
            div { class: "w-full h-2 bg-slate-100 rounded-full overflow-hidden mt-2",
                div {
                    class: format!(
                        "h-full rounded-full transition-all duration-700 ease-out relative {}",
                        if progress >= 100.0 {
                            "bg-gradient-to-r from-green-400 to-green-500"
                        } else {
                            "bg-gradient-to-r from-blue-400 via-purple-500 to-pink-500"
                        },
                    ),
                    style: "width: {progress}%",

                    if progress > 0.0 && progress < 100.0 {
                        div { class: "absolute inset-0 bg-gradient-to-r from-transparent via-white to-transparent animate-pulse opacity-30" }
                    }
                }
            }
        }
    }
}
