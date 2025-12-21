use dioxus::prelude::*;

#[derive(PartialEq, Clone)]
pub enum StateTone {
    Neutral,
    Success,
    Warning,
    Info,
}

impl StateTone {
    pub fn classes(&self) -> (&'static str, &'static str) {
        match self {
            StateTone::Neutral => ("bg-slate-50 text-slate-600", "bg-slate-100 text-slate-500"),
            StateTone::Success => (
                "bg-emerald-50 text-emerald-600",
                "bg-emerald-100 text-emerald-500",
            ),
            StateTone::Warning => ("bg-amber-50 text-amber-600", "bg-amber-100 text-amber-500"),
            StateTone::Info => ("bg-cyan-50 text-cyan-600", "bg-cyan-100 text-cyan-500"),
        }
    }
}

#[component]
pub fn Pill(text: String, tone: Option<StateTone>) -> Element {
    let tone = tone.unwrap_or(StateTone::Neutral);
    let (bg, class_text) = tone.classes();

    rsx! {
        span { class: "inline-flex items-center gap-1 px-3 py-1 rounded-full text-xs font-semibold {bg} {class_text}",
            {text}
        }
    }
}
