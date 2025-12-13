//! UI components module contains all shared base UI components for our app.
//! Components are the building blocks of dioxus apps.
//! They can be used to defined common UI elements like buttons, forms, and modals.

mod typography;
pub use typography::{Paragraph, Tag, H1, H2, H3};

mod button;
pub use button::{Button, ButtonVariant, IconButton};

mod input;
pub use input::{SearchInput, TextInput, Textarea};

mod toggle;
pub use toggle::{Checkbox, Radio, Switch};

mod select;
// Select временно не используется в новых экранах, оставляем модуль при необходимости
// pub use select::Select;

mod card;
pub use card::Card;

mod error;
pub use error::ErrorCard;

mod layout;
pub use layout::{Section, SectionHeader};

mod dashboard;
pub use dashboard::{MetricCard, MetricTone, Pill};

mod chart;
pub use chart::{Chart, ChartDataPoint};

mod modal;
pub use modal::Modal;

mod notification;
pub use notification::{NotificationBanner, NotificationType};

mod loading;
pub use loading::LoadingState;
