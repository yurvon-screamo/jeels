use dioxus::prelude::*;
use dioxus_heroicons::{Icon, solid};

use super::Paragraph;
use crate::components::button::{Button, ButtonVariant};

/// EmptyState - компонент для отображения пустых состояний с призывом к действию.
#[component]
pub fn EmptyState(
    icon: Option<Element>,
    title: String,
    description: Option<String>,
    action_text: Option<String>,
    on_action: Option<EventHandler<()>>,
    additional_content: Option<Element>,
) -> Element {
    rsx! {
        div { class: "text-center space-y-6 py-12",
            if let Some(icon_el) = icon {
                div { class: "flex justify-center mb-6", {icon_el} }
            }

            Paragraph { class: Some("text-xl font-bold text-slate-700".to_string()), "{title}" }

            if let Some(desc) = description {
                Paragraph { class: Some("text-base text-slate-500 max-w-md mx-auto".to_string()),
                    "{desc}"
                }
            }

            if let Some(action) = action_text {
                if let Some(handler) = on_action {
                    div { class: "space-y-3",
                        Button {
                            variant: ButtonVariant::Primary,
                            class: "px-8 py-3 text-lg font-semibold",
                            onclick: move |_| handler.call(()),
                            "{action}"
                        }
                        if let Some(extra) = additional_content {
                            {extra}
                        } else {
                            div { class: "flex items-center justify-center gap-2 text-xs text-slate-400",
                                Icon {
                                    icon: solid::Shape::LightBulb,
                                    size: 16,
                                    class: Some("text-slate-400".to_string()),
                                }
                                span {
                                    "Начните с 5-10 карточек для лучшего запоминания"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
