//! WASM render tests for `pages/phrases`: `PhrasesHeader` (Router-based
//! PageHeader with the info tooltip).

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen_test::*;

use std::collections::HashSet;

use crate::pages::phrases::header::PhrasesHeader;
use crate::pages::phrases::phrase_card_item::PhraseCardItem;
use crate::test_support::{create_wrapper, mount_with_i18n, mount_with_router};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn phrases_header_renders_title_and_info_tooltip() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router(&wrapper, || view! { <PhrasesHeader /> }.into_any());
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"phrases-header\"]")
            .ok()
            .flatten()
            .is_some(),
        "header root must render"
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"phrases-info-icon\"]")
            .unwrap()
            .is_some(),
        "the info icon must render"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// PhraseCardItem (phrase text/answer degrade without the CDN store)
// ═══════════════════════════════════════════════════════════════════════

fn phrase_study_card() -> origa::domain::StudyCard {
    use origa::domain::{Card, PhraseCard, StudyCard};
    StudyCard::new(Card::Phrase(PhraseCard::new(ulid::Ulid::new())))
}

#[wasm_bindgen_test]
async fn phrase_card_item_without_store_renders_skeleton_and_status() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let trigger = RwSignal::new(0u32);
        view! {
            <PhraseCardItem
                study_card=phrase_study_card()
                native_language=Signal::from(origa::domain::NativeLanguage::Russian)
                known_kanji=HashSet::new()
                on_toggle_favorite=Callback::new(|_| ())
                on_mark_as_known=Callback::new(|_| ())
                on_delete=Callback::new(|_| ())
                is_deleting=Signal::from(false)
                phrase_data_trigger=trigger
            />
        }
        .into_any()
    });
    tick().await;

    let item = wrapper
        .query_selector("[data-testid=\"phrases-card-item\"]")
        .unwrap()
        .unwrap();
    // Without the phrase store the text degrades to the skeleton placeholder
    let skeleton = item.query_selector(".anima-skeleton-paper");
    assert!(
        skeleton.is_ok_and(|sk| sk.is_some()),
        "missing phrase text must render the loading skeleton"
    );
    // The status tag and FSRS metrics render regardless
    assert!(
        item.query_selector(".tag").is_ok_and(|t| t.is_some()),
        "status tag must render"
    );
}

#[wasm_bindgen_test]
async fn phrase_card_item_delete_opens_confirm_modal() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let trigger = RwSignal::new(0u32);
        view! {
            <PhraseCardItem
                study_card=phrase_study_card()
                native_language=Signal::from(origa::domain::NativeLanguage::Russian)
                known_kanji=HashSet::new()
                on_toggle_favorite=Callback::new(|_| ())
                on_mark_as_known=Callback::new(|_| ())
                on_delete=Callback::new(|_| ())
                is_deleting=Signal::from(false)
                phrase_data_trigger=trigger
            />
        }
        .into_any()
    });
    tick().await;

    use wasm_bindgen::JsCast;
    wrapper
        .query_selector("[data-testid=\"phrases-card-item-delete-btn\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"phrases-delete-modal\"]")
            .unwrap()
            .is_some(),
        "delete click must open the confirmation modal"
    );
}
