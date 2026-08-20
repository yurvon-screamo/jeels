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

use crate::repository::HybridUserRepository;
use crate::ui_components::{CardLayout, CardLayoutSize, PageLayout, PageLayoutVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::User;
use origa::traits::UserRepository;

#[component]
pub fn Profile() -> impl IntoView {
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let username = RwSignal::new(String::new());
    let disposed = StoredValue::new(());
    let repo_for_init = repository.clone();

    Effect::new(move |_| {
        let repo = repo_for_init.clone();
        spawn_local(async move {
            if let Ok(Some(user)) = repo.get_current_user().await {
                if disposed.is_disposed() {
                    return;
                }
                username.set(user.username().to_string());
                current_user.set(Some(user));
            }
        });
    });

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="profile-page">
            <CardLayout size=CardLayoutSize::Adaptive test_id="profile-card">
                <ProfileContent />
            </CardLayout>
        </PageLayout>
    }
}
