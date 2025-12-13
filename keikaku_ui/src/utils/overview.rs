use crate::components::{ChartDataPoint, MetricTone};
use keikaku::application::use_cases::get_user_info::UserProfile;
use keikaku::domain::VocabularyCard;

pub struct OverviewStats {
    pub username: String,
    pub total_cards: usize,
    pub due_cards: usize,
    pub new_cards: usize,
    pub learning_cards: usize,
    pub known_cards: usize,
    pub streak_days: usize,
}

#[derive(Clone, PartialEq)]
pub struct OverviewCharts {
    pub stability_data: Vec<ChartDataPoint>,
    pub words_progress_data: Vec<ChartDataPoint>,
    pub lessons_data: Vec<ChartDataPoint>,
}

pub type MetricData = (String, String, String, MetricTone);

pub fn calculate_stats(
    profile: Option<&UserProfile>,
    cards: Option<&Vec<VocabularyCard>>,
) -> OverviewStats {
    let username = profile
        .map(|p| p.username.clone())
        .unwrap_or_else(|| "Пользователь".to_string());

    let streak_days = profile.map(|p| p.lesson_history.len()).unwrap_or(0);

    let mut total_cards = 0;
    let mut due_cards = 0;
    let mut new_cards = 0;
    let mut learning_cards = 0;
    let mut known_cards = 0;

    if let Some(cards_list) = cards {
        total_cards = cards_list.len();
        for card in cards_list {
            if card.memory().is_new() {
                new_cards += 1;
            } else if card.memory().is_due() {
                due_cards += 1;
            } else if card.memory().is_in_progress() {
                learning_cards += 1;
            } else if card.memory().is_known_card() {
                known_cards += 1;
            }
        }
    }

    OverviewStats {
        username,
        total_cards,
        due_cards,
        new_cards,
        learning_cards,
        known_cards,
        streak_days,
    }
}

pub fn build_metrics(stats: &OverviewStats) -> Vec<MetricData> {
    vec![
        (
            "Всего карточек".to_string(),
            stats.total_cards.to_string(),
            "Общее количество слов".to_string(),
            MetricTone::Info,
        ),
        (
            "К повторению".to_string(),
            stats.due_cards.to_string(),
            "Нуждаются в повторении".to_string(),
            MetricTone::Warning,
        ),
        (
            "Изучаются".to_string(),
            stats.learning_cards.to_string(),
            "В процессе изучения".to_string(),
            MetricTone::Success,
        ),
        (
            "Изучены".to_string(),
            stats.known_cards.to_string(),
            "Хорошо запомнены".to_string(),
            MetricTone::Neutral,
        ),
        (
            "Новые".to_string(),
            stats.new_cards.to_string(),
            "Ожидают изучения".to_string(),
            MetricTone::Info,
        ),
        (
            "Стрик".to_string(),
            format!("{} дн.", stats.streak_days),
            "Дней подряд".to_string(),
            MetricTone::Success,
        ),
    ]
}

pub fn build_charts(
    lesson_history: &[keikaku::domain::daily_history::DailyHistoryItem],
) -> OverviewCharts {
    let stability_data: Vec<ChartDataPoint> = lesson_history
        .iter()
        .enumerate()
        .filter_map(|(i, item)| {
            item.avg_stability().map(|stability| ChartDataPoint {
                label: format!("День {}", i + 1),
                value: stability,
            })
        })
        .collect();

    let words_progress_data: Vec<ChartDataPoint> = lesson_history
        .iter()
        .enumerate()
        .map(|(i, item)| ChartDataPoint {
            label: format!("День {}", i + 1),
            value: item.total_words() as f64,
        })
        .collect();

    let lessons_data: Vec<ChartDataPoint> = lesson_history
        .iter()
        .enumerate()
        .map(|(i, _)| ChartDataPoint {
            label: format!("День {}", i + 1),
            value: 1.0,
        })
        .collect();

    OverviewCharts {
        stability_data,
        words_progress_data,
        lessons_data,
    }
}
