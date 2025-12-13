use crate::ui::{ChartDataPoint, MetricTone};

use keikaku::application::use_cases::get_user_info::UserProfile;
use keikaku::domain::{daily_history::DailyHistoryItem, VocabularyCard};

pub struct OverviewStats {
    pub username: String,
    pub total_cards: usize,
    pub due_cards: usize,
    pub new_cards: usize,
    pub learning_cards: usize,
    pub known_cards: usize,
    pub streak_days: usize,
}

impl Default for OverviewStats {
    fn default() -> Self {
        Self {
            username: "Загрузка...".to_string(),
            total_cards: 0,
            due_cards: 0,
            new_cards: 0,
            learning_cards: 0,
            known_cards: 0,
            streak_days: 0,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct OverviewCharts {
    pub stability_data: Vec<ChartDataPoint>,
    pub words_progress_data: Vec<ChartDataPoint>,
    pub lessons_data: Vec<ChartDataPoint>,
}

impl Default for OverviewCharts {
    fn default() -> Self {
        Self {
            stability_data: Vec::new(),
            words_progress_data: Vec::new(),
            lessons_data: Vec::new(),
        }
    }
}

pub type MetricData = (String, String, String, MetricTone);

pub fn calculate_stats(
    profile: Option<&UserProfile>,
    cards: Option<&Vec<VocabularyCard>>,
) -> OverviewStats {
    let username = profile
        .map(|p| p.username.clone())
        .unwrap_or_else(|| "Неизвестный пользователь".to_string());

    let empty_vec = Vec::new();
    let cards_data = cards.unwrap_or(&empty_vec);

    let total_cards = cards_data.len();
    let due_cards = cards_data
        .iter()
        .filter(|card| card.memory().is_due())
        .count();
    let new_cards = cards_data
        .iter()
        .filter(|card| card.memory().is_new())
        .count();
    let learning_cards = cards_data
        .iter()
        .filter(|card| card.memory().is_in_progress())
        .count();
    let known_cards = cards_data
        .iter()
        .filter(|card| card.memory().is_known_card())
        .count();

    let streak_days = 0; // TODO: Implement streak calculation

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
            "Общее количество изучаемых карточек".to_string(),
            MetricTone::Neutral,
        ),
        (
            "Для повторения".to_string(),
            stats.due_cards.to_string(),
            "Карточки, готовые к повторению".to_string(),
            if stats.due_cards > 0 {
                MetricTone::Warning
            } else {
                MetricTone::Success
            },
        ),
        (
            "Новые".to_string(),
            stats.new_cards.to_string(),
            "Карточки, которые еще не изучались".to_string(),
            MetricTone::Info,
        ),
        (
            "Изучаемые".to_string(),
            stats.learning_cards.to_string(),
            "Карточки в процессе изучения".to_string(),
            MetricTone::Neutral,
        ),
        (
            "Изученные".to_string(),
            stats.known_cards.to_string(),
            "Карточки, которые хорошо запомнены".to_string(),
            MetricTone::Success,
        ),
        (
            "Дней подряд".to_string(),
            stats.streak_days.to_string(),
            "Количество дней непрерывного обучения".to_string(),
            if stats.streak_days > 0 {
                MetricTone::Success
            } else {
                MetricTone::Neutral
            },
        ),
    ]
}

pub fn build_charts(
    lesson_history: &[keikaku::domain::daily_history::DailyHistoryItem],
) -> OverviewCharts {
    let stability_data = lesson_history
        .iter()
        .enumerate()
        .map(|(i, item)| ChartDataPoint {
            label: format!("День {}", i + 1),
            value: item.avg_stability().unwrap_or(0.0),
        })
        .collect();

    let words_progress_data = lesson_history
        .iter()
        .enumerate()
        .map(|(i, item)| ChartDataPoint {
            label: format!("День {}", i + 1),
            value: item.total_words() as f64,
        })
        .collect();

    let lessons_data = lesson_history
        .iter()
        .enumerate()
        .rev()
        .take(7)
        .map(|(i, item)| ChartDataPoint {
            label: format!("День {}", 7 - i),
            value: item.total_words() as f64, // Using total_words as lesson count approximation
        })
        .collect();

    OverviewCharts {
        stability_data,
        words_progress_data,
        lessons_data,
    }
}
