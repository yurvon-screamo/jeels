use dioxus::prelude::*;
use dioxus_heroicons::{Icon, solid};
use keikaku::application::use_cases::delete_card::DeleteCardUseCase;
use keikaku::domain::VocabularyCard;
use ulid::Ulid;

use crate::ui::{Button, ButtonVariant, Card, Modal, Paragraph};
use crate::views::cards::notification::Notification;
use crate::{DEFAULT_USERNAME, ensure_user, to_error};
use keikaku::settings::ApplicationEnvironment;

#[component]
pub fn DeleteConfirmModal(
    card_id: Option<String>,
    on_close: EventHandler<()>,
    on_confirm: EventHandler<String>,
) -> Element {
    if let Some(card_id) = card_id {
        rsx! {
            Modal { title: "Удалить карточку", on_close,
                div { class: "space-y-4",
                    Card { class: Some("bg-red-50 border-red-200".to_string()),
                        div { class: "flex items-center gap-3",
                            Icon {
                                icon: solid::Shape::ExclamationTriangle,
                                size: 32,
                                class: Some("text-red-500".to_string()),
                            }
                            div { class: "flex-1",
                                div { class: "text-sm font-semibold text-red-800", "Внимание!" }
                                Paragraph { class: Some("text-red-700 text-sm".to_string()),
                                    "Удаление карточки необратимо. Вся история повторений будет потеряна."
                                }
                            }
                        }
                    }
                    Paragraph { class: Some("text-slate-600 text-sm".to_string()),
                        "Вы действительно хотите удалить эту карточку?"
                    }
                    div { class: "flex gap-2 justify-end",
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| on_close.call(()),
                            "Отмена"
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            class: Some(
                                "text-red-600 border-red-200 hover:border-red-300 hover:text-red-700".to_string(),
                            ),
                            onclick: move |_| {
                                on_confirm.call(card_id.clone());
                            },
                            "Удалить навсегда"
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

pub fn delete_card_with_handlers(
    notification: Signal<Notification>,
    delete_confirm: Signal<Option<String>>,
    loading: Signal<bool>,
    on_refresh: EventHandler<()>,
) -> impl Fn(String) {
    move |card_id: String| {
        let mut notification = notification;
        let mut delete_confirm = delete_confirm;
        let mut loading = loading;
        let on_refresh = on_refresh;

        spawn(async move {
            loading.set(true);
            match delete_card(card_id.clone()).await {
                Ok(_) => {
                    delete_confirm.set(None);
                    notification.set(Notification::Success("Карточка удалена".to_string()));
                    on_refresh.call(());
                }
                Err(e) => {
                    notification.set(Notification::Error(format!("Ошибка: {}", e)));
                }
            }
            loading.set(false);
        });
    }
}

async fn delete_card(card_id: String) -> Result<VocabularyCard, String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    let card_id_ulid = card_id.parse::<Ulid>().map_err(|e| e.to_string())?;

    DeleteCardUseCase::new(repo)
        .execute(user_id, card_id_ulid)
        .await
        .map_err(to_error)
}
