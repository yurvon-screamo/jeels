use dioxus::prelude::*;
use keikaku::application::use_cases::knowledge_set_cards::KnowledgeSetCardsUseCase;
use keikaku::domain::knowledge::StudyCard;
use keikaku::domain::value_objects::{Difficulty, Stability};
use keikaku::settings::ApplicationEnvironment;

use crate::components::app_ui::ErrorCard;
use crate::views::cards::create::CreateModal;
use crate::views::cards::delete::{DeleteConfirmModal, delete_card_with_handlers};
use crate::{
    DEFAULT_USERNAME, ensure_user, to_error,
    views::cards::{
        FilterStatus, SortBy, UiCard, filters::CardsFilters, grid::CardsGrid, header::CardsHeader,
        history_drawer::CardHistoryDrawer, stats::CardsStats, types::ReviewInfo,
    },
};
use dioxus_primitives::toast::{ToastOptions, use_toast};

#[derive(Clone, PartialEq)]
pub enum ModalState {
    None,
    Create,
}

#[component]
pub fn Cards() -> Element {
    let mut cards_resource = use_resource(fetch_cards);

    // Read resources once and store results
    let cards_read = cards_resource.read();

    match cards_read.as_ref() {
        Some(Ok(cards)) => {
            let mapped_cards = cards
                .iter()
                .map(|card: &StudyCard| map_card(card))
                .collect::<Vec<_>>();
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
    let mut delete_confirm = use_signal(|| None::<String>);
    let mut selected_card_for_history = use_signal(|| None::<UiCard>);
    let loading = use_signal(|| false);
    let toast = use_toast();

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
                on_edit: move |_card: UiCard| {},
                on_delete: move |card: UiCard| delete_confirm.set(Some(card.id)),
                on_create_click: move |_| modal_state.set(ModalState::Create),
                on_card_click: move |card: UiCard| selected_card_for_history.set(Some(card)),
            }

            match modal_state() {
                ModalState::Create => rsx! {
                    CreateModal {
                        on_close: move |_| modal_state.set(ModalState::None),
                        on_success: move |msg| {
                            toast.success(msg, ToastOptions::new());
                            on_refresh.call(());
                        },
                        on_error: move |msg| toast.error(msg, ToastOptions::new()),
                        loading: loading(),
                    }
                },
                ModalState::None => rsx! {},
            }

            DeleteConfirmModal {
                card_id: delete_confirm(),
                on_close: move || delete_confirm.set(None),
                on_confirm: delete_card_with_handlers(toast, delete_confirm, loading, on_refresh),
            }

            CardHistoryDrawer {
                card: selected_card_for_history(),
                open: selected_card_for_history().is_some(),
                on_open_change: move |open: bool| {
                    if !open {
                        selected_card_for_history.set(None);
                    }
                },
            }
        }
    }
}

fn map_card(card: &StudyCard) -> UiCard {
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

    // Extract data based on card type
    let (question, answer, examples) = match card.card() {
        keikaku::domain::knowledge::Card::Vocabulary(v) => (
            v.word().text().to_string(),
            v.meaning().text().to_string(),
            v.example_phrases()
                .iter()
                .map(|ex: &keikaku::domain::value_objects::ExamplePhrase| {
                    (ex.text().clone(), ex.translation().clone())
                })
                .collect(),
        ),
        keikaku::domain::knowledge::Card::Kanji(k) => (
            k.kanji().text().to_string(),
            k.description().text().to_string(),
            Vec::new(), // Kanji cards don't have examples in the same format
        ),
        keikaku::domain::knowledge::Card::Grammar(g) => (
            g.title().text().to_string(),
            g.description().text().to_string(),
            Vec::new(), // Grammar cards don't have examples in the same format
        ),
    };

    let reviews = card
        .memory()
        .reviews()
        .iter()
        .map(|review: &keikaku::domain::review::Review| ReviewInfo {
            timestamp: review.timestamp(),
            rating: review.rating(),
            interval: review.interval(),
        })
        .collect();

    UiCard {
        id: card.card_id().to_string(),
        question,
        answer,
        examples,
        difficulty: card.memory().difficulty().map(|d: &Difficulty| d.value()),
        stability: card.memory().stability().map(|s: &Stability| s.value()),
        next_review,
        due: card.memory().is_due(),
        is_new: card.memory().is_new(),
        is_in_progress: card.memory().is_in_progress(),
        is_learned: card.memory().is_known_card(),
        is_low_stability: card.memory().is_low_stability(),
        is_high_difficulty: card.memory().is_high_difficulty(),
        reviews,
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
                FilterStatus::New => c.is_new,
                FilterStatus::LowStability => c.is_low_stability,
                FilterStatus::HighDifficulty => c.is_high_difficulty,
                FilterStatus::InProgress => c.is_in_progress,
                FilterStatus::Learned => c.is_learned,
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
        SortBy::Difficulty => {
            result.sort_by(|a, b| match (a.difficulty, b.difficulty) {
                (Some(da), Some(db)) => da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            });
        }
        SortBy::Stability => {
            result.sort_by(|a, b| match (a.stability, b.stability) {
                (Some(sa), Some(sb)) => sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            });
        }
    }

    result
}

async fn fetch_cards() -> Result<Vec<StudyCard>, String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    KnowledgeSetCardsUseCase::new(repo)
        .execute(user_id)
        .await
        .map_err(to_error)
}
