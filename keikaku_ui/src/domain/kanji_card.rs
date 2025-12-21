use dioxus::prelude::*;

use crate::components::app_ui::{Card, Pill, StateTone};
use crate::domain::{PopularWordsGrid, RadicalGrid};
use keikaku::domain::{dictionary::KanjiInfo, value_objects::NativeLanguage};

#[component]
pub fn KanjiCard(
    kanji_info: KanjiInfo,
    show_furigana: bool,
    native_language: NativeLanguage,
    class: Option<String>,
) -> Element {
    let class_str = class.unwrap_or_default();
    let radicals = kanji_info
        .radicals()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let popular_words = kanji_info.popular_words_with_translations(&native_language);
    let has_radicals = !radicals.is_empty();
    let has_popular_words = !popular_words.is_empty();

    rsx! {
        Card { class: Some(format!("p-6 md:p-8 space-y-4 {}", class_str)),
            // Compact header: kanji on the left, description + metadata on the right
            div { class: "grid grid-cols-1 md:grid-cols-[auto,1fr] gap-x-6 gap-y-2 items-start",
                div { class: "text-5xl md:text-6xl font-bold leading-none text-slate-800",
                    "{kanji_info.kanji()}"
                }

                div { class: "min-w-0",
                    div { class: "text-base md:text-lg font-semibold text-slate-800 leading-snug",
                        "{kanji_info.description()}"
                    }

                    div { class: "mt-2 flex items-center gap-2 flex-wrap",
                        Pill {
                            text: format!("JLPT N{}", kanji_info.jlpt().as_number()),
                            tone: Some(
                                match kanji_info.jlpt().as_number() {
                                    5 => StateTone::Success,
                                    4 => StateTone::Info,
                                    3 => StateTone::Warning,
                                    _ => StateTone::Neutral,
                                },
                            ),
                        }
                        span { class: "text-xs md:text-sm text-slate-600 font-medium",
                            "Используется в {kanji_info.used_in()} словах"
                        }
                    }
                }
            }

            // Radicals section
            if has_radicals || has_popular_words {
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-3",
                    // Radicals section
                    if has_radicals {
                        div { class: if !has_popular_words { "md:col-span-2" } else { "" },
                            RadicalGrid {
                                radicals,
                                show_kanji_list: false,
                                dense: Some(true),
                            }
                        }
                    }

                    // Popular words section with translations
                    if has_popular_words {
                        div { class: if !has_radicals { "md:col-span-2" } else { "" },
                            PopularWordsGrid {
                                popular_words,
                                show_furigana,
                                dense: Some(true),
                            }
                        }
                    }
                }
            }
        }
    }
}
