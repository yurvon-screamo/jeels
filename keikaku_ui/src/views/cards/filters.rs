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
                            value: "new",
                            label: "Новые",
                        },
                        FilterOption {
                            value: "low_stability",
                            label: "Низкая стабильность",
                        },
                        FilterOption {
                            value: "in_progress",
                            label: "В процессе",
                        },
                        FilterOption {
                            value: "learned",
                            label: "Изученные",
                        },
                    ],
                    selected: Some(FilterOption {
                        value: match filter_status() {
                            FilterStatus::All => "all",
                            FilterStatus::New => "new",
                            FilterStatus::LowStability => "low_stability",
                            FilterStatus::InProgress => "in_progress",
                            FilterStatus::Learned => "learned",
                        },
                        label: match filter_status() {
                            FilterStatus::All => "Все",
                            FilterStatus::New => "Новые",
                            FilterStatus::LowStability => "Низкая стабильность",
                            FilterStatus::InProgress => "В процессе",
                            FilterStatus::Learned => "Изученные",
                        },
                    }),
                    onselect: move |selected: FilterOption| {
                        filter_status
                            .set(
                                match selected.value {
                                    "new" => FilterStatus::New,
                                    "low_stability" => FilterStatus::LowStability,
                                    "in_progress" => FilterStatus::InProgress,
                                    "learned" => FilterStatus::Learned,
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
                        FilterOption {
                            value: "difficulty",
                            label: "По сложности",
                        },
                        FilterOption {
                            value: "stability",
                            label: "По стабильности",
                        },
                    ],
                    selected: Some(FilterOption {
                        value: match sort_by() {
                            SortBy::Date => "date",
                            SortBy::Question => "question",
                            SortBy::Answer => "answer",
                            SortBy::Difficulty => "difficulty",
                            SortBy::Stability => "stability",
                        },
                        label: match sort_by() {
                            SortBy::Date => "По дате",
                            SortBy::Question => "По вопросу",
                            SortBy::Answer => "По ответу",
                            SortBy::Difficulty => "По сложности",
                            SortBy::Stability => "По стабильности",
                        },
                    }),
                    onselect: move |selected: FilterOption| {
                        sort_by
                            .set(
                                match selected.value {
                                    "question" => SortBy::Question,
                                    "answer" => SortBy::Answer,
                                    "difficulty" => SortBy::Difficulty,
                                    "stability" => SortBy::Stability,
                                    _ => SortBy::Date,
                                },
                            );
                    },
                }
            }
        }
    }
}
