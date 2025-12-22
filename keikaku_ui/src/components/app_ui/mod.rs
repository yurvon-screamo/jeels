//! App-specific UI components that don't have direct equivalents in dx-components.
//!
//! This module is a landing zone for the old `crate::ui` components that we still use,
//! while the rest of the app migrates to `crate::components::*`.

mod typography;
pub use typography::{H2, H3, H4, Paragraph, Size, Tag};

mod card;
pub use card::Card;

mod layout;
pub use layout::SectionHeader;

mod error;
pub use error::ErrorCard;

mod chart;
pub use chart::{Chart, ChartDataPoint};

mod grid;
pub use grid::Grid;

mod info_grid;
pub use info_grid::InfoGrid;

mod pill;
pub use pill::{Pill, StateTone};

mod stat_card;
pub use stat_card::{MetricTone, StatCard};

mod info_section;
pub use info_section::{InfoSection, InfoSectionTone};

mod loading;
pub use loading::LoadingState;

mod empty_state;
pub use empty_state::EmptyState;

mod heatmap;
pub use heatmap::{Heatmap, HeatmapDataPoint};
