use crate::keikaku_api::{ensure_user, init_env, to_error, DEFAULT_USERNAME};
use crate::utils::overview::{
    build_charts, build_metrics, calculate_stats, MetricData, OverviewCharts, OverviewStats,
};
use dioxus::prelude::*;
use keikaku::application::use_cases::{
    get_user_info::{GetUserInfoUseCase, UserProfile},
    list_cards::ListCardsUseCase,
};
use keikaku::domain::VocabularyCard;

pub fn use_overview_data() -> UseOverviewData {
    use_hook(|| UseOverviewData {
        profile: use_resource(|| async move { fetch_profile().await }),
        cards: use_resource(|| async move { fetch_cards().await }),
    })
}

#[derive(Clone)]
pub struct UseOverviewData {
    pub profile: Resource<Result<UserProfile, String>>,
    pub cards: Resource<Result<Vec<VocabularyCard>, String>>,
}

impl UseOverviewData {
    pub fn get_all_data(
        &self,
    ) -> (
        Option<String>,
        OverviewStats,
        Vec<MetricData>,
        OverviewCharts,
    ) {
        let profile_binding = self.profile.read();
        let cards_binding = self.cards.read();

        // Check for errors
        let has_error = if let Some(Err(error)) = profile_binding.as_ref() {
            Some(format!("Ошибка загрузки профиля: {}", error))
        } else if let Some(Err(error)) = cards_binding.as_ref() {
            Some(format!("Ошибка загрузки карточек: {}", error))
        } else {
            None
        };

        // Get data
        let profile = profile_binding.as_ref().and_then(|r| r.as_ref().ok());
        let cards = cards_binding.as_ref().and_then(|r| r.as_ref().ok());

        let stats = calculate_stats(profile, cards);
        let metrics = build_metrics(&stats);

        let lesson_history = profile.map(|p| p.lesson_history.as_slice()).unwrap_or(&[]);
        let charts = build_charts(lesson_history);

        (has_error, stats, metrics, charts)
    }

    pub fn get_stats(&self) -> OverviewStats {
        let profile_binding = self.profile.read();
        let profile = profile_binding.as_ref().and_then(|r| r.as_ref().ok());
        let cards_binding = self.cards.read();
        let cards = cards_binding.as_ref().and_then(|r| r.as_ref().ok());
        calculate_stats(profile, cards)
    }

    pub fn get_metrics(&self) -> Vec<MetricData> {
        let stats = self.get_stats();
        build_metrics(&stats)
    }

    pub fn get_charts(&self) -> OverviewCharts {
        let profile_binding = self.profile.read();
        let lesson_history = profile_binding
            .as_ref()
            .and_then(|r| r.as_ref().ok())
            .map(|p| p.lesson_history.as_slice())
            .unwrap_or(&[]);
        build_charts(lesson_history)
    }

    pub fn has_error(&self) -> Option<String> {
        if let Some(Err(error)) = self.profile.read().as_ref() {
            Some(format!("Ошибка загрузки профиля: {}", error))
        } else {
            None
        }
    }
}

async fn fetch_profile() -> Result<UserProfile, String> {
    let env = init_env().await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    GetUserInfoUseCase::new(repo)
        .execute(user_id)
        .await
        .map_err(to_error)
}

async fn fetch_cards() -> Result<Vec<VocabularyCard>, String> {
    let env = init_env().await?;
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    ListCardsUseCase::new(repo)
        .execute(user_id)
        .await
        .map_err(to_error)
}
