use crate::domain::KanjiCard;
use crate::ui::{Button, ButtonVariant, Card, ErrorCard, TextInput, H1};
use dioxus::prelude::*;
use keikaku::application::use_cases::get_kanji_info::GetKanjiInfoUseCase;
use keikaku::domain::{dictionary::KanjiInfo, value_objects::NativeLanguage};

pub async fn fetch_kanji_info(query: String) -> Result<Option<KanjiInfo>, String> {
    if query.trim().is_empty() {
        return Ok(None);
    }

    match GetKanjiInfoUseCase::new().execute(&query) {
        Ok(kanji_info) => Ok(Some(kanji_info)),
        Err(err) => Err(format!("Ошибка получения информации о кандзи: {}", err)),
    }
}

#[component]
pub fn Kanji() -> Element {
    let mut current_query = use_signal(|| "語".to_string());
    let kanji_resource = use_resource(move || fetch_kanji_info(current_query()));
    let kanji_read = kanji_resource.read();

    match kanji_read.as_ref() {
        Some(Ok(kanji_info)) => rsx! {
            KanjiContent {
                initial_kanji_info: kanji_info.clone(),
                on_search: move |query: String| {
                    current_query.set(query);
                },
            }
        },
        Some(Err(err)) => rsx! {
            ErrorCard { message: err.clone() }
        },
        None => rsx! {
            div { class: "bg-bg min-h-screen text-text-main px-6 py-8", "Загрузка..." }
        },
    }
}

#[component]
fn KanjiContent(
    initial_kanji_info: Option<keikaku::domain::dictionary::KanjiInfo>,
    on_search: EventHandler<String>,
) -> Element {
    let query = use_signal(|| "語".to_string());
    let mut loading = use_signal(|| false);

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            H1 { "Кандзи" }

            Card {
                div { class: "grid grid-cols-9 gap-3",
                    div { class: "col-span-8",
                        TextInput {
                            label: "Кандзи",
                            value: query,
                            placeholder: "Введите кандзи для поиска...",
                        }
                    }

                    div { class: "col-span-1 flex items-end",
                        Button {
                            variant: ButtonVariant::Rainbow,
                            class: Some("w-full px-2 py-1.5 text-xs".to_string()),
                            onclick: move |_| {
                                loading.set(true);
                                let search_query = query();
                                on_search.call(search_query);
                                loading.set(false);
                            },
                            disabled: Some(loading()),
                            "Поиск"
                        }
                    }
                }
            }

            if let Some(info) = initial_kanji_info {
                KanjiCard {
                    kanji_info: info.clone(),
                    show_furigana: true,
                    native_language: NativeLanguage::Russian,
                    class: None,
                }
            }
        }
    }
}
