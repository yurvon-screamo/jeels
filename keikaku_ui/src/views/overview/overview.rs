use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use keikaku::domain::VocabularyCard;
use keikaku::settings::ApplicationEnvironment;
use keikaku::{
    application::use_cases::{
        get_user_info::{GetUserInfoUseCase, UserProfile},
        list_cards::ListCardsUseCase,
    },
    domain::daily_history::DailyHistoryItem,
};

use crate::{
    DEFAULT_USERNAME,
    components::app_ui::{ErrorCard, HeatmapDataPoint},
    ensure_user, to_error,
    views::overview::OverviewCharts,
};
use crate::{
    components::app_ui::{ChartDataPoint, StateTone},
    views::overview::{chart::OverviewChartsComponent, metric::OverviewMetrics},
};

#[component]
pub fn Overview() -> Element {
    let profile_resource = use_resource(fetch_profile);
    let cards_resource = use_resource(fetch_cards);

    // Read resources once and store results
    let profile_read = profile_resource.read();
    let cards_read = cards_resource.read();

    match (profile_read.as_ref(), cards_read.as_ref()) {
        (Some(Err(profile_err)), _) => rsx! {
            ErrorCard { message: format!("Ошибка загрузки профиля: {}", profile_err) }
        },
        (_, Some(Err(cards_err))) => rsx! {
            ErrorCard { message: format!("Ошибка загрузки карточек: {}", cards_err) }
        },
        (Some(Ok(profile)), Some(Ok(cards))) => {
            let stats = calculate_stats(cards);
            let charts = build_charts(&profile.lesson_history[..]);
            let heatmap_data = build_heatmap_data(&profile.lesson_history[..]);

            rsx! {
                OverviewMetrics { stats, heatmap_data }
                OverviewChartsComponent { charts }
            }
        }
        _ => rsx! {
            div { class: "text-text-main", "Загрузка..." }
        },
    }
}

fn format_date(date: DateTime<Utc>) -> String {
    date.naive_local().format("%m.%d %H:%M").to_string()
}

fn build_charts(lesson_history: &[DailyHistoryItem]) -> OverviewCharts {
    let now = Utc::now();
    let ten_days_ago = now - chrono::Duration::days(10);

    let mut lesson_history: Vec<_> = lesson_history
        .iter()
        .filter(|item| item.timestamp() >= ten_days_ago)
        .cloned()
        .collect();

    lesson_history.sort_by_key(|item| item.timestamp());

    let stability_data = lesson_history
        .iter()
        .map(|item| ChartDataPoint {
            label: format_date(item.timestamp()),
            value: item.avg_stability().unwrap_or(0.0),
        })
        .collect();

    let difficulty_data = lesson_history
        .iter()
        .map(|item| ChartDataPoint {
            label: format_date(item.timestamp()),
            value: item.avg_difficulty().unwrap_or(0.0),
        })
        .collect();

    let new_words_data = lesson_history
        .iter()
        .map(|item| ChartDataPoint {
            label: format_date(item.timestamp()),
            value: item.new_words() as f64,
        })
        .collect();

    let learned_words_data = lesson_history
        .iter()
        .map(|item| ChartDataPoint {
            label: format_date(item.timestamp()),
            value: item.known_words() as f64,
        })
        .collect();

    let in_progress_words_data = lesson_history
        .iter()
        .map(|item| ChartDataPoint {
            label: format_date(item.timestamp()),
            value: item.in_progress_words() as f64,
        })
        .collect();

    let low_stability_words_data = lesson_history
        .iter()
        .map(|item| ChartDataPoint {
            label: format_date(item.timestamp()),
            value: item.low_stability_words() as f64,
        })
        .collect();

    OverviewCharts {
        stability_data,
        difficulty_data,
        new_words_data,
        learned_words_data,
        in_progress_words_data,
        low_stability_words_data,
    }
}

async fn fetch_profile() -> Result<UserProfile, String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    GetUserInfoUseCase::new(repo)
        .execute(user_id)
        .await
        .map_err(to_error)
}

async fn fetch_cards() -> Result<Vec<VocabularyCard>, String> {
    let env = ApplicationEnvironment::get();
    let repo = env.get_repository().await.map_err(to_error)?;
    let user_id = ensure_user(env, DEFAULT_USERNAME).await?;
    ListCardsUseCase::new(repo)
        .execute(user_id)
        .await
        .map_err(to_error)
}

fn calculate_stats(cards: &Vec<VocabularyCard>) -> OverviewStats {
    let total_cards = cards.len();
    let due_cards = cards.iter().filter(|card| card.memory().is_due()).count();
    let new_cards = cards.iter().filter(|card| card.memory().is_new()).count();
    let learning_cards = cards
        .iter()
        .filter(|card| card.memory().is_in_progress())
        .count();
    let known_cards = cards
        .iter()
        .filter(|card| card.memory().is_known_card())
        .count();
    let low_stability_cards = cards
        .iter()
        .filter(|card| card.memory().is_low_stability())
        .count();

    OverviewStats {
        total_cards,
        due_cards,
        new_cards,
        learning_cards,
        known_cards,
        low_stability_cards,
    }
}

#[derive(Clone, PartialEq, Default)]
pub struct OverviewStats {
    pub total_cards: usize,
    pub due_cards: usize,
    pub new_cards: usize,
    pub learning_cards: usize,
    pub known_cards: usize,
    pub low_stability_cards: usize,
}

fn build_heatmap_data(
    lesson_history: &[keikaku::domain::daily_history::DailyHistoryItem],
) -> Vec<HeatmapDataPoint> {
    lesson_history
        .iter()
        .map(|item| {
            let date = item.timestamp().date_naive();
            let minutes = item.total_duration().num_minutes() as u32;
            HeatmapDataPoint::new(date, minutes)
        })
        .collect()
}

pub type MetricData = (String, String, String, StateTone);
