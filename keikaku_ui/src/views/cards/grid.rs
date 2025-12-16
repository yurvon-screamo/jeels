use dioxus::prelude::*;
use std::rc::Rc;

use crate::{
    ui::{Card, EmptyState, Grid, IconButton, LoadingState, Paragraph, Pill, StateTone, H3},
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
                columns: Some("grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4".to_string()),
                gap: Some("gap-4".to_string()),
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
    let card_rc = Rc::new(card);

    rsx! {
        Card {
            class: Some(
                "p-4 hover:shadow-soft-hover hover:scale-[1.01] transition-all duration-200 cursor-pointer relative"
                    .to_string(),
            ),
            // Иконочные кнопки в правом верхнем углу
            div { class: "absolute top-2 right-2 flex gap-1",
                IconButton {
                    icon: rsx! {
                        svg { fill: "currentColor", view_box: "0 0 4 4",
                            path { d: "M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" }
                        }
                    },
                    onclick: {
                        let card_clone = Rc::clone(&card_rc);
                        move |_| on_edit.call((*card_clone).clone())
                    },
                    class: Some("w-8 h-8 bg-blue-500 hover:bg-blue-600".to_string()),
                }
                IconButton {
                    icon: rsx! {
                        svg { fill: "currentColor", view_box: "0 0 4 4",
                            path { d: "M9 2a1 1 0 000 2h2a1 1 0 100-2H9z" }
                            path {
                                "fill-rule": "evenodd",
                                d: "M10 5a3 3 0 100 6h.007a.75.75 0 01.75.75V12a1 1 0 01-1 1H9a1 1 0 01-1-1v-1.25a.75.75 0 01.75-.75H10a1.5 1.5 0 000-3H9a.75.75 0 010-1.5h1zm-.75 7.5a.75.75 0 000 1.5h1.5a.75.75 0 000-1.5h-1.5z",
                                clip_rule: "evenodd",
                            }
                        }
                    },
                    onclick: {
                        let card_clone = Rc::clone(&card_rc);
                        move |_| on_delete.call((*card_clone).clone())
                    },
                    class: Some("w-8 h-8 bg-red-500 hover:bg-red-600".to_string()),
                }
            }

            div { class: "space-y-3 pr-16",
                H3 { class: Some("text-lg font-bold text-slate-800 leading-tight".to_string()),
                    {card_rc.question.clone()}
                }

                Paragraph { class: Some("text-sm text-slate-600 leading-relaxed".to_string()),
                    {card_rc.answer.clone()}
                }

                // Отображение примеров
                if !card_rc.examples.is_empty() {
                    div { class: "space-y-1",
                        for (text , translation) in card_rc.examples.iter().take(2) {
                            div { class: "text-xs",
                                span { class: "text-slate-700 font-medium", "{text}" }
                                span { class: "text-slate-500 ml-1", "— {translation}" }
                            }
                        }
                    }
                }

                // Теги статуса, сложности и стабильности
                div { class: "flex items-center gap-2 flex-wrap",
                    Pill {
                        text: if card_rc.is_new { "Новая".to_string() } else if card_rc.is_low_stability { "Низкая стабильность".to_string() } else if card_rc.is_learned { "Изучено".to_string() } else if card_rc.is_in_progress { "В процессе".to_string() } else { "???".to_string() },
                        tone: Some(
                            if card_rc.is_new {
                                StateTone::Info
                            } else if card_rc.is_low_stability {
                                StateTone::Warning
                            } else if card_rc.is_learned {
                                StateTone::Success
                            } else if card_rc.due {
                                StateTone::Warning
                            } else {
                                StateTone::Neutral
                            },
                        ),
                    }

                    // Тег сложности
                    if let Some(difficulty) = card_rc.difficulty {
                        Pill {
                            text: format!("Сложность: {:.1}", difficulty),
                            tone: Some(StateTone::Neutral),
                        }
                    }

                    // Тег стабильности
                    if let Some(stability) = card_rc.stability {
                        Pill {
                            text: format!("Стабильность: {:.1}", stability),
                            tone: Some(StateTone::Neutral),
                        }
                    }

                    if !card_rc.is_new {
                        Pill {
                            text: format!("Повтор: {}", card_rc.next_review),
                            tone: Some(if card_rc.due { StateTone::Warning } else { StateTone::Info }),
                        }
                    }
                }
            }
        }
    }
}
