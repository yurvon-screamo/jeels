use dioxus::prelude::*;
use keikaku::application::use_cases::list_cards::ListCardsUseCase;
use keikaku::domain::VocabularyCard;
use keikaku::settings::ApplicationEnvironment;

use crate::ui::ErrorCard;
use crate::views::cards::create::CreateModal;
use crate::views::cards::delete::{delete_card_with_handlers, DeleteConfirmModal};
use crate::views::cards::edit::EditModal;
use crate::views::cards::notification::{Notification, NotificationArea};
use crate::{
    ensure_user, to_error,
    views::cards::{
        filters::CardsFilters, grid::CardsGrid, header::CardsHeader, stats::CardsStats,
        FilterStatus, SortBy, UiCard,
    },
    DEFAULT_USERNAME,
};

#[derive(Clone, PartialEq)]
pub enum ModalState {
    None,
    Create,
    Edit {
        card_id: String,
        question: String,
        answer: String,
    },
}

#[component]
pub fn Cards() -> Element {
    let mut cards_resource = use_resource(fetch_cards);

    // Read resources once and store results
    let cards_read = cards_resource.read();

    match cards_read.as_ref() {
        Some(Ok(cards)) => {
            let mapped_cards = cards.iter().map(map_card).collect::<Vec<_>>();
            let processed_data = process_cards_data(mapped_cards);

            rsx! {
                CardsContent {
                    cards_data: processed_data.clone(),
                    on_refresh: move || cards_resource.restart(),
                }
            }
        }
        Some(Err(err)) => rsx! {
            ErrorCard { message: format!("Ошибка загрузки карточек: {}", err) }
        },
        None => rsx! {
            div { class: "bg-bg min-h-screen text-text-main px-6 py-8", "Загрузка..." }
        },
    }
}

#[derive(Clone, PartialEq)]
struct ProcessedCardsData {
    pub cards: Vec<UiCard>,
    pub stats: CardsStatsData,
}

#[derive(Clone, PartialEq)]
struct CardsStatsData {
    pub total_count: usize,
    pub due_count: usize,
    pub filtered_count: usize,
}

#[component]
fn CardsContent(cards_data: ProcessedCardsData, on_refresh: EventHandler<()>) -> Element {
    let search = use_signal(String::new);
    let filter_status = use_signal(|| FilterStatus::All);
    let sort_by = use_signal(|| SortBy::Date);
    let mut modal_state = use_signal(|| ModalState::None);
    let mut notification = use_signal(|| Notification::None);
    let mut delete_confirm = use_signal(|| None::<String>);
    let loading = use_signal(|| false);

    let filtered_and_sorted = move || {
        filter_and_sort_cards(
            cards_data.cards.clone(),
            search(),
            filter_status(),
            sort_by(),
        )
    };

    rsx! {
        div { class: "bg-bg min-h-screen text-text-main px-6 py-8 space-y-6",
            NotificationArea {
                notification,
                on_close: move |_| notification.set(Notification::None),
            }

            CardsHeader {
                total_count: cards_data.stats.total_count,
                due_count: cards_data.stats.due_count,
                on_create_click: move |_| modal_state.set(ModalState::Create),
            }

            CardsStats {
                total_count: cards_data.stats.total_count,
                due_count: cards_data.stats.due_count,
                filtered_count: filtered_and_sorted().len(),
            }

            CardsFilters { search, filter_status, sort_by }

            CardsGrid {
                cards: filtered_and_sorted(),
                loading: loading(),
                on_edit: move |card: UiCard| {
                    modal_state
                        .set(ModalState::Edit {
                            card_id: card.id,
                            question: card.question,
                            answer: card.answer,
                        })
                },
                on_delete: move |card: UiCard| delete_confirm.set(Some(card.id)),
                on_create_click: move |_| modal_state.set(ModalState::Create),
            }

            match modal_state() {
                ModalState::Create => rsx! {
                    CreateModal {
                        on_close: move |_| modal_state.set(ModalState::None),
                        on_success: move |msg| {
                            notification.set(Notification::Success(msg));
                            on_refresh.call(());
                        },
                        on_error: move |msg| notification.set(Notification::Error(msg)),
                        loading: loading(),
                    }
                },
                ModalState::Edit { card_id, question, answer } => rsx! {
                    EditModal {
                        card_id,
                        initial_question: question,
                        initial_answer: answer,
                        on_close: move |_| modal_state.set(ModalState::None),
                        on_success: move |msg| {
                            notification.set(Notification::Success(msg));
                            on_refresh.call(());
                        },
                        on_error: move |msg| notification.set(Notification::Error(msg)),
                        loading: loading(),
                    }
                },
                ModalState::None => rsx! {},
            }

            DeleteConfirmModal {
                card_id: delete_confirm(),
                on_close: move || delete_confirm.set(None),
                on_confirm: delete_card_with_handlers(notification, delete_confirm, loading, on_refresh),
            }
        }
    }
}

fn map_card(card: &VocabularyCard) -> UiCard {
    let next_review = card
        .memory()
        .next_review_date()
        .map(|d| {
            let now = chrono::Utc::now();
            let diff = (*d - now).num_days();

            if diff < 0 {
                "Просрочено".to_string()
            } else if diff == 0 {
                "Сегодня".to_string()
            } else if diff == 1 {
                "Завтра".to_string()
            } else if diff < 7 {
                format!("Через {} дн.", diff)
            } else {
                d.format("%d.%m.%Y").to_string()
            }
        })
        .unwrap_or_else(|| "—".to_string());

    UiCard {
        id: card.id().to_string(),
        question: card.word().text().to_string(),
        answer: card.meaning().text().to_string(),
        next_review,
        due: card.memory().is_due(),
    }
}

fn process_cards_data(cards: Vec<UiCard>) -> ProcessedCardsData {
    let total_count = cards.len();
    let due_count = cards.iter().filter(|c| c.due).count();

    ProcessedCardsData {
        cards,
        stats: CardsStatsData {
            total_count,
            due_count,
            filtered_count: total_count, // Will be updated after filtering
        },
    }
}

fn filter_and_sort_cards(
    cards: Vec<UiCard>,
    search: String,
    filter_status: FilterStatus,
    sort_by: SortBy,
) -> Vec<UiCard> {
    let q = search.to_lowercase();
    let mut result: Vec<UiCard> = cards
        .into_iter()
        .filter(|c| {
            let matches_search = q.is_empty()
                || c.question.to_lowercase().contains(&q)
                || c.answer.to_lowercase().contains(&q);

            let matches_status = match filter_status {
                FilterStatus::All => true,
                FilterStatus::Due => c.due,
                FilterStatus::NotDue => !c.due,
            };

            matches_search && matches_status
        })
        .collect::<Vec<_>>();

    match sort_by {
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
}

async fn fetch_cards() -> Result<Vec<VocabularyCard>, String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    ListCardsUseCase::new(repo)
        .execute(user_id)
        .await
        .map_err(to_error)
}
