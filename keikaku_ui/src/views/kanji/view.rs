use dioxus::prelude::*;

use super::use_cases::use_kanji;
use crate::ui::{Button, ButtonVariant, Card, Paragraph, TextInput, H1};

#[component]
pub fn Kanji() -> Element {
    let kanji = use_kanji();

    let kanji_for_button = kanji.clone();
    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            H1 { "Кандзи" }

            Card {
                div { class: "space-y-4",
                    TextInput {
                        label: "Кандзи",
                        value: kanji.query,
                        placeholder: "Введите кандзи для поиска...",
                    }

                    Button {
                        variant: ButtonVariant::Rainbow,
                        onclick: move |_| {
                            let mut kanji = kanji_for_button.clone();
                            kanji.fetch_kanji_info();
                        },
                        disabled: Some((kanji.loading)()),
                        "Поиск"
                    }
                }
            }

            if let Some(info) = (kanji.info)() {
                Card {
                    div { class: "space-y-4",
                        div { class: "text-4xl font-bold text-center", "{info.kanji()}" }

                        div { class: "space-y-2",
                            h3 { class: "font-semibold", "Описание:" }
                            Paragraph { "{info.description()}" }
                        }

                        div { class: "space-y-2",
                            h3 { class: "font-semibold", "JLPT уровень:" }
                            Paragraph { "{info.jlpt()}" }
                        }

                        div { class: "space-y-2",
                            h3 { class: "font-semibold", "Используется в словах:" }
                            Paragraph { "{info.used_in()}" }
                        }

                        if !info.radicals().is_empty() {
                            div { class: "space-y-2",
                                h3 { class: "font-semibold", "Радикалы:" }
                                p {
                                    "{info.radicals().iter().map(|r| r.radical().to_string()).collect::<Vec<_>>().join(\", \")}"
                                }
                            }
                        }

                        if !info.popular_words().is_empty() {
                            div { class: "space-y-2",
                                h3 { class: "font-semibold", "Популярные слова:" }
                                ul { class: "list-disc list-inside",
                                    for word in info.popular_words() {
                                        li { "{word}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
