use dioxus::prelude::*;
use keikaku::domain::japanese::IsJapaneseText;

#[component]
pub fn FuriganaText(text: String, show_furigana: bool) -> Element {
    if show_furigana && text.has_furigana() {
        let furigana_html = text.as_furigana();
        rsx! {
            span {
                class: "inline-block",
                dangerous_inner_html: "{furigana_html}",
            }
        }
    } else {
        rsx! {
            span { class: "inline-block", {text} }
        }
    }
}
