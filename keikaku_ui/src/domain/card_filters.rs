use dioxus::prelude::*;

use crate::ui::{Card, SearchInput};

#[derive(Clone, PartialEq)]
pub enum FilterStatus {
    All,
    Due,
    NotDue,
}

#[derive(Clone, PartialEq)]
pub enum SortBy {
    Date,
    Question,
    Answer,
}

#[component]
pub fn CardFilters(
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
                        SortBy::Date => "date",
                        SortBy::Question => "question",
                        SortBy::Answer => "answer",
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

#[component]
pub fn FilterSelect(
    label: String,
    value: String,
    options: Vec<(&'static str, &'static str)>,
    onchange: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "flex-1 min-w-[200px]",
            label { class: "block text-xs font-semibold text-slate-500 mb-2", {label} }
            select {
                class: "w-full px-4 py-2 rounded-xl bg-slate-50 border border-transparent text-slate-700 focus:bg-white focus:border-pink-200 focus:ring-4 focus:ring-pink-50 focus:outline-none",
                value,
                onchange: move |e| onchange.call(e.value()),
                for (opt_value , opt_label) in options {
                    option { value: opt_value, {opt_label} }
                }
            }
        }
    }
}
