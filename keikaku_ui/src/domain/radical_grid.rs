use dioxus::prelude::*;

use crate::domain::RadicalCard;
use crate::ui::{InfoGrid, InfoSection, InfoSectionTone};
use keikaku::domain::dictionary::RadicalInfo;

#[component]
pub fn RadicalGrid(radicals: Vec<RadicalInfo>, show_kanji_list: bool) -> Element {
    rsx! {
        InfoSection {
            title: "Радикалы".to_string(),
            tone: InfoSectionTone::Purple,
            InfoGrid {
                for radical_info in radicals {
                    RadicalCard {
                        radical_info: radical_info.clone(),
                        show_kanji_list,
                        class: None,
                    }
                }
            }
        }
    }
}
