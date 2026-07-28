use std::sync::Arc;

use super::super::shared::{
    CardsLoadedCallback, ListGrouping, card_list_view, create_card_list_context,
};
use super::phrase_card_item::PhraseCardItem;
use crate::i18n::{td_string, use_i18n};
use crate::loaders::phrase_data_loader::load_phrase_details_batch;
use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{Card, StudyCard};
use ulid::Ulid;

#[component]
pub fn PhrasesContent(refresh_trigger: RwSignal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let initial_load_done = RwSignal::new(false);
    let on_cards_loaded: CardsLoadedCallback = Arc::new(move |cards: &[StudyCard]| {
        if initial_load_done.get_untracked() {
            return;
        }

        let phrase_ids: Vec<Ulid> = cards
            .iter()
            .filter_map(|card| {
                if let Card::Phrase(pc) = card.card() {
                    Some(*pc.phrase_id())
                } else {
                    None
                }
            })
            .collect();

        if !phrase_ids.is_empty() {
            let refresh = refresh_trigger;
            let done = initial_load_done;
            spawn_local(async move {
                let results = load_phrase_details_batch(&phrase_ids).await;
                let failed = results.iter().filter(|r| r.is_err()).count();
                if failed > 0 {
                    tracing::warn!(
                        failed,
                        total = phrase_ids.len(),
                        "Some phrase data chunks failed to load"
                    );
                }
                done.set(true);
                refresh.update(|n| *n += 1);
            });
        }
    });

    let ctx = create_card_list_context(
        repository,
        refresh_trigger,
        |card| matches!(card, Card::Phrase(_)),
        Some(on_cards_loaded),
    );

    let ctx_for_render = ctx.clone();
    let empty_message =
        Signal::derive(move || td_string!(i18n.get_locale(), phrases.not_found).to_string());

    card_list_view(
        ctx,
        ListGrouping::Flat,
        true,
        "phrases",
        empty_message,
        Some("grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-4 items-start"),
        move |card| {
        let ctx = ctx_for_render.clone();
        let card_id = *card.card_id();
        view! {
            <PhraseCardItem
                study_card=card
                native_language=ctx.native_lang
                known_kanji=ctx.known_kanji.get()
                on_toggle_favorite=ctx.on_toggle_favorite
                on_mark_as_known=Callback::new(move |_| ctx.on_mark_as_known.run(card_id))
                on_delete=ctx.on_delete
                is_deleting=ctx.is_deleting
                phrase_data_trigger=refresh_trigger
            />
        }
        .into_any()
    })
    .into_any()
}
