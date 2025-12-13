use dioxus::prelude::*;

use crate::{
    ui::{Card, LabeledSelect, SearchInput},
    views::cards::{FilterStatus, SortBy},
};

#[derive(Clone, PartialEq)]
struct FilterOption {
    value: &'static str,
    label: &'static str,
}

impl std::fmt::Display for FilterOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

#[component]
pub fn CardsFilters(
    search: Signal<String>,
    filter_status: Signal<FilterStatus>,
    sort_by: Signal<SortBy>,
) -> Element {
    rsx! {
        Card { class: Some("space-y-4".to_string()),
            SearchInput {
                label: Some("ПОИСК".to_string()),
                placeholder: Some("Поиск по слову или переводу...".to_string()),
                value: Some(search),
                oninput: Some(EventHandler::new(move |e: Event<FormData>| search.set(e.value()))),
            }
            div { class: "flex flex-wrap gap-3",
                LabeledSelect {
                    label: "СТАТУС".to_string(),
                    options: vec![
                        FilterOption {
                            value: "all",
                            label: "Все",
                        },
                        FilterOption {
                            value: "due",
                            label: "К повторению",
                        },
                        FilterOption {
                            value: "not_due",
                            label: "Запланированы",
                        },
                    ],
                    selected: Some(FilterOption {
                        value: match filter_status() {
                            FilterStatus::All => "all",
                            FilterStatus::Due => "due",
                            FilterStatus::NotDue => "not_due",
                        },
                        label: "",
                    }),
                    onselect: move |selected: FilterOption| {
                        filter_status
                            .set(
                                match selected.value {
                                    "due" => FilterStatus::Due,
                                    "not_due" => FilterStatus::NotDue,
                                    _ => FilterStatus::All,
                                },
                            );
                    },
                }
                LabeledSelect {
                    label: "СОРТИРОВКА".to_string(),
                    options: vec![
                        FilterOption {
                            value: "date",
                            label: "По дате",
                        },
                        FilterOption {
                            value: "question",
                            label: "По вопросу",
                        },
                        FilterOption {
                            value: "answer",
                            label: "По ответу",
                        },
                    ],
                    selected: Some(FilterOption {
                        value: match sort_by() {
                            SortBy::Date => "date",
                            SortBy::Question => "question",
                            SortBy::Answer => "answer",
                        },
                        label: "",
                    }),
                    onselect: move |selected: FilterOption| {
                        sort_by
                            .set(
                                match selected.value {
                                    "question" => SortBy::Question,
                                    "answer" => SortBy::Answer,
                                    _ => SortBy::Date,
                                },
                            );
                    },
                }
            }
        }
    }
}
