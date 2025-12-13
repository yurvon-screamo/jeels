use dioxus::prelude::*;

use super::FuriganaText;

#[component]
pub fn WordCard(text: String, show_furigana: bool) -> Element {
    rsx! {
        div { class: "p-4 bg-gradient-to-br from-slate-50 to-slate-100 border-2 border-slate-200 rounded-2xl shadow-sm min-h-[120px] flex flex-col justify-center items-center text-center space-y-2",
            FuriganaText { text, show_furigana }
        }
    }
}
