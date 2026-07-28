use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::User;
use origa::use_cases::ToggleFavoriteUseCase;
use ulid::Ulid;

pub fn create_toggle_favorite_callback(
    repository: HybridUserRepository,
    current_user: RwSignal<Option<User>>,
    refresh_trigger: RwSignal<u32>,
) -> (Callback<Ulid>, RwSignal<bool>) {
    let is_pending = RwSignal::new(false);
    let callback = Callback::new(move |card_id: Ulid| {
        let repository = repository.clone();
        let pending = is_pending;
        spawn_local(async move {
            pending.set(true);
            let use_case = ToggleFavoriteUseCase::new(&repository);
            let result = use_case.execute(card_id).await;
            if result.is_ok() {
                current_user.update(|u| {
                    if let Some(user) = u {
                        let _ = user.toggle_favorite(card_id);
                    }
                });
                refresh_trigger.update(|v| *v += 1);
            }
            pending.set(false);
        });
    });
    (callback, is_pending)
}
