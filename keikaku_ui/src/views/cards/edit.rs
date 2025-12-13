use dioxus::prelude::*;

use crate::ui::{Button, ButtonVariant, Modal, TextInput};

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
    let mut question = use_signal(|| initial_question.clone());
    let mut answer = use_signal(|| initial_answer.clone());

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

                            spawn(async move {
                                // Note: loading state is managed by parent component
                                match edit_card(card_id, q, a).await {
                                    Ok(_) => {
                                        question.set(String::new());
                                        answer.set(String::new());
                                        on_success.call("Карточка обновлена".to_string());
                                    }
                                    Err(e) => {
                                        on_error.call(format!("Ошибка: {}", e));
                                    }
                                }
                            });
                        },
                        disabled: Some(loading),
                        if loading {
                            "Сохранение..."
                        } else {
                            "Сохранить"
                        }
                    }
                }
            }
        }
    }
}

async fn edit_card(card_id: String, question: String, answer: String) -> Result<(), String> {
    use crate::{ensure_user, init_env, to_error, DEFAULT_USERNAME};
    use keikaku::application::use_cases::edit_card::EditCardUseCase;
    use ulid::Ulid;

    let env = init_env().await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    let card_id_ulid = card_id.parse::<Ulid>().map_err(|e| e.to_string())?;

    EditCardUseCase::new(repo)
        .execute(user_id, card_id_ulid, question, answer, Vec::new())
        .await
        .map_err(to_error)?;

    Ok(())
}
