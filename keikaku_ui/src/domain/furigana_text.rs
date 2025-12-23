use dioxus::prelude::*;
use keikaku::domain::japanese::IsJapaneseText;

#[component]
pub fn FuriganaText(text: String, show_furigana: bool, class: Option<String>) -> Element {
    let class_str = class.unwrap_or_else(|| "inline-block".to_string());

    if show_furigana && text.has_furigana().unwrap_or(false) {
        let furigana_html = text.as_furigana().unwrap_or_default();
        rsx! {
            span { class: "{class_str}", dangerous_inner_html: "{furigana_html}" }
        }
    } else {
        rsx! {
            span { class: "{class_str}", {text} }
        }
    }
}
