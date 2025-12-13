use dioxus::prelude::*;

use crate::domain::{CardFilters, CardHeader, CardsList, CardStats, FilterStatus, SortBy, UiCard};
use crate::ui::{Modal, NotificationBanner, NotificationType};

#[derive(Clone, PartialEq)]
enum ModalState {
    None,
    Create,
    Edit { card_id: String },
}

#[derive(Clone, PartialEq)]
enum Notification {
    None,
    Success(String),
    Error(String),
}

#[component]
pub fn Cards() -> Element {
    let cards = use_signal(Vec::<UiCard>::new);
    let search = use_signal(String::new);
    let filter_status = use_signal(|| FilterStatus::All);
    let sort_by = use_signal(|| SortBy::Date);
    let mut modal_state = use_signal(|| ModalState::None);
    let mut notification = use_signal(|| Notification::None);

    let filtered_and_sorted = move || {
        let q = search().to_lowercase();
        let mut result: Vec<UiCard> = cards()
            .into_iter()
            .filter(|c| {
                let matches_search = q.is_empty()
                    || c.question.to_lowercase().contains(&q)
                    || c.answer.to_lowercase().contains(&q);

                let matches_status = match filter_status() {
                    FilterStatus::All => true,
                    FilterStatus::Due => c.due,
                    FilterStatus::NotDue => !c.due,
                };

                matches_search && matches_status
            })
            .collect::<Vec<_>>();

        match sort_by() {
            SortBy::Date => {
                result.sort_by(|a, b| {
                    if a.due && !b.due {
                        std::cmp::Ordering::Less
                    } else if !a.due && b.due {
                        std::cmp::Ordering::Greater
                    } else {
                        a.next_review.cmp(&b.next_review)
                    }
                });
            }
            SortBy::Question => {
                result.sort_by(|a, b| a.question.cmp(&b.question));
            }
            SortBy::Answer => {
                result.sort_by(|a, b| a.answer.cmp(&b.answer));
            }
        }

        result
    };

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            NotificationArea {
                notification,
                on_close: move |_| notification.set(Notification::None),
            }

            CardHeader {
                total_count: cards().len(),
                due_count: cards().iter().filter(|c| c.due).count(),
            }

            CardStats {
                total_count: cards().len(),
                due_count: cards().iter().filter(|c| c.due).count(),
                filtered_count: filtered_and_sorted().len(),
            }

            CardFilters { search, filter_status, sort_by }

            CardsList {
                cards: filtered_and_sorted(),
                loading: false,
                on_edit: move |card_id| modal_state.set(ModalState::Edit { card_id }),
                on_delete: move |card_id| {},
            }

            // Модалки будут добавлены позже
        }
    }
}

#[component]
fn NotificationArea(notification: Signal<Notification>, on_close: EventHandler<()>) -> Element {
    match notification() {
        Notification::Success(msg) => rsx! {
            NotificationBanner {
                message: msg,
                notification_type: NotificationType::Success,
                on_close,
            }
        },
        Notification::Error(msg) => rsx! {
            NotificationBanner {
                message: msg,
                notification_type: NotificationType::Error,
                on_close,
            }
        },
        Notification::None => rsx! {},
    }
}
