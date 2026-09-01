pub(crate) mod content;
pub(crate) mod danger_zone_card;
pub(crate) mod legal_card;
pub(crate) mod password_card;
pub(crate) mod personal_data_card;
pub(crate) mod settings_card;

pub use content::ProfileContent;
pub use danger_zone_card::DangerZoneCard;
pub use legal_card::legal_card;
pub use password_card::PasswordCard;
pub use personal_data_card::PersonalDataCard;
pub use settings_card::SettingsCard;

use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;

/// The profile page mounts [`ProfileContent`], which owns its user state via
/// the shared `AuthStore` (the display-name fallback lives in
/// `crate::utils::display_name`).
#[component]
pub fn Profile() -> impl IntoView {
    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="profile-page">
            <CardLayout size=CardLayoutSize::Adaptive test_id="profile-card">
                <ProfileContent />
            </CardLayout>
        </PageLayout>
    }
}
