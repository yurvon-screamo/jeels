use dioxus::prelude::*;

use super::WordCard;
use crate::ui::Pill;

#[component]
pub fn CardAnswer(
    question: String,
    answer: String,
    show_furigana: bool,
    examples: Option<Vec<String>>,
) -> Element {
    let examples = examples.unwrap_or_default();

    rsx! {
        div { class: "p-8 bg-gradient-to-br from-pink-50 to-purple-50 border-2 border-pink-200 rounded-3xl shadow-sm min-h-[200px] flex flex-col justify-center space-y-4",
            div { class: "text-center space-y-2",
                WordCard { text: answer, show_furigana }
                div { class: "flex gap-2 justify-center flex-wrap mt-4",
                    if !examples.is_empty() {
                        for example in examples {
                            Pill { text: example, tone: None }
                        }
                    } else {
                        Pill { text: "Примеры".to_string(), tone: None }
                        Pill { text: "Синонимы".to_string(), tone: None }
                    }
                }
            }
        }
    }
}
