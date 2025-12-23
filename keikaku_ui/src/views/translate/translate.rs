use crate::{DEFAULT_USERNAME, ensure_user, to_error};
use dioxus::prelude::*;
use keikaku::application::use_cases::translate::TranslateUseCase;
use keikaku::settings::ApplicationEnvironment;

pub fn use_translate() -> UseTranslate {
    let text = use_signal(String::new);
    let result = use_signal(|| None as Option<String>);
    let loading = use_signal(|| false);

    UseTranslate {
        text,
        result,
        loading,
    }
}

#[derive(Clone, PartialEq)]
pub struct UseTranslate {
    pub text: Signal<String>,
    pub result: Signal<Option<String>>,
    pub loading: Signal<bool>,
}

impl UseTranslate {
    pub fn translate(&mut self) {
        let src = (self.text)();
        if src.trim().is_empty() {
            return;
        }

        self.loading.set(true);
        let mut result = self.result;
        let mut loading = self.loading;

        spawn(async move {
            match run_translate(src).await {
                Ok(r) => result.set(Some(r)),
                Err(e) => result.set(Some(format!("Ошибка: {e}"))),
            }
            loading.set(false);
        });
    }
}

async fn run_translate(src: String) -> Result<String, String> {
    let env = ApplicationEnvironment::get();
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let translation = env
        .get_translation_service(user_id)
        .await
        .map_err(to_error)?;

    TranslateUseCase::new(repo, &translation)
        .execute(user_id, src)
        .await
        .map_err(to_error)
}
