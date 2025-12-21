use dioxus::prelude::*;

use super::{Card, Paragraph};

#[component]
pub fn Section(title: String, description: Option<String>, children: Element) -> Element {
    rsx! {
        Card {
            div { class: "flex flex-col gap-3 ma-4",
                h2 { class: "text-xl font-bold text-slate-800", {title.clone()} }
                if let Some(desc) = description {
                    Paragraph { class: Some("text-sm text-slate-500".to_string()), "{desc}" }
                }
            }
            {children}
        }
    }
}

#[component]
pub fn SectionHeader(title: String, subtitle: Option<String>, actions: Option<Element>) -> Element {
    rsx! {
        div { class: "flex flex-col md:flex-row md:items-center md:justify-between gap-3",
            div {
                h2 { class: "text-xl font-bold text-slate-800", {title} }
                if let Some(text) = subtitle {
                    Paragraph { class: Some("text-sm text-slate-500".to_string()), "{text}" }
                }
            }
            if let Some(action_slot) = actions {
                div { class: "flex items-center gap-2", {action_slot} }
            }
        }
    }
}
