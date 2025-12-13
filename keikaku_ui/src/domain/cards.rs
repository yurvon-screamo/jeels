use crate::ui::{Button, ButtonVariant, Card, LoadingState, Paragraph, Pill, MetricTone};
use dioxus::prelude::*;

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

#[derive(Clone, PartialEq)]
pub struct UiCard {
    pub id: String,
    pub question: String,
    pub answer: String,
    pub next_review: String,
    pub due: bool,
}

#[component]
pub fn CardsHeader(total_count: usize, due_count: usize) -> Element {
    use crate::ui::SectionHeader;

    rsx! {
        SectionHeader {
            title: "Карточки".to_string(),
            subtitle: Some(
                "Управление карточками для изучения".to_string(),
            ),
            actions: Some(rsx! {
                Button {
                    variant: ButtonVariant::Rainbow,
                    class: Some("w-auto px-6".to_string()),
                    onclick: move |_| {},
                    "+ Создать карточку"
                }
            }),
        }
    }
}

#[component]
pub fn CardsStats(total_count: usize, due_count: usize, filtered_count: usize) -> Element {
    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
            Card { class: Some("p-4".to_string()),
                div { class: "flex items-center justify-between",
                    div {
                        span { class: "text-xs font-semibold text-slate-500 uppercase",
                            "Всего карточек"
                        }
                        div { class: "text-2xl font-bold text-slate-800 mt-1", "{total_count}" }
                    }
                }
            }
            Card { class: Some("p-4".to_string()),
                div { class: "flex items-center justify-between",
                    div {
                        span { class: "text-xs font-semibold text-slate-500 uppercase",
                            "К повторению"
                        }
                        div { class: "text-2xl font-bold text-amber-600 mt-1", "{due_count}" }
                    }
                }
            }
            Card { class: Some("p-4".to_string()),
                div { class: "flex items-center justify-between",
                    div {
                        span { class: "text-xs font-semibold text-slate-500 uppercase",
                            "Показано"
                        }
                        div { class: "text-2xl font-bold text-slate-800 mt-1", "{filtered_count}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn CardsFilters(
    search: Signal<String>,
    filter_status: Signal<FilterStatus>,
    sort_by: Signal<SortBy>,
) -> Element {
    use crate::ui::SearchInput;

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

#[component]
pub fn CardsList(
    cards: Vec<UiCard>,
    loading: bool,
    on_edit: EventHandler<String>,
    on_delete: EventHandler<String>,
) -> Element {
    if loading {
        rsx! {
            Card { class: Some("p-12 text-center".to_string()),
                LoadingState { message: Some("Загрузка карточек...".to_string()) }
            }
        }
    } else if cards.is_empty() {
        rsx! {
            Card { class: Some("p-12 text-center".to_string()), EmptyCardsState {} }
        }
    } else {
        rsx! {
            Card { class: Some("space-y-3".to_string()),
                for card in cards {
                    CardRow {
                        card: card.clone(),
                        on_edit: move |id| on_edit.call(id),
                        on_delete: move |_| on_delete.call(card.id.clone()),
                    }
                }
            }
        }
    }
}

#[component]
pub fn EmptyCardsState() -> Element {
    rsx! {
        div { class: "space-y-4",
            div { class: "text-4xl mb-4", "📝" }
            Paragraph { class: Some("text-lg font-semibold text-slate-700".to_string()),
                "У вас пока нет карточек"
            }
            Paragraph { class: Some("text-sm text-slate-500".to_string()),
                "Создайте первую карточку, чтобы начать обучение"
            }
            Button {
                variant: ButtonVariant::Rainbow,
                class: Some("mt-4".to_string()),
                onclick: move |_| {},
                "Создать первую карточку"
            }
        }
    }
}

#[component]
pub fn CardRow(
    card: UiCard,
    on_edit: EventHandler<String>,
    on_delete: EventHandler<String>,
) -> Element {
    use crate::ui::H2;

    let card_id = card.id.clone();

    rsx! {
        div { class: "p-5 border border-slate-100 rounded-2xl shadow-soft hover:shadow-soft-hover transition-all duration-300 bg-white",
            div { class: "flex items-start justify-between gap-4",
                div { class: "flex-1 space-y-2",
                    H2 { class: Some("text-xl font-bold text-slate-800".to_string()),
                        {card.question.clone()}
                    }
                    Paragraph { class: Some("text-sm text-slate-600 leading-relaxed".to_string()),
                        {card.answer.clone()}
                    }
                    div { class: "flex items-center gap-2 flex-wrap mt-3",
                        Pill {
                            text: format!("Повтор: {}", card.next_review),
                            tone: Some(if card.due { MetricTone::Warning } else { MetricTone::Info }),
                        }
                        Pill {
                            text: if card.due { "К повторению".to_string() } else { "Запланирована".to_string() },
                            tone: Some(if card.due { MetricTone::Warning } else { MetricTone::Neutral }),
                        }
                    }
                }
                div { class: "flex gap-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        class: Some("w-auto px-4 py-2 text-sm".to_string()),
                        onclick: move |_| on_edit.call(card.id.clone()),
                        "Редактировать"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        class: Some(
                            "w-auto px-4 py-2 text-sm text-red-600 border-red-200 hover:border-red-300 hover:text-red-700"
                                .to_string(),
                        ),
                        onclick: move |_| on_delete.call(card_id.clone()),
                        "Удалить"
                    }
                }
            }
        }
    }
}
