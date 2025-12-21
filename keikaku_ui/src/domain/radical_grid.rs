use dioxus::prelude::*;

use crate::components::app_ui::{InfoGrid, InfoSection, InfoSectionTone};
use crate::domain::RadicalCard;
use keikaku::domain::dictionary::RadicalInfo;

#[component]
pub fn RadicalGrid(
    radicals: Vec<RadicalInfo>,
    show_kanji_list: bool,
    dense: Option<bool>,
) -> Element {
    let dense = dense.unwrap_or(false);

    // Compact mode is intended for embedding (e.g. inside KanjiCard) where the full RadicalCard
    // would be too tall. If kanji list is enabled, we keep the detailed layout.
    let use_dense = dense && !show_kanji_list;

    rsx! {
        InfoSection {
            title: "Радикалы".to_string(),
            tone: InfoSectionTone::Purple,
            class: if use_dense { Some("p-3 space-y-2".to_string()) } else { None },

            if use_dense {
                div { class: "grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2",
                    for radical_info in radicals {
                        div { class: "bg-white rounded-lg border border-slate-200 px-3 py-2",
                            div { class: "flex items-start gap-3",
                                span { class: "text-2xl font-bold text-purple-600 leading-none flex-shrink-0",
                                    "{radical_info.radical()}"
                                }
                                div { class: "min-w-0 flex-1",
                                    div { class: "text-sm font-semibold text-slate-800 leading-snug truncate",
                                        "{radical_info.name()}"
                                    }
                                    div { class: "mt-1 text-xs text-slate-600",
                                        "{radical_info.stroke_count()} черт"
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
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
}
