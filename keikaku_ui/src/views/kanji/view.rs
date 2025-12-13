use crate::ui::{Button, ButtonVariant, Card, ErrorCard, Paragraph, TextInput, H1};
use dioxus::prelude::*;
use keikaku::application::use_cases::get_kanji_info::GetKanjiInfoUseCase;
use keikaku::domain::dictionary::KanjiInfo;

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
                div { class: "space-y-4",
                    TextInput {
                        label: "Кандзи",
                        value: query,
                        placeholder: "Введите кандзи для поиска...",
                    }

                    Button {
                        variant: ButtonVariant::Rainbow,
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

            if let Some(info) = initial_kanji_info {
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
