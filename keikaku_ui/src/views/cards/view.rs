use dioxus::prelude::*;
use keikaku::application::use_cases::{
    create_card::CreateCardUseCase, delete_card::DeleteCardUseCase, edit_card::EditCardUseCase,
    list_cards::ListCardsUseCase,
};
use keikaku::domain::{value_objects::CardContent, VocabularyCard};
use ulid::Ulid;

use crate::domain::{CardFilters, CardHeader, CardStats, CardsList, FilterStatus, SortBy, UiCard};
use crate::ui::{Button, ButtonVariant, Modal, NotificationBanner, NotificationType, TextInput};
use crate::{ensure_user, init_env, to_error, DEFAULT_USERNAME};

use super::use_cases::map_card;

async fn fetch_cards() -> Result<Vec<VocabularyCard>, String> {
    let env = init_env().await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    ListCardsUseCase::new(repo)
        .execute(user_id)
        .await
        .map_err(to_error)
}

async fn create_card(question: String, answer: String) -> Result<VocabularyCard, String> {
    let env = init_env().await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let llm_service = env.get_llm_service().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    let card_content = CardContent::new(
        keikaku::domain::value_objects::Answer::new(answer).map_err(to_error)?,
        Vec::new(),
    );

    CreateCardUseCase::new(repo, llm_service)
        .execute(user_id, question, Some(card_content))
        .await
        .map_err(to_error)
}

async fn edit_card(
    card_id: String,
    question: String,
    answer: String,
) -> Result<VocabularyCard, String> {
    let env = init_env().await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    let card_id_ulid = card_id.parse::<Ulid>().map_err(|e| e.to_string())?;

    EditCardUseCase::new(repo)
        .execute(user_id, card_id_ulid, question, answer, Vec::new())
        .await
        .map_err(to_error)
}

async fn delete_card(card_id: String) -> Result<VocabularyCard, String> {
    let env = init_env().await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    let card_id_ulid = card_id.parse::<Ulid>().map_err(|e| e.to_string())?;

    DeleteCardUseCase::new(repo)
        .execute(user_id, card_id_ulid)
        .await
        .map_err(to_error)
}

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
    let mut cards = use_signal(Vec::<UiCard>::new);
    let search = use_signal(String::new);
    let filter_status = use_signal(|| FilterStatus::All);
    let sort_by = use_signal(|| SortBy::Date);
    let mut modal_state = use_signal(|| ModalState::None);
    let mut notification = use_signal(|| Notification::None);
    let mut delete_confirm = use_signal(|| None::<String>);
    let loading = use_signal(|| false);

    let cards_resource = use_resource(fetch_cards);

    use_effect(move || {
        if let Some(Ok(remote)) = cards_resource.read().as_ref() {
            let mapped = remote.iter().map(map_card).collect::<Vec<_>>();
            cards.set(mapped);
        }
    });

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
                loading: cards_resource.read().is_none()
                    || cards_resource.read().as_ref().map(|r| r.is_err()).unwrap_or(false),
                on_edit: move |card_id| modal_state.set(ModalState::Edit { card_id }),
                on_delete: move |card_id: String| delete_confirm.set(Some(card_id)),
            }

            CreateEditModal {
                modal_state,
                on_close: move || modal_state.set(ModalState::None),
                on_success: move |msg| notification.set(Notification::Success(msg)),
                on_error: move |msg| notification.set(Notification::Error(msg)),
                loading,
            }

            DeleteConfirmModal {
                card_id: delete_confirm(),
                on_close: move || delete_confirm.set(None),
                on_confirm: move |card_id: String| {
                    let mut cards = cards;
                    let mut notification = notification;
                    let mut delete_confirm = delete_confirm;
                    let mut loading = loading;

                    spawn(async move {
                        loading.set(true);
                        match delete_card(card_id.clone()).await {
                            Ok(_) => {
                                cards.write().retain(|c| c.id != card_id);
                                delete_confirm.set(None);
                                notification
                                    .set(
                                        Notification::Success(
                                            "Карточка удалена".to_string(),
                                        ),
                                    );
                            }
                            Err(e) => {
                                notification
                                    .set(Notification::Error(format!("Ошибка: {}", e)));
                            }
                        }
                        loading.set(false);
                    });
                },
            }
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

#[component]
fn CreateEditModal(
    modal_state: Signal<ModalState>,
    on_close: EventHandler<()>,
    on_success: EventHandler<String>,
    on_error: EventHandler<String>,
    loading: Signal<bool>,
) -> Element {
    let mut question = use_signal(String::new);
    let mut answer = use_signal(String::new);

    match modal_state() {
        ModalState::Create => {
            rsx! {
                Modal { title: "Создать карточку", on_close,
                    div { class: "space-y-4",
                        TextInput {
                            label: "Вопрос",
                            value: question,
                            placeholder: "Введите вопрос...",
                        }
                        TextInput {
                            label: "Ответ",
                            value: answer,
                            placeholder: "Введите ответ...",
                        }
                        div { class: "flex gap-2 justify-end",
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| on_close.call(()),
                                "Отмена"
                            }
                            Button {
                                variant: ButtonVariant::Rainbow,
                                onclick: move |_| {
                                    let q = question();
                                    let a = answer();
                                    if q.trim().is_empty() || a.trim().is_empty() {
                                        return;
                                    }

                                    let on_success = on_success;
                                    let on_error = on_error;
                                    let mut modal_state = modal_state;
                                    let mut loading = loading;

                                    spawn(async move {
                                        loading.set(true);
                                        match create_card(q, a).await {
                                            Ok(_) => {
                                                question.set(String::new());
                                                answer.set(String::new());
                                                modal_state.set(ModalState::None);
                                                on_success.call("Карточка создана".to_string());
                                            }
                                            Err(e) => {
                                                on_error.call(format!("Ошибка: {}", e));
                                            }
                                        }
                                        loading.set(false);
                                    });
                                },
                                disabled: Some(loading()),
                                "Создать"
                            }
                        }
                    }
                }
            }
        }
        ModalState::Edit { card_id } => {
            rsx! {
                Modal {
                    title: "Редактировать карточку",
                    on_close,
                    div { class: "space-y-4",
                        TextInput {
                            label: "Вопрос",
                            value: question,
                            placeholder: "Введите вопрос...",
                        }
                        TextInput {
                            label: "Ответ",
                            value: answer,
                            placeholder: "Введите ответ...",
                        }
                        div { class: "flex gap-2 justify-end",
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| on_close.call(()),
                                "Отмена"
                            }
                            Button {
                                variant: ButtonVariant::Rainbow,
                                onclick: move |_| {
                                    let q = question();
                                    let a = answer();
                                    if q.trim().is_empty() || a.trim().is_empty() {
                                        return;
                                    }

                                    let card_id = card_id.clone();
                                    let on_success = on_success;
                                    let on_error = on_error;
                                    let mut modal_state = modal_state;
                                    let mut loading = loading;

                                    spawn(async move {
                                        loading.set(true);
                                        match edit_card(card_id, q, a).await {
                                            Ok(_) => {
                                                question.set(String::new());
                                                answer.set(String::new());
                                                modal_state.set(ModalState::None);
                                                on_success.call("Карточка обновлена".to_string());
                                            }
                                            Err(e) => {
                                                on_error.call(format!("Ошибка: {}", e));
                                            }
                                        }
                                        loading.set(false);
                                    });
                                },
                                disabled: Some(loading()),
                                "Сохранить"
                            }
                        }
                    }
                }
            }
        }
        ModalState::None => rsx! {},
    }
}

#[component]
fn DeleteConfirmModal(
    card_id: Option<String>,
    on_close: EventHandler<()>,
    on_confirm: EventHandler<String>,
) -> Element {
    if let Some(card_id) = card_id {
        rsx! {
            Modal { title: "Удалить карточку", on_close,
                div { class: "space-y-4",
                    p {
                        "Вы действительно хотите удалить эту карточку?"
                    }
                    div { class: "flex gap-2 justify-end",
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| on_close.call(()),
                            "Отмена"
                        }
                        Button {
                            variant: ButtonVariant::Rainbow,
                            onclick: move |_| {
                                on_confirm.call(card_id.clone());
                            },
                            "Удалить"
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}
