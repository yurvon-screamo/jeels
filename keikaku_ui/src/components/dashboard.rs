use dioxus::prelude::*;

use super::{Card, Paragraph};

#[derive(PartialEq, Clone)]
pub enum MetricTone {
    Neutral,
    Success,
    Warning,
    Info,
}

impl MetricTone {
    fn classes(&self) -> (&'static str, &'static str) {
        match self {
            MetricTone::Neutral => ("bg-slate-50 text-slate-600", "bg-slate-100 text-slate-500"),
            MetricTone::Success => (
                "bg-emerald-50 text-emerald-600",
                "bg-emerald-100 text-emerald-500",
            ),
            MetricTone::Warning => ("bg-amber-50 text-amber-600", "bg-amber-100 text-amber-500"),
            MetricTone::Info => ("bg-cyan-50 text-cyan-600", "bg-cyan-100 text-cyan-500"),
        }
    }
}

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
pub fn MetricCard(
    label: String,
    value: String,
    hint: Option<String>,
    tone: Option<MetricTone>,
) -> Element {
    let tone = tone.unwrap_or(MetricTone::Neutral);
    let (bg, text) = tone.classes();

    rsx! {
        div { class: "p-4 rounded-2xl shadow-soft border border-slate-50 {bg}",
            span { class: "text-xs font-semibold uppercase tracking-widest {text}", {label} }
            h3 { class: "text-2xl font-bold mt-2 text-slate-800", {value} }
            if let Some(hint_text) = hint {
                Paragraph { class: Some(format!("mt-1 text-xs {}", text)), "{hint_text}" }
            }
        }
    }
}

#[component]
pub fn ToolbarTab(
    label: String,
    description: String,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        button {
            class: format!(
                "min-w-[140px] px-4 py-3 rounded-2xl text-left border transition-all duration-200 shadow-sm focus:outline-none focus:ring-4 focus:ring-pink-50 {}",
                if active {
                    "bg-rainbow-soft text-slate-800 border-transparent shadow-glow"
                } else {
                    "bg-white/80 text-slate-600 border-slate-100 hover:border-accent-pink/50"
                },
            ),
            onclick: move |e| onclick.call(e),
            div { class: "font-semibold text-sm", {label} }
            div { class: "text-xs text-slate-500 mt-1", {description} }
        }
    }
}

#[component]
pub fn Pill(text: String, tone: Option<MetricTone>) -> Element {
    let tone = tone.unwrap_or(MetricTone::Neutral);
    let (bg, class_text) = tone.classes();
    rsx! {
        span { class: "inline-flex items-center gap-1 px-3 py-1 rounded-full text-xs font-semibold {bg} {class_text}",
            {text}
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
