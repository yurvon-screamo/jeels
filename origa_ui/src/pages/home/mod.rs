pub mod activity_chart;
pub(crate) mod category_grid;
pub mod content;
pub(crate) mod content_sync;
pub mod dashboard_stats;
pub mod home_skeleton;
pub mod jlpt_progress_card;
pub mod recent_study;
pub mod today_overview;
pub mod welcome_card;

#[cfg(all(target_arch = "wasm32", test))]
mod home_wasm_tests;

pub use activity_chart::ActivityChart;
pub use content::HomeContent;
pub use dashboard_stats::{
    ActivityDataPoint, CompletionForecast, RecentlyStudiedItem, compute_30day_chart_data,
    compute_completion_forecast, compute_studied_today,
};
pub use home_skeleton::JlptSkeleton;
pub use jlpt_progress_card::JlptProgressCard;
pub use origa::domain::{RatingRatio, TodayOverview, compute_rating_ratio, compute_today_overview};
pub use recent_study::StudiedTodayList;
pub use today_overview::TodayOverviewCard;
pub use welcome_card::WelcomeCard;

use crate::i18n::*;
use crate::store::auth_store::AuthStore;
use crate::ui_components::{
    PageLayout, PageLayoutVariant, Spinner, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use origa::domain::User;
use origa::traits::UserRepository;

#[component]
pub fn Home() -> impl IntoView {
    let i18n = use_i18n();
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");
    let repository = auth_store.repository().clone();
    let navigate = use_navigate();

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let is_checking_onboarding = RwSignal::new(true);
    let disposed = StoredValue::new(());

    Effect::new({
        let repository = repository.clone();
        let navigate = navigate.clone();
        move |_| {
            let repository = repository.clone();
            let navigate = navigate.clone();
            spawn_local(async move {
                match repository.get_current_user().await {
                    Ok(Some(user)) => {
                        if disposed.is_disposed() {
                            return;
                        }
                        if !user.is_onboarding_completed() {
                            navigate("/onboarding", Default::default());
                            return;
                        }
                        current_user.set(Some(user));
                    },
                    Ok(None) => {
                        navigate("/login", Default::default());
                    },
                    Err(e) => {
                        tracing::error!("Home: get_current_user error: {:?}", e);
                        navigate("/login", Default::default());
                    },
                }
                is_checking_onboarding.set(false);
            });
        }
    });

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="home-page">
            <Show when=move || is_checking_onboarding.get()>
                <div class="py-20 flex flex-col items-center justify-center gap-4" data-testid="home-loading">
                    <Spinner test_id="home-spinner" />
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {t!(i18n, home.loading)}
                    </Text>
                </div>
            </Show>

            <Show when=move || !is_checking_onboarding.get()>
                <div class="flex flex-col" data-testid="home-content">
                    <HomeContent test_id="home-main" />
                </div>
            </Show>
        </PageLayout>
    }
}
