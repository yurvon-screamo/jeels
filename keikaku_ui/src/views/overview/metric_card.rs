use dioxus::prelude::*;

use crate::components::app_ui::{Paragraph, StateTone};

#[component]
pub fn MetricCard(
    label: String,
    value: String,
    hint: Option<String>,
    tone: Option<StateTone>,
) -> Element {
    let tone = tone.unwrap_or(StateTone::Neutral);
    let (bg, text) = tone.classes();

    rsx! {
        div { class: "p-2 rounded-2xl shadow-soft border border-slate-50 {bg}",
            span { class: "text-xs font-semibold uppercase tracking-widest {text}", {label} }
            h3 { class: "text-2xl font-bold mt-2 text-slate-800", {value} }
            if let Some(hint_text) = hint {
                Paragraph { class: Some(format!("mt-1 text-xs {}", text)), "{hint_text}" }
            }
        }
    }
}
