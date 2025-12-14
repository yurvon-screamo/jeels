use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use keikaku::application::use_cases::{
    get_user_info::{GetUserInfoUseCase, UserProfile},
    list_cards::ListCardsUseCase,
};
use keikaku::domain::VocabularyCard;
use keikaku::settings::ApplicationEnvironment;

use crate::{
    ensure_user, to_error, ui::ErrorCard, views::overview::OverviewCharts, DEFAULT_USERNAME,
};
use crate::{
    ui::{ChartDataPoint, StateTone},
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
            let primary_metrics = build_primary_metrics(&stats);
            let card_status_metrics = build_card_status_metrics(&stats);
            let charts = build_charts(&profile.lesson_history[..]);

            rsx! {
                OverviewMetrics { primary_metrics, card_status_metrics }
                OverviewChartsComponent { charts }
            }
        }
        _ => rsx! {
            div { class: "bg-bg min-h-screen text-text-main px-6 py-8", "Загрузка..." }
        },
    }
}

fn format_date(date: DateTime<Utc>) -> String {
    date.naive_local().format("%m.%d %H:%M").to_string()
}

fn build_charts(
    lesson_history: &[keikaku::domain::daily_history::DailyHistoryItem],
) -> OverviewCharts {
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
        .rev()
        .take(7)
        .map(|item| ChartDataPoint {
            label: format_date(item.timestamp()),
            value: item.known_words() as f64,
        })
        .collect();

    OverviewCharts {
        stability_data,
        difficulty_data,
        new_words_data,
        learned_words_data,
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

    let streak_days = 0; // TODO: Implement streak calculation

    OverviewStats {
        total_cards,
        due_cards,
        new_cards,
        learning_cards,
        known_cards,
        streak_days,
    }
}

fn build_primary_metrics(stats: &OverviewStats) -> Vec<MetricData> {
    vec![
        (
            "Всего карточек".to_string(),
            stats.total_cards.to_string(),
            "Общее количество изучаемых карточек".to_string(),
            StateTone::Neutral,
        ),
        (
            "Дней подряд".to_string(),
            stats.streak_days.to_string(),
            "Количество дней непрерывного обучения".to_string(),
            if stats.streak_days > 0 {
                StateTone::Success
            } else {
                StateTone::Neutral
            },
        ),
    ]
}

fn build_card_status_metrics(stats: &OverviewStats) -> Vec<MetricData> {
    vec![
        (
            "Для повторения".to_string(),
            stats.due_cards.to_string(),
            "Карточки, готовые к повторению".to_string(),
            if stats.due_cards > 0 {
                StateTone::Warning
            } else {
                StateTone::Success
            },
        ),
        (
            "Новые".to_string(),
            stats.new_cards.to_string(),
            "Карточки, которые еще не изучались".to_string(),
            StateTone::Info,
        ),
        (
            "Изучаемые".to_string(),
            stats.learning_cards.to_string(),
            "Карточки в процессе изучения".to_string(),
            StateTone::Neutral,
        ),
        (
            "Изученные".to_string(),
            stats.known_cards.to_string(),
            "Карточки, которые хорошо запомнены".to_string(),
            StateTone::Success,
        ),
    ]
}

#[derive(Default)]
struct OverviewStats {
    pub total_cards: usize,
    pub due_cards: usize,
    pub new_cards: usize,
    pub learning_cards: usize,
    pub known_cards: usize,
    pub streak_days: usize,
}

type MetricData = (String, String, String, StateTone);
