use dioxus::prelude::*;

use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use crate::components::sheet::{
    Sheet, SheetContent, SheetFooter, SheetHeader, SheetSide, SheetTitle,
};
use keikaku::settings::ApplicationEnvironment;

#[component]
pub fn CreateModal(
    on_close: EventHandler<()>,
    on_success: EventHandler<String>,
    on_error: EventHandler<String>,
    loading: bool,
) -> Element {
    let question = use_signal(String::new);

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
                    SheetTitle { "Создать карточку" }
                }

                div { class: "space-y-4",
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Вопрос" }
                        Input {
                            placeholder: "Введите текст...",
                            value: question(),
                            oninput: {
                                let mut question = question;
                                move |e: FormEvent| question.set(e.value())
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
                            if q.trim().is_empty() {
                                return;
                            }

                            let on_success = on_success;
                            let on_error = on_error;
                            spawn(async move {
                                match create_card(q).await {
                                    Ok(_) => {
                                        on_success.call("Карточка создана".to_string());
                                    }
                                    Err(e) => {
                                        on_error.call(format!("Ошибка: {}", e));
                                    }
                                }
                            });
                        },
                        {if loading { "Создание..." } else { "Создать" }}
                    }
                }
            }
        }
    }
}

async fn create_card(question: String) -> Result<(), String> {
    use crate::{DEFAULT_USERNAME, ensure_user, to_error};
    use keikaku::application::use_cases::create_vocabulary_card::CreateVocabularyCardUseCase;

    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let llm_service = env.get_llm_service(user_id).await.map_err(to_error)?;

    CreateVocabularyCardUseCase::new(repo, &llm_service)
        .execute(user_id, question)
        .await
        .map_err(to_error)?;

    Ok(())
}
