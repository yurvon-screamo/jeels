use dioxus::prelude::*;
use std::rc::Rc;

use crate::{
    ui::{Button, ButtonVariant, Card, EmptyState, Grid, LoadingState, Paragraph, Pill, StateTone},
    views::cards::UiCard,
};

#[component]
pub fn CardsGrid(
    cards: Vec<UiCard>,
    loading: bool,
    on_edit: EventHandler<UiCard>,
    on_delete: EventHandler<UiCard>,
    on_create_click: EventHandler<()>,
) -> Element {
    if loading {
        rsx! {
            Card { class: Some("p-12 text-center".to_string()),
                LoadingState { message: Some("Загрузка карточек...".to_string()) }
            }
        }
    } else if cards.is_empty() {
        rsx! {
            Card { class: Some("p-12 text-center".to_string()),
                EmptyCardsState { on_create_click }
            }
        }
    } else {
        rsx! {
            Grid {
                columns: Some("grid-cols-1 md:grid-cols-2 lg:grid-cols-3".to_string()),
                gap: Some("gap-6".to_string()),
                for card in cards {
                    CardItem { card: card.clone(), on_edit, on_delete }
                }
            }
        }
    }
}

#[component]
fn EmptyCardsState(on_create_click: EventHandler<()>) -> Element {
    rsx! {
        EmptyState {
            icon: Some("🎯".to_string()),
            title: "Добро пожаловать в Keikaku!".to_string(),
            description: Some(
                "Начните свое путешествие по изучению японского языка. Создайте свою первую карточку и откройте для себя эффективный метод повторений."
                    .to_string(),
            ),
            action_text: Some("+ Создать первую карточку".to_string()),
            on_action: Some(on_create_click),
            additional_content: Some(rsx! {
                div { class: "text-xs text-slate-400",
                    "💡 Начните с 5-10 карточек для лучшего запоминания"
                }
            }),
        }
    }
}

#[component]
fn CardItem(
    card: UiCard,
    on_edit: EventHandler<UiCard>,
    on_delete: EventHandler<UiCard>,
) -> Element {
    use crate::ui::H2;

    let card_rc = Rc::new(card);

    rsx! {
        Card {
            class: Some(
                "p-6 hover:shadow-soft-hover hover:scale-[1.02] transition-all duration-300 cursor-pointer"
                    .to_string(),
            ),
            div { class: "space-y-4",
                H2 { class: Some("text-xl font-bold text-slate-800 leading-tight".to_string()),
                    {card_rc.question.clone()}
                }

                Paragraph { class: Some("text-sm text-slate-600 leading-relaxed".to_string()),
                    {card_rc.answer.clone()}
                }

                div { class: "flex items-center gap-2 flex-wrap",
                    Pill {
                        text: format!("Повтор: {}", card_rc.next_review),
                        tone: Some(if card_rc.due { StateTone::Warning } else { StateTone::Info }),
                    }
                    Pill {
                        text: if card_rc.due { "К повторению".to_string() } else { "Запланирована".to_string() },
                        tone: Some(if card_rc.due { StateTone::Warning } else { StateTone::Neutral }),
                    }
                }

                div { class: "flex gap-2 pt-2 border-t border-slate-100",
                    Button {
                        variant: ButtonVariant::Outline,
                        class: Some("flex-1 text-sm".to_string()),
                        onclick: {
                            let card_clone = Rc::clone(&card_rc);
                            move |_| on_edit.call((*card_clone).clone())
                        },
                        "Редактировать"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        class: Some(
                            "flex-1 text-sm text-red-600 border-red-200 hover:border-red-300 hover:text-red-700"
                                .to_string(),
                        ),
                        onclick: {
                            let card_clone = Rc::clone(&card_rc);
                            move |_| on_delete.call((*card_clone).clone())
                        },
                        "Удалить"
                    }
                }
            }
        }
    }
}
