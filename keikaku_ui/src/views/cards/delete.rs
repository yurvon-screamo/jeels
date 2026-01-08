use dioxus::prelude::*;
use dioxus_heroicons::{Icon, solid};
use dioxus_primitives::toast::{ToastOptions, Toasts};
use keikaku::application::use_cases::delete_card::DeleteCardUseCase;
use ulid::Ulid;

use crate::components::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
};
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
            AlertDialogRoot {
                open: true,
                on_open_change: move |v: bool| {
                    if !v {
                        on_close.call(())
                    }
                },
                AlertDialogContent {
                    AlertDialogTitle { "Удалить карточку" }
                    AlertDialogDescription {
                        div { class: "flex items-start gap-3",
                            Icon {
                                icon: solid::Shape::ExclamationTriangle,
                                size: 20,
                                class: Some("text-destructive".to_string()),
                            }
                            div { class: "space-y-2",
                                div { class: "text-sm font-semibold", "Внимание!" }
                                div { class: "text-sm text-muted-foreground",
                                    "Удаление карточки необратимо. Вся история повторений будет потеряна."
                                }
                                div { class: "text-sm",
                                    "Вы действительно хотите удалить эту карточку?"
                                }
                            }
                        }
                    }
                    AlertDialogActions {
                        AlertDialogCancel { "Отмена" }
                        AlertDialogAction { on_click: move |_| on_confirm.call(card_id.clone()),
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
    toast: Toasts,
    delete_confirm: Signal<Option<String>>,
    loading: Signal<bool>,
    on_refresh: EventHandler<()>,
) -> impl Fn(String) {
    move |card_id: String| {
        let mut delete_confirm = delete_confirm;
        let mut loading = loading;
        let on_refresh = on_refresh;

        spawn(async move {
            loading.set(true);
            match delete_card(card_id.clone()).await {
                Ok(_) => {
                    delete_confirm.set(None);
                    toast.success("Карточка удалена".to_string(), ToastOptions::new());
                    on_refresh.call(());
                }
                Err(e) => {
                    toast.error(format!("Ошибка: {}", e), ToastOptions::new());
                }
            }
            loading.set(false);
        });
    }
}

async fn delete_card(card_id: String) -> Result<(), String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    let card_id_ulid = card_id.parse::<Ulid>().map_err(|e| e.to_string())?;

    DeleteCardUseCase::new(repo)
        .execute(user_id, card_id_ulid)
        .await
        .map_err(to_error)
}
