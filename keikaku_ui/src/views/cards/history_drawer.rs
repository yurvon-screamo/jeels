use dioxus::prelude::*;

use crate::{
    components::button::{Button, ButtonVariant},
    components::sheet::{Sheet, SheetContent, SheetFooter, SheetHeader, SheetSide, SheetTitle},
    views::cards::UiCard,
};
use keikaku::domain::value_objects::Rating;

#[component]
pub fn CardHistoryDrawer(
    card: Option<UiCard>,
    open: bool,
    on_open_change: EventHandler<bool>,
) -> Element {
    rsx! {
        Sheet { open, on_open_change,
            SheetContent { side: SheetSide::Right, class: "w-[60%]",
                SheetHeader {
                    SheetTitle { "История рейтингов карточки" }
                }

                if let Some(card) = card {
                    div { class: "space-y-6",
                        // Информация о карточке
                        div { class: "space-y-2",
                            h3 { class: "text-lg font-semibold text-slate-800",
                                {card.question.clone()}
                            }
                            p { class: "text-sm text-slate-600", {card.answer.clone()} }
                        }

                        // История рейтингов
                        if card.reviews.is_empty() {
                            div { class: "text-center py-8 text-slate-500",
                                "История рейтингов пуста"
                            }
                        } else {
                            div { class: "space-y-4",
                                h4 { class: "font-medium text-slate-700",
                                    "История повторений"
                                }

                                // Таблица с историей
                                div { class: "border rounded-lg overflow-hidden",
                                    table { class: "w-full text-sm",
                                        thead { class: "bg-slate-50",
                                            tr {
                                                th { class: "px-4 py-2 text-left font-medium text-slate-700",
                                                    "Дата"
                                                }
                                                th { class: "px-4 py-2 text-left font-medium text-slate-700",
                                                    "Рейтинг"
                                                }
                                                th { class: "px-4 py-2 text-left font-medium text-slate-700",
                                                    "Интервал"
                                                }
                                                th { class: "px-4 py-2 text-left font-medium text-slate-700",
                                                    "Стабильность"
                                                }
                                                th { class: "px-4 py-2 text-left font-medium text-slate-700",
                                                    "Сложность"
                                                }
                                            }
                                        }
                                        tbody {
                                            for review in card.reviews.iter().rev() {
                                                tr { class: "border-t",
                                                    td { class: "px-4 py-2 text-slate-600",
                                                        {review.timestamp.format("%d.%m.%Y %H:%M").to_string()}
                                                    }
                                                    td { class: "px-4 py-2",
                                                        span { class: get_rating_badge_class(review.rating),
                                                            {format_rating(review.rating)}
                                                        }
                                                    }
                                                    td { class: "px-4 py-2 text-slate-600",
                                                        {format_duration(review.interval)}
                                                    }
                                                    td { class: "px-4 py-2 text-slate-600",
                                                        if let Some(stability) = card.stability {
                                                            {format!("{:.1}", stability)}
                                                        } else {
                                                            "-"
                                                        }
                                                    }
                                                    td { class: "px-4 py-2 text-slate-600",
                                                        if let Some(difficulty) = card.difficulty {
                                                            {format!("{:.1}", difficulty)}
                                                        } else {
                                                            "-"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                SheetFooter {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| on_open_change.call(false),
                        "Закрыть"
                    }
                }
            }
        }
    }
}

fn format_rating(rating: Rating) -> String {
    match rating {
        Rating::Easy => "Легко",
        Rating::Good => "Хорошо",
        Rating::Hard => "Трудно",
        Rating::Again => "Заново",
    }
    .to_string()
}

fn get_rating_badge_class(rating: Rating) -> String {
    match rating {
        Rating::Easy => "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800",
        Rating::Good => "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800",
        Rating::Hard => "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800",
        Rating::Again => "inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-red-100 text-red-800",
    }.to_string()
}

fn format_duration(interval: chrono::Duration) -> String {
    let days = interval.num_days();
    let hours = interval.num_hours() % 24;
    let minutes = interval.num_minutes() % 60;

    if days > 0 {
        format!("{}д {}ч", days, hours)
    } else if hours > 0 {
        format!("{}ч {}м", hours, minutes)
    } else {
        format!("{}м", minutes)
    }
}
