use dioxus::prelude::*;

use crate::domain::RatingButtons;
use crate::ui::Card;

#[component]
pub fn LearnActive() -> Element {
    rsx! {
        div {
            class: "space-y-6",
            tabindex: "0",

            LearnProgress {
                current: 1,
                total: 10,
                progress: 10.0,
            }

            LearnCardDisplay {}

            LearnNavigation {}
        }
    }
}

#[component]
pub fn LearnProgress(current: usize, total: usize, progress: f64) -> Element {
    rsx! {
        Card { class: Some("space-y-3".to_string()),
            div { class: "flex items-center justify-between text-sm",
                span { class: "font-semibold text-slate-700", "Прогресс" }
                span { class: "text-slate-500", "{current} из {total}" }
            }
            div { class: "w-full h-3 bg-slate-100 rounded-full overflow-hidden",
                div {
                    class: "h-full bg-rainbow-vibrant rounded-full transition-all duration-500 ease-out",
                    style: "width: {progress}%",
                }
            }
        }
    }
}

#[component]
pub fn LearnCardDisplay() -> Element {
    rsx! {
        Card { class: Some("space-y-4".to_string()),
            // Здесь будет логика отображения карточки
            div { class: "text-center",
                "Карточка для изучения"
            }
        }
    }
}

#[component]
pub fn LearnNavigation() -> Element {
    rsx! {
        div { class: "flex gap-2",
            // Кнопки навигации
        }
    }
}
