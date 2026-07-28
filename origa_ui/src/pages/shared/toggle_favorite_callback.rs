use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{StudyCard, User};
use origa::use_cases::ToggleFavoriteUseCase;
use ulid::Ulid;

/// Creates a favorite-toggle callback with optimistic UI update and rollback on error.
///
/// `all_cards` is updated locally before the API call so the user sees an instant
/// response; if the API call fails, the optimistic change is rolled back. The same
/// pattern was previously inlined in `pages/kanji/content.rs` and is now shared by
/// every card-list page (words, grammar, kanji, phrases).
pub fn create_toggle_favorite_callback(
    repository: HybridUserRepository,
    current_user: RwSignal<Option<User>>,
    all_cards: RwSignal<Vec<StudyCard>>,
    refresh_trigger: RwSignal<u32>,
) -> (Callback<Ulid>, RwSignal<bool>) {
    let is_pending = RwSignal::new(false);
    let callback = Callback::new(move |card_id: Ulid| {
        let repository = repository.clone();
        let pending = is_pending;
        let all_cards_for_update = all_cards;
        let current_user_for_update = current_user;
        let refresh = refresh_trigger;

        // Optimistic UI update FIRST: flip the card's favorite flag locally
        // before any async work, so the UI reacts instantly.
        toggle_card_in_signal(&all_cards_for_update, card_id);

        spawn_local(async move {
            pending.set(true);
            let use_case = ToggleFavoriteUseCase::new(&repository);
            let result = use_case.execute(card_id).await;
            if result.is_ok() {
                current_user_for_update.update(|u| {
                    if let Some(user) = u {
                        let _ = user.toggle_favorite(card_id);
                    }
                });
                refresh.update(|v| *v += 1);
            } else {
                // Rollback the optimistic update if the API call failed.
                toggle_card_in_signal(&all_cards_for_update, card_id);
            }
            pending.set(false);
        });
    });
    (callback, is_pending)
}

fn toggle_card_in_signal(all_cards: &RwSignal<Vec<StudyCard>>, card_id: Ulid) {
    all_cards.update(|cards| {
        for card in cards.iter_mut() {
            if *card.card_id() == card_id {
                card.toggle_favorite();
            }
        }
    });
}
