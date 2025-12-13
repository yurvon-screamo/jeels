use dioxus::prelude::*;

use super::FuriganaText;

#[component]
pub fn WordCard(text: String, show_furigana: bool, class: Option<String>) -> Element {
    let text_class = if let Some(custom_class) = class {
        custom_class
    } else {
        "text-4xl md:text-5xl font-bold".to_string()
    };

    rsx! {
        div { class: "p-4 bg-gradient-to-br from-slate-50 to-slate-100 border-2 border-slate-200 rounded-2xl shadow-sm min-h-[120px] flex flex-col justify-center items-center text-center space-y-2",
            FuriganaText { text, show_furigana, class: Some(text_class) }
        }
    }
}
