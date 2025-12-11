//! The components module contains all shared components for our app. Components are the building blocks of dioxus apps.
//! They can be used to defined common UI elements like buttons, forms, and modals.

mod hero;
pub use hero::Hero;

mod typography;
pub use typography::{H1, H2, H3, Paragraph, Tag};

mod button;
pub use button::{Button, ButtonVariant, IconButton};

mod input;
pub use input::{TextInput, SearchInput, Textarea};

mod toggle;
pub use toggle::{Switch, Checkbox, Radio};

mod select;
pub use select::Select;

mod card;
pub use card::Card;
