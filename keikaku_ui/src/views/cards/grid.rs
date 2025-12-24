use dioxus::prelude::*;
use dioxus_heroicons::{IconButton, solid};
use std::rc::Rc;

use crate::{
    components::app_ui::{Card, EmptyState, Grid, H3, LoadingState, Paragraph, Pill, StateTone},
    domain::FuriganaText,
    views::cards::UiCard,
};

#[component]
pub fn CardsGrid(
    cards: Vec<UiCard>,
    loading: bool,
    on_edit: EventHandler<UiCard>,
    on_delete: EventHandler<UiCard>,
    on_create_click: EventHandler<()>,
    on_card_click: Option<EventHandler<UiCard>>,
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
                    CardItem {
                        card: card.clone(),
                        on_edit,
                        on_delete,
                        on_card_click,
                    }
                }
            }
        }
    }
}

#[component]
fn EmptyCardsState(on_create_click: EventHandler<()>) -> Element {
    rsx! {
        EmptyState {
            icon: Some(rsx! {
                dioxus_heroicons::Icon {
                    icon: solid::Shape::Flag,
                    size: 56,
                    class: Some("text-accent-pink".to_string()),
                }
            }),
            title: "Добро пожаловать в Keikaku!".to_string(),
            description: Some(
                "Начните свое путешествие по изучению японского языка. Создайте свою первую карточку и откройте для себя эффективный метод повторений."
                    .to_string(),
            ),
            action_text: Some("+ Создать первую карточку".to_string()),
            on_action: Some(on_create_click),
            additional_content: None,
        }
    }
}

#[component]
fn CardItem(
    card: UiCard,
    on_edit: EventHandler<UiCard>,
    on_delete: EventHandler<UiCard>,
    on_card_click: Option<EventHandler<UiCard>>,
) -> Element {
    let card_rc = Rc::new(card);

    rsx! {
        div {
            class: "cursor-pointer",
            onclick: {
                let card_clone = Rc::clone(&card_rc);
                move |_| {
                    if let Some(handler) = on_card_click {
                        handler.call((*card_clone).clone());
                    }
                }
            },
            Card {
                class: Some(
                    "p-4 hover:shadow-soft-hover hover:scale-[1.01] transition-all duration-200 relative"
                        .to_string(),
                ),
                // Иконочные кнопки в правом верхнем углу
                div { class: "absolute top-2 right-2 flex gap-1",
                    IconButton {
                        icon: solid::Shape::Pencil,
                        onclick: {
                            let card_clone = Rc::clone(&card_rc);
                            move |_| on_edit.call((*card_clone).clone())
                        },
                        class: "w-8 h-8 rounded-xl bg-cyan-500 hover:bg-cyan-600 text-white flex items-center justify-center shadow-md shadow-cyan-500/15 hover:scale-110 hover:shadow-glow active:scale-95 transition-all duration-300 ease-elastic",
                        title: "Редактировать",
                        size: 16,
                    }
                    IconButton {
                        icon: solid::Shape::Trash,
                        onclick: {
                            let card_clone = Rc::clone(&card_rc);
                            move |_| on_delete.call((*card_clone).clone())
                        },
                        class: "w-8 h-8 rounded-xl bg-amber-500 hover:bg-amber-600 text-white flex items-center justify-center shadow-md shadow-amber-500/15 hover:scale-110 hover:shadow-glow active:scale-95 transition-all duration-300 ease-elastic",
                        title: "Удалить",
                        size: 16,
                    }
                }

                div { class: "space-y-3 pr-16",
                    H3 { class: Some("text-lg font-bold text-slate-800 leading-tight".to_string()),
                        FuriganaText {
                            text: card_rc.question.clone(),
                            show_furigana: true,
                            class: None,
                        }
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
                    div { class: "grid grid-cols-2 gap-1 text-xs",
                        Pill {
                            text: if card_rc.is_new { "Новая".to_string() } else if card_rc.is_low_stability { "Низкая стаб.".to_string() } else if card_rc.is_high_difficulty { "Высокая сложн.".to_string() } else if card_rc.is_learned { "Изучено".to_string() } else if card_rc.is_in_progress { "В процессе".to_string() } else { "???".to_string() },
                            tone: Some(
                                if card_rc.is_new {
                                    StateTone::Info
                                } else if card_rc.is_low_stability {
                                    StateTone::Warning
                                } else if card_rc.is_high_difficulty {
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
}
