use dioxus::prelude::*;

use crate::domain::{PopularWordsGrid, RadicalGrid};
use crate::ui::{Card, InfoSection, InfoSectionTone, Pill};
use keikaku::domain::{dictionary::KanjiInfo, value_objects::NativeLanguage};

#[component]
pub fn KanjiCard(
    kanji_info: KanjiInfo,
    show_furigana: bool,
    native_language: NativeLanguage,
    class: Option<String>,
) -> Element {
    let class_str = class.unwrap_or_default();

    rsx! {
        Card { class: Some(format!("space-y-6 {}", class_str)),
            // Header section with kanji and description in 3:1 grid
            div { class: "grid grid-cols-4 gap-6",
                // Left column (3 parts): Kanji character and metadata
                div { class: "col-span-3 space-y-4",
                    // Main kanji character
                    div { class: "text-5xl md:text-6xl font-bold text-slate-800",
                        "{kanji_info.kanji()}"
                    }

                    // Metadata: JLPT level and usage count
                    div { class: "flex items-center gap-3 flex-wrap",
                        Pill {
                            text: format!("JLPT N{}", kanji_info.jlpt().as_number()),
                            tone: Some(
                                match kanji_info.jlpt().as_number() {
                                    5 => crate::ui::StateTone::Success,
                                    4 => crate::ui::StateTone::Info,
                                    3 => crate::ui::StateTone::Warning,
                                    _ => crate::ui::StateTone::Neutral,
                                },
                            ),
                        }
                        span { class: "text-sm text-slate-600 font-medium",
                            "Используется в {kanji_info.used_in()} словах"
                        }
                    }
                }

                // Right column (1 part): Description
                div { class: "col-span-1",
                    InfoSection {
                        title: "Значение".to_string(),
                        tone: InfoSectionTone::Neutral,
                        p { class: "text-slate-700 leading-relaxed text-sm",
                            "{kanji_info.description()}"
                        }
                    }
                }
            }

            // Radicals section
            {
                let radicals = kanji_info.radicals();
                if !radicals.is_empty() {
                    rsx! {
                        RadicalGrid {
                            radicals: radicals.into_iter().cloned().collect(),
                            show_kanji_list: false,
                        }
                    }
                } else {
                    rsx! {}
                }
            }

            // Popular words section with translations
            {
                let popular_words = kanji_info.popular_words_with_translations(&native_language);
                if !popular_words.is_empty() {
                    rsx! {
                        PopularWordsGrid { popular_words, show_furigana }
                    }
                } else {
                    rsx! {}
                }
            }
        }
    }
}
