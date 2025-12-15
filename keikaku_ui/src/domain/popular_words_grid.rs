use dioxus::prelude::*;

use crate::domain::PopularWordCard;
use crate::ui::{InfoGrid, InfoSection, InfoSectionTone};
use keikaku::domain::dictionary::PopularWord;

#[component]
pub fn PopularWordsGrid(popular_words: Vec<PopularWord>, show_furigana: bool) -> Element {
    rsx! {
        InfoSection {
            title: "Популярные слова".to_string(),
            tone: InfoSectionTone::Blue,
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
