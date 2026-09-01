use crate::loaders::recalculate_user_jlpt_progress;
use crate::repository::cdn_provider;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use origa::traits::UserRepository;
use origa::use_cases::{
    CompleteOnboardingScoringUseCase, ImportOnboardingSetsUseCase, USERNAME_MAX_CHARS,
};

use super::onboarding_state::OnboardingState;

/// Persists the display name entered on the intro step when the user moves on.
///
/// Fire-and-forget by design: the name is optional personalization, so a save
/// failure is logged and the user still proceeds (the profile page can fix the
/// name later). An empty input skips the save; an unchanged name skips the
/// write (one read still goes out to compare against the current profile).
///
/// Reads-modifies-writes the *current* profile rather than replaying a
/// snapshot captured at page load: the user can flip the language on this very
/// step, and a stale `native_language`/`daily_load` would silently roll the
/// choice back on the full-record save.
pub(super) fn create_save_intro_username_callback(
    repository: crate::repository::HybridUserRepository,
    username: RwSignal<String>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        let name: String = username
            .get_untracked()
            .trim()
            .chars()
            .take(USERNAME_MAX_CHARS)
            .collect();
        if name.is_empty() {
            return;
        }

        let repo = repository.clone();
        spawn_local(async move {
            let Ok(Some(mut user)) = repo.get_current_user().await else {
                tracing::error!("Onboarding intro: get_current_user error");
                return;
            };
            if name == user.username() {
                return;
            }

            user.set_username(name);
            if let Err(e) = repo.save_sync(&user).await {
                tracing::error!("Failed to save username on onboarding intro: {:?}", e);
            }
        });
    })
}

pub(super) fn create_on_skip_callback<N>(
    repository: crate::repository::HybridUserRepository,
    state: RwSignal<OnboardingState>,
    disposed: StoredValue<()>,
    navigate: N,
) -> Callback<()>
where
    N: Fn(&str, NavigateOptions) + Clone + Send + Sync + 'static,
{
    Callback::new(move |_: ()| {
        let repo = repository.clone();
        let nav = navigate.clone();

        spawn_local(async move {
            let Ok(Some(mut user)) = repo.get_current_user().await else {
                tracing::error!("Onboarding skip: get_current_user error");
                return;
            };

            user.set_daily_load(state.get_untracked().daily_load);
            user.mark_set_as_imported(origa::domain::ONBOARDING_SKIPPED_KEY.to_string());
            recalculate_user_jlpt_progress(&mut user);

            // Hard block on remote failure: completing onboarding is a sync
            // checkpoint, so the user must not proceed to /home without a
            // canonical remote record. This differs from the import path
            // below, which logs and continues because the import itself has
            // already committed locally by the time this save runs.
            if let Err(e) = repo.save_sync(&user).await {
                tracing::error!("Onboarding skip: save error: {:?}", e);
                return;
            }

            if disposed.is_disposed() {
                return;
            }
            nav("/home", Default::default());
        });
    })
}

pub(super) fn create_on_start_import_callback(
    repository: crate::repository::HybridUserRepository,
    state: RwSignal<OnboardingState>,
    is_importing: RwSignal<bool>,
    disposed: StoredValue<()>,
) -> Callback<()> {
    Callback::new(move |_: ()| {
        let repo = repository.clone();
        let cdn = cdn_provider();
        let disposed = disposed;
        is_importing.set(true);

        spawn_local(async move {
            let set_ids = state.get().get_final_sets();

            if set_ids.is_empty() {
                tracing::warn!("No sets selected for import");
                is_importing.set(false);
                return;
            }

            // Read the CURRENT profile, not the page-load snapshot: the
            // intro-step name save (and other onboarding writes) land after
            // this page loaded, and the import below persists the FULL
            // record — replaying a stale snapshot would roll those writes
            // back (the lost-display-name bug).
            let Ok(Some(mut user)) = repo.get_current_user().await else {
                tracing::error!("User not loaded");
                is_importing.set(false);
                return;
            };

            // recalculate_user_jlpt_progress depends on JLPT_CONTENT (UI-side
            // CDN singleton), so it cannot move into origa/. Applied here so
            // the single save_sync inside execute persists both it and the
            // imported cards together.
            user.set_daily_load(state.get_untracked().daily_load);
            recalculate_user_jlpt_progress(&mut user);

            let use_case = ImportOnboardingSetsUseCase::new(&repo, cdn);
            let target_level = state.get_untracked().target_level();
            let result = use_case.execute(user, set_ids, target_level).await;

            if disposed.is_disposed() {
                return;
            }
            match result {
                Ok(import_result) => {
                    tracing::info!(
                        "Imported: {} vocabulary, {} kanji, {} grammar, {} duplicates skipped",
                        import_result.created_vocabulary,
                        import_result.created_kanji,
                        import_result.created_grammar,
                        import_result.skipped_duplicates
                    );

                    state.update(|s| {
                        s.go_to_next_step();
                    });
                    is_importing.set(false);
                },
                Err(e) => {
                    tracing::error!("Import failed: {:?}", e);
                    is_importing.set(false);
                },
            }
        });
    })
}

/// Atomically finishes onboarding scoring: clears the per-click "don't know"
/// records, marks the user as onboarding-completed (so `/home` no longer
/// bounces back to `/onboarding`), persists both via a single `save_sync`,
/// then seeds ready-to-learn phrase cards for the now-known vocabulary.
///
/// Phrases are a derivative payload: a failure inside
/// [`CompleteOnboardingScoringUseCase`] before the save_sync bubbles up as
/// `Err`, but a seed-step failure is logged and swallowed so the user can
/// still proceed to `/home`. The next dictionary load will re-run seeding
/// with the up-to-date known-vocabulary hash.
pub(super) fn create_on_finish_callback<N>(
    repository: crate::repository::HybridUserRepository,
    is_finishing: RwSignal<bool>,
    disposed: StoredValue<()>,
    navigate: N,
) -> Callback<()>
where
    N: Fn(&str, NavigateOptions) + Clone + Send + Sync + 'static,
{
    Callback::new(move |_: ()| {
        let repo = repository.clone();
        let nav = navigate.clone();
        is_finishing.set(true);
        spawn_local(async move {
            let use_case = CompleteOnboardingScoringUseCase::new(&repo);
            match use_case.execute().await {
                Ok(seeded) => {
                    tracing::info!(seeded_phrases = seeded, "Onboarding scoring completed");
                },
                Err(e) => {
                    tracing::warn!(error = ?e, "CompleteOnboardingScoring failed");
                },
            }
            if disposed.is_disposed() {
                return;
            }
            nav("/home", Default::default());
        });
    })
}
