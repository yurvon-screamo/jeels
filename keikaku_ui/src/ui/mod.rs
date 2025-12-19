//! UI components module contains all shared base UI components for our app.
//! Components are the building blocks of dioxus apps.
//! They can be used to defined common UI elements like buttons, forms, and modals.

mod typography;
pub use typography::{H1, H2, H3, H4, Paragraph, Size, Tag};

mod button;
pub use button::{Button, ButtonVariant};

mod input;
pub use input::{SearchInput, TextInput, Textarea};

mod toggle;
pub use toggle::{Checkbox, Radio, Switch};

mod select;
pub use select::Select;

mod card;
pub use card::Card;

mod error;
pub use error::ErrorCard;

mod layout;
pub use layout::{Section, SectionHeader};

mod pill;
pub use pill::{Pill, StateTone};

mod chart;
pub use chart::{Chart, ChartDataPoint};

mod grid;
pub use grid::Grid;

mod modal;
pub use modal::Modal;

mod notification;
pub use notification::{NotificationBanner, NotificationType};

mod loading;
pub use loading::LoadingState;

mod info_section;
pub use info_section::{InfoSection, InfoSectionTone};

mod stat_card;
pub use stat_card::{MetricTone, StatCard};

mod info_grid;
pub use info_grid::InfoGrid;

mod progress_bar;
pub use progress_bar::ProgressBar;

mod labeled_select;
pub use labeled_select::LabeledSelect;

mod empty_state;
pub use empty_state::EmptyState;

mod heatmap;
pub use heatmap::{Heatmap, HeatmapDataPoint};
