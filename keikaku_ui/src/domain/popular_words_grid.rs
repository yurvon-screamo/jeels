use dioxus::prelude::*;

use crate::domain::FuriganaText;
use crate::domain::PopularWordCard;
use crate::ui::{InfoGrid, InfoSection, InfoSectionTone};
use keikaku::domain::dictionary::PopularWord;

#[component]
pub fn PopularWordsGrid(
    popular_words: Vec<PopularWord>,
    show_furigana: bool,
    dense: Option<bool>,
) -> Element {
    let dense = dense.unwrap_or(false);

    rsx! {
        InfoSection {
            title: "Популярные слова".to_string(),
            tone: InfoSectionTone::Blue,
            class: if dense { Some("p-3 space-y-2".to_string()) } else { None },

            if dense {
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-2",
                    for popular_word in popular_words {
                        div { class: "bg-white rounded-lg border border-slate-200 px-3 py-2",
                            div { class: "flex flex-col gap-1",
                                FuriganaText {
                                    text: popular_word.word().to_string(),
                                    show_furigana,
                                    class: Some("text-base font-semibold text-slate-800".to_string()),
                                }
                                div { class: "text-sm text-slate-600", "{popular_word.translation()}" }
                            }
                        }
                    }
                }
            } else {
                InfoGrid {
                    for popular_word in popular_words {
                        PopularWordCard {
                            popular_word: popular_word.clone(),
                            show_furigana,
                            class: None,
                        }
                    }
                }
            }
        }
    }
}
