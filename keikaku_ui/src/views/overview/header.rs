use dioxus::prelude::*;

use crate::ui::{Card, Paragraph, H1};

#[component]
pub fn OverviewHeader(username: String) -> Element {
    rsx! {
        header { class: "flex flex-col md:flex-row md:items-center md:justify-between gap-3 ml-4 mb-6",
            div {
                H1 { class: Some("text-3xl font-extrabold text-slate-800".to_string()),
                    "Keikaku"
                }
                Paragraph { class: Some("text-slate-500".to_string()),
                    "Ваш прогресс в изучении японского языка."
                }
            }
            Card { class: Some("px-4 py-3 rounded-2xl bg-white/80 shadow-soft".to_string()),
                div { class: "text-xs text-slate-500 uppercase tracking-widest", "Аккаунт" }
                div { class: "text-sm font-semibold text-slate-700", {username} }
            }
        }
    }
}
