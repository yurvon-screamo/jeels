use dioxus::prelude::*;

use crate::ui::{Button, ButtonVariant, Card, Paragraph, H2};

#[component]
pub fn LearnCompleted(total_cards: usize, on_restart: EventHandler<()>) -> Element {
    rsx! {
        Card { class: Some("space-y-8 text-center py-12 max-w-md mx-auto".to_string()),
            div { class: "text-7xl mb-6 animate-bounce", "🎉" }

            H2 { class: Some("text-3xl font-bold text-slate-800 mb-2".to_string()),
                "Поздравляем!"
            }

            Paragraph { class: Some("text-lg text-slate-600 mb-6".to_string()),
                "Вы успешно завершили сессию обучения"
            }

            div { class: "bg-slate-50 rounded-lg p-6 mb-8",
                div { class: "grid grid-cols-1 gap-4",
                    div { class: "text-center",
                        div { class: "text-3xl font-bold text-blue-600", "{total_cards}" }
                        div { class: "text-sm text-slate-500", "Карточек пройдено" }
                    }
                }
            }

            div { class: "space-y-3",
                Button {
                    variant: ButtonVariant::Rainbow,
                    class: Some("w-full".to_string()),
                    onclick: move |_| on_restart.call(()),
                    "Начать новую сессию"
                }

                Paragraph { class: Some("text-xs text-slate-400".to_string()),
                    "💡 Совет: Повторяйте материал регулярно для лучшего запоминания"
                }
            }
        }
    }
}
