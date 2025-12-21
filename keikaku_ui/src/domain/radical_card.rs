use dioxus::prelude::*;

use crate::components::app_ui::{Card, Pill, StateTone};
use keikaku::domain::dictionary::RadicalInfo;

#[component]
pub fn RadicalCard(
    radical_info: RadicalInfo,
    show_kanji_list: bool,
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
            // Radical character and basic info
            div { class: "flex items-start gap-3 mb-3",
                span { class: "text-3xl font-bold text-purple-600 flex-shrink-0",
                    "{radical_info.radical()}"
                }
                div { class: "flex-1 min-w-0",
                    h4 { class: "font-semibold text-slate-800 text-lg leading-tight",
                        "{radical_info.name()}"
                    }
                    div { class: "flex items-center gap-2 mt-1",
                        Pill {
                            text: format!("{} черт", radical_info.stroke_count()),
                            tone: None,
                        }
                        Pill {
                            text: format!("JLPT N{}", radical_info.jlpt().as_number()),
                            tone: Some(
                                match radical_info.jlpt().as_number() {
                                    5 => StateTone::Success,
                                    4 => StateTone::Info,
                                    3 => StateTone::Warning,
                                    _ => StateTone::Neutral,
                                },
                            ),
                        }
                    }
                }
            }

            // Description
            p { class: "text-sm text-slate-700 leading-relaxed mb-3", "{radical_info.description()}" }

            // Kanji list (if requested)
            {
                if show_kanji_list {
                    let kanji_list = radical_info.kanji();
                    if !kanji_list.is_empty() {
                        rsx! {
                            div { class: "border-t border-slate-200 pt-3",
                                div { class: "text-xs text-slate-500 uppercase tracking-wide font-semibold mb-2",
                                    "Содержится в кандзи:"
                                }
                                div { class: "flex flex-wrap gap-1",
                                    for kanji in kanji_list.iter().take(10) { // Limit to first 10
                                        span { class: "inline-flex items-center justify-center w-8 h-8 bg-slate-100 rounded text-sm font-medium text-slate-700",
                                            "{kanji}"
                                        }
                                    }
                                    {
                                        if kanji_list.len() > 10 {
                                            rsx! {
                                                span { class: "text-xs text-slate-500 ml-2", "и ещё {kanji_list.len() - 10}..." }
                                            }
                                        } else {
                                            rsx! {}
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {}
                    }
                } else {
                    rsx! {}
                }
            }
        }
    }
}
