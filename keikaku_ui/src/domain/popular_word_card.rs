use dioxus::prelude::*;

use crate::components::app_ui::Card;
use crate::domain::FuriganaText;
use keikaku::domain::dictionary::PopularWord;

#[component]
pub fn PopularWordCard(
    popular_word: PopularWord,
    show_furigana: bool,
    class: Option<String>,
) -> Element {
    let class_str = class.unwrap_or_default();

    rsx! {
        Card {
            class: Some(
                format!(
                    "bg-white rounded-lg p-4 border border-slate-200 hover:shadow-md transition-shadow duration-200 {}",
                    class_str,
                ),
            ),
            // Word with furigana and translation
            div { class: "flex flex-col gap-2",
                // Japanese word with furigana
                FuriganaText {
                    text: popular_word.word().to_string(),
                    show_furigana,
                    class: Some("text-xl font-medium text-slate-800".to_string()),
                }

                // Translation
                div { class: "text-base text-slate-600 font-medium", "{popular_word.translation()}" }
            }
        }
    }
}
