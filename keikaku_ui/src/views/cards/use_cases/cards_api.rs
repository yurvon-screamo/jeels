use crate::keikaku_api::{ensure_user, init_env, to_error, DEFAULT_USERNAME};
use dioxus::prelude::*;
use keikaku::application::use_cases::{
    create_card::CreateCardUseCase, delete_card::DeleteCardUseCase, edit_card::EditCardUseCase,
    list_cards::ListCardsUseCase,
};
use keikaku::domain::{value_objects::CardContent, VocabularyCard};
use ulid::Ulid;

pub fn use_cards_api() -> UseCardsApi {
    use_hook(|| UseCardsApi {
        loading: use_signal(|| false),
    })
}

#[derive(Clone)]
pub struct UseCardsApi {
    pub loading: Signal<bool>,
}

impl UseCardsApi {
    pub async fn fetch_cards(&self) -> Result<Vec<VocabularyCard>, String> {
        let env = init_env().await?;
        let repo = env.get_repository().await.map_err(to_error)?;
        let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
        ListCardsUseCase::new(repo)
            .execute(user_id)
            .await
            .map_err(to_error)
    }

    pub async fn create_card(
        &self,
        question: String,
        answer: String,
    ) -> Result<VocabularyCard, String> {
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

    pub async fn edit_card(
        &self,
        card_id: &str,
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

    pub async fn delete_card(&self, card_id: &str) -> Result<VocabularyCard, String> {
        let env = init_env().await?;
        let repo = env.get_repository().await.map_err(to_error)?;
        let user_id = ensure_user(env, DEFAULT_USERNAME).await?;

        let card_id_ulid = card_id.parse::<Ulid>().map_err(|e| e.to_string())?;

        DeleteCardUseCase::new(repo)
            .execute(user_id, card_id_ulid)
            .await
            .map_err(to_error)
    }
}
