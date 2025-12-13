use dioxus::prelude::*;

use crate::{
    domain::FilterStatus,
    ui::{Card, SearchInput, Select},
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
    sort_by: Signal<crate::domain::SortBy>,
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
                FilterSelect {
                    label: "СТАТУС".to_string(),
                    value: match filter_status() {
                        FilterStatus::All => "all",
                        FilterStatus::Due => "due",
                        FilterStatus::NotDue => "not_due",
                    },
                    options: vec![
                        ("all", "Все"),
                        ("due", "К повторению"),
                        ("not_due", "Запланированы"),
                    ],
                    onchange: move |value: String| {
                        filter_status
                            .set(
                                match value.as_str() {
                                    "due" => FilterStatus::Due,
                                    "not_due" => FilterStatus::NotDue,
                                    _ => FilterStatus::All,
                                },
                            );
                    },
                }
                FilterSelect {
                    label: "СОРТИРОВКА".to_string(),
                    value: match sort_by() {
                        crate::domain::SortBy::Date => "date",
                        crate::domain::SortBy::Question => "question",
                        crate::domain::SortBy::Answer => "answer",
                    },
                    options: vec![
                        ("date", "По дате"),
                        ("question", "По вопросу"),
                        ("answer", "По ответу"),
                    ],
                    onchange: move |value: String| {
                        sort_by
                            .set(
                                match value.as_str() {
                                    "question" => crate::domain::SortBy::Question,
                                    "answer" => crate::domain::SortBy::Answer,
                                    _ => crate::domain::SortBy::Date,
                                },
                            );
                    },
                }
            }
        }
    }
}

#[component]
fn FilterSelect(
    label: String,
    value: String,
    options: Vec<(&'static str, &'static str)>,
    onchange: EventHandler<String>,
) -> Element {
    let filter_options: Vec<FilterOption> = options
        .iter()
        .map(|(val, lbl)| FilterOption {
            value: val,
            label: lbl,
        })
        .collect();

    let filter_options_clone = filter_options.clone();
    let selected_option = use_memo(move || {
        filter_options_clone
            .iter()
            .find(|opt| opt.value == value)
            .cloned()
    });

    rsx! {
        div { class: "flex-1 min-w-[200px]",
            Select {
                label: Some(label),
                options: filter_options,
                selected: selected_option,
                onselect: move |selected: FilterOption| {
                    onchange.call(selected.value.to_string());
                },
            }
        }
    }
}
