use crate::loaders::recalculate_user_jlpt_progress;
use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::traits::UserRepository;
use ulid::Ulid;

use super::scoring_helpers::ScoringCard;

/// Reactive state shared between `ScoringStep` and the mark-all handler.
/// Grouping the signals into a struct keeps the helper's argument list
/// under clippy's threshold and documents the contract in one place.
pub(super) struct MarkAllState {
    pub repository: HybridUserRepository,
    pub disposed: StoredValue<()>,
    pub mark_all_trigger: RwSignal<u32>,
    pub cards: RwSignal<Vec<ScoringCard>>,
    pub current_index: RwSignal<usize>,
    pub is_loading: RwSignal<bool>,
    pub is_rating: RwSignal<bool>,
    pub scoring_completed: RwSignal<bool>,
    pub known_count: RwSignal<usize>,
}

/// Wire up the "mark everything remaining as known" trigger emitted by the
/// parent Onboarding component. Lives in its own module so `scoring_step.rs`
/// stays under the file-size limit; the trigger pattern (`RwSignal<u32>`)
/// keeps the parent→child IPC type-neutral, avoiding `recursion_limit`
/// pressure from prop-passing large structs.
pub(super) fn spawn_mark_all_effect(state: MarkAllState) {
    let MarkAllState {
        repository,
        disposed,
        mark_all_trigger,
        cards,
        current_index,
        is_loading,
        is_rating,
        scoring_completed,
        known_count,
    } = state;

    Effect::new(move |_| {
        let trigger_val = mark_all_trigger.get();
        if trigger_val == 0 {
            return;
        }
        if is_loading.get()
            || scoring_completed.get()
            || cards.get().is_empty()
            || is_rating.get_untracked()
        {
            return;
        }

        let remaining_ids: Vec<Ulid> = cards
            .get_untracked()
            .iter()
            .skip(current_index.get_untracked())
            .map(|c| c.card_id)
            .collect();

        if remaining_ids.is_empty() {
            return;
        }

        is_rating.set(true);

        let repo = repository.clone();
        spawn_local(async move {
            let Ok(Some(mut user)) = repo.get_current_user().await else {
                is_rating.set(false);
                return;
            };

            let mut success_count: usize = 0;
            for card_id in &remaining_ids {
                if let Some(study_card) = user.knowledge_set().get_card(*card_id) {
                    if !study_card.memory().is_new() {
                        continue;
                    }
                }

                if user.mark_card_as_known(*card_id).is_ok() {
                    success_count += 1;
                } else {
                    tracing::warn!("Failed to rate card {} in batch mark-all", card_id);
                }
            }

            if disposed.is_disposed() {
                return;
            }

            recalculate_user_jlpt_progress(&mut user);
            if repo.save(&user).await.is_ok() {
                known_count.update(|c| *c += success_count);
                scoring_completed.set(true);
            } else {
                tracing::error!("Failed to save user after batch mark-all-known");
            }

            is_rating.set(false);
        });
    });
}
