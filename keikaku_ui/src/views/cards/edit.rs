use dioxus::prelude::*;

use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use crate::components::sheet::{
    Sheet, SheetContent, SheetFooter, SheetHeader, SheetSide, SheetTitle,
};
use keikaku::settings::ApplicationEnvironment;

#[component]
pub fn EditModal(
    card_id: String,
    initial_question: String,
    initial_answer: String,
    on_close: EventHandler<()>,
    on_success: EventHandler<String>,
    on_error: EventHandler<String>,
    loading: bool,
) -> Element {
    let question = use_signal(|| initial_question.clone());
    let answer = use_signal(|| initial_answer.clone());

    rsx! {
        Sheet {
            open: true,
            on_open_change: move |v: bool| {
                if !v {
                    on_close.call(())
                }
            },
            SheetContent { side: SheetSide::Right,
                SheetHeader {
                    SheetTitle { "Редактировать карточку" }
                }

                div { class: "space-y-4",
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Вопрос" }
                        Input {
                            placeholder: "Введите вопрос...",
                            value: question(),
                            oninput: {
                                let mut question = question;
                                move |e: FormEvent| question.set(e.value())
                            },
                        }
                    }
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Ответ" }
                        Input {
                            placeholder: "Введите ответ...",
                            value: answer(),
                            oninput: {
                                let mut answer = answer;
                                move |e: FormEvent| answer.set(e.value())
                            },
                        }
                    }
                }

                SheetFooter {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| on_close.call(()),
                        "Отмена"
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        disabled: loading,
                        onclick: move |_| {
                            let q = question();
                            let a = answer();
                            if q.trim().is_empty() || a.trim().is_empty() {
                                return;
                            }

                            let card_id = card_id.clone();
                            let on_success = on_success;
                            let on_error = on_error;
                            spawn(async move {
                                match edit_card(card_id, q, a).await {
                                    Ok(_) => {
                                        on_success.call("Карточка обновлена".to_string())
                                    }
                                    Err(e) => on_error.call(format!("Ошибка: {}", e)),
                                }
                            });
                        },
                        {if loading { "Сохранение..." } else { "Сохранить" }}
                    }
                }
            }
        }
    }
}

async fn edit_card(card_id: String, question: String, answer: String) -> Result<(), String> {
    use crate::{DEFAULT_USERNAME, ensure_user, to_error};
    use keikaku::application::use_cases::edit_card::EditCardUseCase;
    use ulid::Ulid;

    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    let card_id_ulid = card_id.parse::<Ulid>().map_err(|e| e.to_string())?;

    EditCardUseCase::new(repo)
        .execute(user_id, card_id_ulid, question, answer, Vec::new())
        .await
        .map_err(to_error)?;

    Ok(())
}
