use dioxus::prelude::*;

use crate::components::app_ui::Card;
use crate::components::input::Input;
use crate::components::select::{
    Select, SelectItemIndicator, SelectList, SelectOption, SelectTrigger, SelectValue,
};
use crate::views::cards::{FilterStatus, SortBy};

#[component]
pub fn CardsFilters(
    search: Signal<String>,
    filter_status: Signal<FilterStatus>,
    sort_by: Signal<SortBy>,
) -> Element {
    let status_value = match filter_status() {
        FilterStatus::All => "all",
        FilterStatus::New => "new",
        FilterStatus::LowStability => "low_stability",
        FilterStatus::HighDifficulty => "high_difficulty",
        FilterStatus::InProgress => "in_progress",
        FilterStatus::Learned => "learned",
    }
    .to_string();

    let sort_value = match sort_by() {
        SortBy::Date => "date",
        SortBy::Question => "question",
        SortBy::Answer => "answer",
        SortBy::Difficulty => "difficulty",
        SortBy::Stability => "stability",
    }
    .to_string();

    rsx! {
        Card { class: Some("space-y-4".to_string()),
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "ПОИСК" }
                Input {
                    placeholder: "Поиск по слову или переводу...",
                    value: search(),
                    oninput: move |e: FormEvent| search.set(e.value()),
                }
            }
            div { class: "flex flex-wrap gap-3",
                div { class: "flex-1 min-w-[200px]",
                    label { class: "text-sm font-medium", "СТАТУС" }
                    Select::<String> {
                        value: Some(Some(status_value.clone())),
                        on_value_change: move |v: Option<String>| {
                            if let Some(v) = v {
                                filter_status
                                    .set(
                                        match v.as_str() {
                                            "new" => FilterStatus::New,
                                            "low_stability" => FilterStatus::LowStability,
                                            "high_difficulty" => FilterStatus::HighDifficulty,
                                            "in_progress" => FilterStatus::InProgress,
                                            "learned" => FilterStatus::Learned,
                                            _ => FilterStatus::All,
                                        },
                                    );
                            }
                        },
                        placeholder: "Выберите...",
                        SelectTrigger { aria_label: "Статус", width: "100%", SelectValue {} }
                        SelectList { aria_label: "Статус",
                            SelectOption::<String> { index: 0usize, value: "all".to_string(),
                                "Все"
                                SelectItemIndicator {}
                            }
                            SelectOption::<String> { index: 1usize, value: "new".to_string(),
                                "Новые"
                                SelectItemIndicator {}
                            }
                            SelectOption::<String> {
                                index: 2usize,
                                value: "low_stability".to_string(),
                                "Низкая стабильность"
                                SelectItemIndicator {}
                            }
                            SelectOption::<String> {
                                index: 3usize,
                                value: "high_difficulty".to_string(),
                                "Высокая сложность"
                                SelectItemIndicator {}
                            }
                            SelectOption::<String> {
                                index: 4usize,
                                value: "in_progress".to_string(),
                                "В процессе"
                                SelectItemIndicator {}
                            }
                            SelectOption::<String> { index: 5usize, value: "learned".to_string(),
                                "Изученные"
                                SelectItemIndicator {}
                            }
                        }
                    }
                }
                div { class: "flex-1 min-w-[200px]",
                    label { class: "text-sm font-medium", "СОРТИРОВКА" }
                    Select::<String> {
                        value: Some(Some(sort_value.clone())),
                        on_value_change: move |v: Option<String>| {
                            if let Some(v) = v {
                                sort_by
                                    .set(
                                        match v.as_str() {
                                            "question" => SortBy::Question,
                                            "answer" => SortBy::Answer,
                                            "difficulty" => SortBy::Difficulty,
                                            "stability" => SortBy::Stability,
                                            _ => SortBy::Date,
                                        },
                                    );
                            }
                        },
                        placeholder: "Выберите...",
                        SelectTrigger { aria_label: "Сортировка", width: "100%", SelectValue {} }
                        SelectList { aria_label: "Сортировка",
                            SelectOption::<String> { index: 0usize, value: "date".to_string(),
                                "По дате"
                                SelectItemIndicator {}
                            }
                            SelectOption::<String> { index: 1usize, value: "question".to_string(),
                                "По вопросу"
                                SelectItemIndicator {}
                            }
                            SelectOption::<String> { index: 2usize, value: "answer".to_string(),
                                "По ответу"
                                SelectItemIndicator {}
                            }
                            SelectOption::<String> {
                                index: 3usize,
                                value: "difficulty".to_string(),
                                "По сложности"
                                SelectItemIndicator {}
                            }
                            SelectOption::<String> { index: 4usize, value: "stability".to_string(),
                                "По стабильности"
                                SelectItemIndicator {}
                            }
                        }
                    }
                }
            }
        }
    }
}
