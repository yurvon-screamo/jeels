use dioxus::prelude::*;

#[component]
pub fn FuriganaText(text: String, show_furigana: bool) -> Element {
    rsx! {
        span { class: "inline-block",
            {text}
            if show_furigana {
                span { class: "text-xs text-slate-500 block mt-1", "フリガナ…" }
            }
        }
    }
}
