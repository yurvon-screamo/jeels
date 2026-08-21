//! WASM render tests for `pages/sets` components: `SetCard`, `SetWordItem`,
//! `SetsHeader` (Router-based PageHeader).

#![cfg(all(target_arch = "wasm32", test))]

use std::collections::HashSet;

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

use super::header::SetsHeader;
use super::set_card::SetCard;
use super::set_word_item::SetWordItem;
use super::types::SetInfo;
use crate::test_support::{create_wrapper, mount_with_i18n, mount_with_router, shared_cell};

wasm_bindgen_test_configure!(run_in_browser);

fn sample_set(imported: bool) -> SetInfo {
    SetInfo {
        set_id: "set-1".into(),
        title: "Migii N5 Vocabulary".into(),
        description: "Top frequent words".into(),
        word_count: Some(42),
        set_type: "vocab".into(),
        level: origa::domain::JapaneseLevel::N5,
        is_imported: imported,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SetCard
// ══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn set_card_not_imported_shows_checkbox_and_import() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let selected: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
        view! {
            <SetCard
                set_info=sample_set(false)
                known_kanji=HashSet::new()
                on_import=Callback::new(|_| ())
                selected_sets=selected
                on_toggle_select=Callback::new(|_| ())
            />
        }
        .into_any()
    });
    tick().await;

    let card = wrapper
        .query_selector("[data-testid=\"sets-card-item\"]")
        .unwrap()
        .unwrap();
    assert!(
        card.text_content().unwrap().contains("Migii N5 Vocabulary"),
        "title must render"
    );

    let checkbox = card.query_selector("input[type=\"checkbox\"]");
    assert!(
        checkbox.is_ok_and(|c| c.is_some()),
        "non-imported set must offer the selection checkbox"
    );
}

#[wasm_bindgen_test]
async fn set_card_imported_hides_checkbox() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let selected: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
        view! {
            <SetCard
                set_info=sample_set(true)
                known_kanji=HashSet::new()
                on_import=Callback::new(|_| ())
                selected_sets=selected
                on_toggle_select=Callback::new(|_| ())
            />
        }
        .into_any()
    });
    tick().await;

    let card = wrapper
        .query_selector("[data-testid=\"sets-card-item\"]")
        .unwrap()
        .unwrap();
    let checkbox = card.query_selector("input[type=\"checkbox\"]");
    assert!(
        checkbox.is_ok_and(|c| c.is_none()),
        "imported set must not offer selection"
    );
}

#[wasm_bindgen_test]
async fn set_card_selected_gets_ring_class() {
    let wrapper = create_wrapper();
    let (set_selected, get_selected) = shared_cell::<RwSignal<HashSet<String>>>();
    mount_with_i18n(&wrapper, move || {
        let selected = RwSignal::new(HashSet::new());
        set_selected.set(Some(selected));
        view! {
            <SetCard
                set_info=sample_set(false)
                known_kanji=HashSet::new()
                on_import=Callback::new(|_| ())
                selected_sets=selected
                on_toggle_select=Callback::new(|_| ())
            />
        }
        .into_any()
    });
    let selected = get_selected.get().expect("captured");
    tick().await;

    // Act: select the set
    selected.update(|s| {
        s.insert("set-1".to_string());
    });
    tick().await;

    let card = wrapper
        .query_selector("[data-testid=\"sets-card-item\"]")
        .unwrap()
        .unwrap();
    let class = card.get_attribute("class").unwrap_or_default();
    assert!(
        class.contains("ring-2"),
        "selected card must be ringed; got: {class}"
    );
}

#[wasm_bindgen_test]
async fn set_card_toggle_select_dispatches_set_id() {
    let wrapper = create_wrapper();
    let (set_toggled, get_toggled) = shared_cell::<RwSignal<Option<String>>>();
    mount_with_i18n(&wrapper, move || {
        let toggled = RwSignal::new(None);
        set_toggled.set(Some(toggled));
        let selected: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
        view! {
            <SetCard
                set_info=sample_set(false)
                known_kanji=HashSet::new()
                on_import=Callback::new(|_| ())
                selected_sets=selected
                on_toggle_select=Callback::new(move |id: String| toggled.set(Some(id)))
            />
        }
        .into_any()
    });
    let toggled = get_toggled.get().expect("captured");
    tick().await;

    // Act: click the checkbox
    let checkbox = wrapper
        .query_selector("[data-testid=\"sets-card-item\"] input[type=\"checkbox\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>();
    checkbox.click();
    tick().await;

    assert_eq!(
        toggled.get(),
        Some("set-1".to_string()),
        "checkbox must dispatch the set id"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// SetWordItem
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn set_word_item_renders_word_and_known_icon() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let selected_words: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
        view! {
            <SetWordItem
                word="ねこ".to_string()
                known_meaning=None
                outcome=origa::domain::WordImportOutcome::AlreadyExists
                selected_words=selected_words
                known_kanji=HashSet::new()
                on_toggle=Callback::new(|()| ())
            />
        }
        .into_any()
    });
    tick().await;

    let item = wrapper
        .query_selector("[data-testid=\"sets-drawer-item\"]")
        .unwrap()
        .unwrap();
    assert!(
        item.text_content().unwrap().contains("ねこ"),
        "the word must render"
    );
    // Known words render the check-circle SVG icon (injected via inner_html)
    let svg = item.query_selector("svg");
    assert!(
        svg.is_ok_and(|s| s.is_some()),
        "status icon must render for the word"
    );
}

#[wasm_bindgen_test]
async fn set_word_item_click_row_toggles_selection() {
    let wrapper = create_wrapper();
    let (set_toggled, get_toggled) = shared_cell::<RwSignal<bool>>();
    mount_with_i18n(&wrapper, move || {
        let toggled = RwSignal::new(false);
        set_toggled.set(Some(toggled));
        let selected_words: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
        view! {
            <SetWordItem
                word="いぬ".to_string()
                known_meaning=None
                outcome=origa::domain::WordImportOutcome::New
                selected_words=selected_words
                known_kanji=HashSet::new()
                on_toggle=Callback::new(move |()| toggled.set(true))
            />
        }
        .into_any()
    });
    let toggled = get_toggled.get().expect("captured");
    tick().await;

    wrapper
        .query_selector("[data-testid=\"sets-drawer-item\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert!(toggled.get(), "row click must invoke on_toggle");
}

#[wasm_bindgen_test]
async fn set_word_item_without_dictionary_entry_does_not_toggle() {
    let wrapper = create_wrapper();
    let (set_toggled, get_toggled) = shared_cell::<RwSignal<bool>>();
    mount_with_i18n(&wrapper, move || {
        let toggled = RwSignal::new(false);
        set_toggled.set(Some(toggled));
        let selected_words: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
        view! {
            <SetWordItem
                word="は".to_string()
                known_meaning=None
                outcome=origa::domain::WordImportOutcome::NoDictionaryEntry
                selected_words=selected_words
                known_kanji=HashSet::new()
                on_toggle=Callback::new(move |()| toggled.set(true))
            />
        }
        .into_any()
    });
    let toggled = get_toggled.get().expect("captured");
    tick().await;

    wrapper
        .query_selector("[data-testid=\"sets-drawer-item\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert!(
        !toggled.get(),
        "a word without a dictionary entry must not be selectable"
    );
}

#[wasm_bindgen_test]
async fn set_word_item_known_meaning_rendered() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let selected_words: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
        view! {
            <SetWordItem
                word="ねこ".to_string()
                known_meaning=Some("кошка".to_string())
                outcome=origa::domain::WordImportOutcome::New
                selected_words=selected_words
                known_kanji=HashSet::new()
                on_toggle=Callback::new(|()| ())
            />
        }
        .into_any()
    });
    tick().await;

    let text = wrapper
        .query_selector("[data-testid=\"sets-drawer-item\"]")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(text.contains("кошка"), "meaning must render; got: {text}");
}

// ═══════════════════════════════════════════════════════════════════════
// SetsHeader (Router-based PageHeader)
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn sets_header_renders_title() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router(&wrapper, || view! { <SetsHeader /> }.into_any());
    tick().await;

    let header = wrapper
        .query_selector("[data-testid=\"sets-header\"]")
        .ok()
        .flatten()
        .expect("sets header root must render");
    assert!(!header.text_content().unwrap().is_empty());
}
