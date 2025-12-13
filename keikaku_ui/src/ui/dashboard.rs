use dioxus::prelude::*;

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
                super::Paragraph { class: Some(format!("mt-1 text-xs {}", text)), "{hint_text}" }
            }
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
