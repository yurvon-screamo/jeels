mod chart;
mod metric;
mod metric_card;
mod overview;

pub use overview::Overview;

use crate::components::app_ui::ChartDataPoint;

#[derive(Clone, PartialEq, Default)]
pub(crate) struct OverviewCharts {
    pub stability_data: Vec<ChartDataPoint>,
    pub difficulty_data: Vec<ChartDataPoint>,
    pub new_words_data: Vec<ChartDataPoint>,
    pub learned_words_data: Vec<ChartDataPoint>,
    pub in_progress_words_data: Vec<ChartDataPoint>,
    pub low_stability_words_data: Vec<ChartDataPoint>,
    pub high_difficulty_words_data: Vec<ChartDataPoint>,
}
