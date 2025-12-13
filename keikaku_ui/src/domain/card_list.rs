use dioxus::prelude::*;

use crate::ui::{Button, ButtonVariant, Card, LoadingState, MetricTone, Paragraph, Pill};

#[derive(Clone, PartialEq)]
pub struct UiCard {
    pub id: String,
    pub question: String,
    pub answer: String,
    pub next_review: String,
    pub due: bool,
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
