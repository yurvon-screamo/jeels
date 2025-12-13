//! UI components module contains all shared base UI components for our app.
//! Components are the building blocks of dioxus apps.
//! They can be used to defined common UI elements like buttons, forms, and modals.

mod typography;
pub use typography::{Paragraph, H1, H2};

mod button;
pub use button::{Button, ButtonVariant};

mod icon_button;
pub use icon_button::IconButton;

mod input;
pub use input::{SearchInput, TextInput, Textarea};

mod toggle;
pub use toggle::{Checkbox, Radio, Switch};

mod select;

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

mod modal;
pub use modal::Modal;

mod notification;
pub use notification::{NotificationBanner, NotificationType};

mod loading;
pub use loading::LoadingState;
