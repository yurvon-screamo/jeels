use crate::loaders::recalculate_user_jlpt_progress;
use crate::repository::cdn_provider;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use origa::domain::User;
use origa::traits::UserRepository;
use origa::use_cases::{CompleteOnboardingScoringUseCase, ImportOnboardingSetsUseCase};

use super::onboarding_state::OnboardingState;

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
    current_user: RwSignal<Option<User>>,
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

            let Some(mut user) = current_user.get() else {
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
