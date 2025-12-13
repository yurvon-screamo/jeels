use dioxus::prelude::*;

use crate::ui::{Button, ButtonVariant, Modal, TextInput};

#[component]
pub fn CreateModal(
    on_close: EventHandler<()>,
    on_success: EventHandler<String>,
    on_error: EventHandler<String>,
    loading: bool,
) -> Element {
    let mut question = use_signal(String::new);
    let mut answer = use_signal(String::new);

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

                            spawn(async move {
                                // Note: loading state is managed by parent component
                                match create_card(q, a).await {
                                    Ok(_) => {
                                        question.set(String::new());
                                        answer.set(String::new());
                                        on_success.call("Карточка создана".to_string());
                                    }
                                    Err(e) => {
                                        on_error.call(format!("Ошибка: {}", e));
                                    }
                                }
                            });
                        },
                        disabled: Some(loading),
                        if loading {
                            "Создание..."
                        } else {
                            "Создать"
                        }
                    }
                }
            }
        }
    }
}

async fn create_card(question: String, answer: String) -> Result<(), String> {
    use crate::{ensure_user, init_env, to_error, DEFAULT_USERNAME};
    use keikaku::application::use_cases::create_card::CreateCardUseCase;
    use keikaku::domain::value_objects::CardContent;

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
        .map_err(to_error)?;

    Ok(())
}
