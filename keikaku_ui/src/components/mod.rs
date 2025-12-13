//! The components module contains all shared components for our app. Components are the building blocks of dioxus apps.
//! They can be used to defined common UI elements like buttons, forms, and modals.

mod typography;
pub use typography::{Paragraph, H1, H2};

mod button;
pub use button::{Button, ButtonVariant};

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

mod dashboard;
pub use dashboard::{MetricCard, MetricTone, Pill, Section, SectionHeader};

mod chart;
pub use chart::{Chart, ChartDataPoint};

mod modal;
pub use modal::Modal;

mod notification;
pub use notification::{NotificationBanner, NotificationType};

mod loading;
pub use loading::LoadingState;

mod cards;
pub use cards::{CardsFilters, CardsHeader, CardsList, CardsStats, FilterStatus, SortBy, UiCard};

mod keyboard;
pub use keyboard::{handle_key_event, KeyAction};
