use crate::keikaku_api::{ensure_user, init_env, to_error, DEFAULT_USERNAME};
use dioxus::prelude::*;
use keikaku::application::use_cases::translate::TranslateUseCase;

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    Auto,
    JpToRu,
    RuToJp,
}

pub fn use_translate() -> UseTranslate {
    use_hook(|| UseTranslate {
        text: use_signal(String::new),
        direction: use_signal(|| Direction::Auto),
        result: use_signal(|| None as Option<String>),
        loading: use_signal(|| false),
    })
}

#[derive(Clone, PartialEq)]
pub struct UseTranslate {
    pub text: Signal<String>,
    pub direction: Signal<Direction>,
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
    let env = init_env().await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let translation = env.get_translation_service().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

    TranslateUseCase::new(repo, translation)
        .execute(user_id, src)
        .await
        .map_err(to_error)
}
