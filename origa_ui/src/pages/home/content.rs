use super::content_sync::{
    run_sync, show_sync_error_toast, show_sync_success_toast, show_sync_toast,
};
use super::{
    ActivityChart, ActivityDataPoint, CompletionForecast, JlptProgressCard, JlptSkeleton,
    RatingRatio, RecentlyStudiedItem, StudiedTodayList, TodayOverview, TodayOverviewCard,
    WelcomeCard, compute_30day_chart_data, compute_completion_forecast, compute_rating_ratio,
    compute_studied_today, compute_today_overview,
};
use crate::i18n::use_i18n;
use crate::loaders::recalculate_user_jlpt_progress;
use crate::repository::{HybridUserRepository, set_last_sync_time};
use crate::store::ConnectivityStore;
use crate::ui_components::{ToastContainer, ToastData};
use crate::utils::display_name::display_name_for;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::JlptProgress;
use origa::traits::UserRepository;
use std::collections::HashSet;

#[component]
pub fn HomeContent(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let jlpt_progress = RwSignal::new(JlptProgress::new());
    let today_overview = RwSignal::new(TodayOverview::default());
    let recent_studied = RwSignal::new(Vec::<RecentlyStudiedItem>::new());
    let chart_data = RwSignal::new(Vec::<ActivityDataPoint>::new());
    let rating_ratio: RwSignal<Option<RatingRatio>> = RwSignal::new(None);
    let known_kanji: RwSignal<HashSet<char>> = RwSignal::new(HashSet::new());
    let forecast: RwSignal<CompletionForecast> = RwSignal::new(CompletionForecast::default());

    let is_loading = RwSignal::new(true);
    let user_name: RwSignal<String> = RwSignal::new(String::new());
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());
    let disposed = StoredValue::new(());

    let persist_user = move |repo: HybridUserRepository, user: origa::domain::User| {
        spawn_local(async move {
            if disposed.is_disposed() {
                return;
            }
            if let Err(e) = repo.save(&user).await {
                tracing::warn!("Failed to persist JLPT recalculation: {e}");
            }
        });
    };

    let repo_for_init = repository.clone();
    Effect::new(move |_| {
        let repo = repo_for_init.clone();
        spawn_local(async move {
            match repo.get_current_user().await {
                Ok(Some(mut user)) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    recalculate_user_jlpt_progress(&mut user);
                    user_name.set(display_name_for(user.username(), user.email()));

                    let ks = user.knowledge_set();
                    jlpt_progress.set(user.jlpt_progress().clone());
                    known_kanji.set(ks.get_known_kanji());

                    today_overview.set(compute_today_overview(ks, ks.lesson_history()));
                    recent_studied.set(compute_studied_today(ks, user.native_language()));
                    chart_data.set(compute_30day_chart_data(
                        ks.lesson_history(),
                        user.native_language(),
                    ));
                    rating_ratio.set(compute_rating_ratio(ks.lesson_history()));
                    forecast.set(compute_completion_forecast(
                        ks,
                        ks.lesson_history(),
                        user.native_language(),
                    ));

                    is_loading.set(false);

                    persist_user(repo.clone(), user.clone());
                },
                Ok(None) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    is_loading.set(false);
                },
                Err(e) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    tracing::error!("Home: get_current_user error: {:?}", e);
                    is_loading.set(false);
                },
            }
        });
    });

    let repo_sync = repository.clone();
    let i18n_sync = i18n;
    let connectivity = use_context::<ConnectivityStore>();
    Effect::new(move |_| {
        let repo = repo_sync.clone();
        let i18n = i18n_sync;
        let is_online = connectivity
            .as_ref()
            .map(|c| c.is_online.get())
            .unwrap_or(true);
        if !is_online {
            return;
        }
        spawn_local(async move {
            show_sync_toast(toasts, i18n);

            let save_repo = repo.clone();
            match run_sync(repo).await {
                Ok(Some(mut user)) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    recalculate_user_jlpt_progress(&mut user);
                    let ks = user.knowledge_set();
                    jlpt_progress.set(user.jlpt_progress().clone());
                    known_kanji.set(ks.get_known_kanji());
                    today_overview.set(compute_today_overview(ks, ks.lesson_history()));
                    recent_studied.set(compute_studied_today(ks, user.native_language()));
                    chart_data.set(compute_30day_chart_data(
                        ks.lesson_history(),
                        user.native_language(),
                    ));
                    rating_ratio.set(compute_rating_ratio(ks.lesson_history()));
                    forecast.set(compute_completion_forecast(
                        ks,
                        ks.lesson_history(),
                        user.native_language(),
                    ));
                    show_sync_success_toast(toasts, i18n);
                    set_last_sync_time(js_sys::Date::now() as u64 / 1000);

                    persist_user(save_repo, user.clone());
                },
                Ok(None) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    show_sync_success_toast(toasts, i18n);
                    set_last_sync_time(js_sys::Date::now() as u64 / 1000);
                },
                Err(e) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    tracing::error!("Home: sync error: {:?}", e);
                    show_sync_error_toast(toasts, i18n, &e);
                },
            }
        });
    });

    view! {
        <main class="flex-1" data-testid=test_id_val>
            <div class="py-4 sm:py-5 space-y-6 sm:space-y-8">
                <WelcomeCard
                    username=Signal::from(user_name)
                    test_id=Signal::derive(|| "home-welcome".to_string())
                />

                <Show
                    when=move || !is_loading.get()
                    fallback=move || view! { <JlptSkeleton /> }
                >
                    <JlptProgressCard
                        jlpt_progress=Signal::derive(move || jlpt_progress.get())
                        test_id=Signal::derive(|| "home-jlpt-progress".to_string())
                    />

                    <div class="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(280px,5fr)_minmax(360px,7fr)] lg:gap-8">
                        <TodayOverviewCard
                            overview=Signal::derive(move || today_overview.get())
                            forecast=Signal::derive(move || forecast.get())
                            test_id=Signal::derive(|| "home-today-overview".to_string())
                        />
                        <ActivityChart
                            chart_data=Signal::derive(move || chart_data.get())
                            rating_ratio=Signal::derive(move || rating_ratio.get())
                            test_id=Signal::derive(|| "home-activity-chart".to_string())
                        />
                    </div>

                    <StudiedTodayList
                        items=Signal::derive(move || recent_studied.get())
                        known_kanji=Signal::derive(move || known_kanji.get())
                        test_id=Signal::derive(|| "home-recent-study".to_string())
                    />
                </Show>
            </div>

            <ToastContainer
                toasts=toasts
                duration_ms=5000
                test_id=Signal::derive(|| "home-toasts".to_string())
            />
        </main>
    }
}
