mod chart;
mod header;
mod metric;
mod metric_card;
mod overview;

pub use overview::Overview;

use crate::ui::ChartDataPoint;

#[derive(Clone, PartialEq, Default)]
pub(crate) struct OverviewCharts {
    pub stability_data: Vec<ChartDataPoint>,
    pub words_progress_data: Vec<ChartDataPoint>,
    pub lessons_data: Vec<ChartDataPoint>,
}
