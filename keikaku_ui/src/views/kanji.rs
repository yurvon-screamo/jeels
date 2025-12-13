use crate::components::{
    Button, ButtonVariant, Card, LoadingState, Paragraph, SectionHeader, TextInput,
};
use crate::hooks::{use_kanji, UseKanji};
use dioxus::prelude::*;

#[component]
pub fn Kanji() -> Element {
    let kanji = use_kanji();

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            SectionHeader {
                title: "Кандзи".to_string(),
                subtitle: Some("cli: kanji <символ>".to_string()),
                actions: None,
            }

            KanjiSearch { kanji: kanji.clone() }

            if (kanji.loading)() {
                LoadingState { message: Some("Загрузка информации о кандзи...".to_string()) }
            } else if let Some(data) = (kanji.info)() {
                KanjiInfo { data }
            } else {
                KanjiNotFound {}
            }
        }
    }
}

#[component]
fn KanjiSearch(kanji: UseKanji) -> Element {
    rsx! {
        Card { class: Some("space-y-4".to_string()),
            TextInput {
                label: Some("КАНДЗИ".to_string()),
                placeholder: Some("一".to_string()),
                value: Some(kanji.query),
                oninput: Some(EventHandler::new(move |e: Event<FormData>| kanji.query.set(e.value()))),
                class: None,
                r#type: None,
            }
            Button {
                variant: ButtonVariant::Rainbow,
                class: Some("w-full".to_string()),
                onclick: move |_| kanji.fetch_kanji_info(),
                "Показать"
            }
        }
    }
}

#[component]
fn KanjiInfo(data: keikaku::domain::dictionary::KanjiInfo) -> Element {
    rsx! {
        Card { class: Some("grid grid-cols-1 md:grid-cols-2 gap-4".to_string()),
            KanjiBasicInfo { data: data.clone() }
            KanjiRadicals { radicals: data.radicals().iter().map(|r| (*r).clone()).collect() }
        }
    }
}

#[component]
fn KanjiBasicInfo(data: keikaku::domain::dictionary::KanjiInfo) -> Element {
    rsx! {
        Card { class: Some("p-4 bg-slate-50 border border-slate-100 rounded-2xl".to_string()),
            Paragraph { class: Some("text-4xl font-bold text-slate-900".to_string()),
                {data.kanji().to_string()}
            }
            Paragraph { class: Some("text-sm text-slate-500".to_string()),
                "JLPT {data.jlpt()}, использований: {data.used_in()}"
            }
            Paragraph { class: Some("text-sm text-slate-600 mt-2".to_string()), {data.description()} }
        }
    }
}

#[component]
fn KanjiRadicals(radicals: Vec<keikaku::domain::dictionary::RadicalInfo>) -> Element {
    rsx! {
        Card { class: Some("p-4 bg-white border border-slate-100 rounded-2xl".to_string()),
            Paragraph { class: Some("text-sm font-semibold text-slate-700".to_string()), "Радикалы" }
            div { class: "flex flex-col gap-2 mt-2",
                for radical in radicals.iter() {
                    RadicalItem { radical: radical.clone() }
                }
            }
        }
    }
}

#[component]
fn RadicalItem(radical: keikaku::domain::dictionary::RadicalInfo) -> Element {
    rsx! {
        Card { class: Some("p-3 bg-slate-50 border border-slate-100 rounded-xl".to_string()),
            Paragraph { class: Some("text-lg font-bold text-slate-800".to_string()),
                {radical.radical().to_string()}
            }
            Paragraph { class: Some("text-xs text-slate-500".to_string()), {radical.name().to_string()} }
        }
    }
}

#[component]
fn KanjiNotFound() -> Element {
    rsx! {
        Paragraph { class: Some("text-sm text-slate-500".to_string()),
            "Нет данных по символу."
        }
    }
}
