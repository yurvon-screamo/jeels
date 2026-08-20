//! WASM render tests for `pages/shared` components: `LoadMoreButton`,
//! `MarkAsKnownButton`, `DailyLoadSelector`, `DailyLoadList`, `GroupedGrid`.

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

use crate::pages::shared::{
    DailyLoadList, DailyLoadSelector, GroupedGrid, LoadMoreButton, MarkAsKnownButton,
};
use crate::test_support::{create_wrapper, mount_with_i18n, shared_cell};

wasm_bindgen_test_configure!(run_in_browser);

// ═══════════════════════════════════════════════════════════════════════
// LoadMoreButton
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn load_more_button_hidden_when_all_visible() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let visible_count = RwSignal::new(50usize);
        view! {
            <LoadMoreButton visible_count=visible_count total=Signal::from(50usize) test_id="lm1" />
        }
        .into_any()
    });
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"lm1\"]")
            .unwrap()
            .is_none(),
        "button must hide when everything is visible"
    );
}

#[wasm_bindgen_test]
async fn load_more_button_click_increases_by_page_size() {
    let wrapper = create_wrapper();
    let (set_count, get_count) = shared_cell::<RwSignal<usize>>();
    mount_with_i18n(&wrapper, move || {
        let visible_count = RwSignal::new(50usize);
        set_count.set(Some(visible_count));
        view! {
            <LoadMoreButton visible_count=visible_count total=Signal::from(130usize) test_id="lm2" />
        }
        .into_any()
    });
    let visible_count = get_count.get().expect("captured");
    tick().await;

    // Act
    wrapper
        .query_selector("[data-testid=\"lm2\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    // Assert: 50 + 50 = 100, capped below the total 130
    assert_eq!(visible_count.get(), 100);
}

#[wasm_bindgen_test]
async fn load_more_button_click_caps_at_total() {
    let wrapper = create_wrapper();
    let (set_count, get_count) = shared_cell::<RwSignal<usize>>();
    mount_with_i18n(&wrapper, move || {
        let visible_count = RwSignal::new(100usize);
        set_count.set(Some(visible_count));
        view! {
            <LoadMoreButton visible_count=visible_count total=Signal::from(120usize) test_id="lm3" />
        }
        .into_any()
    });
    let visible_count = get_count.get().expect("captured");
    tick().await;

    wrapper
        .query_selector("[data-testid=\"lm3\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert_eq!(visible_count.get(), 120, "must cap at the total");
}

#[wasm_bindgen_test]
async fn load_more_button_label_shows_remaining_count() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let visible_count = RwSignal::new(10usize);
        view! {
            <LoadMoreButton visible_count=visible_count total=Signal::from(35usize) test_id="lm4" />
        }
        .into_any()
    });
    tick().await;

    let text = wrapper
        .query_selector("[data-testid=\"lm4\"]")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(
        text.contains("25"),
        "remaining count must be shown; got: {text}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// MarkAsKnownButton
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn mark_as_known_button_idle_renders_check_icon() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! { <MarkAsKnownButton on_click=Callback::new(|()| {}) test_id="mk1" /> }.into_any()
    });
    tick().await;

    let btn = wrapper
        .query_selector("[data-testid=\"mk1\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlButtonElement>();
    assert!(!btn.disabled(), "idle button must be enabled");

    let svg = btn.query_selector("svg");
    assert!(svg.is_ok_and(|s| s.is_some()), "check icon must render");
}

#[wasm_bindgen_test]
async fn mark_as_known_button_pending_shows_spinner_and_disables() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <MarkAsKnownButton
                on_click=Callback::new(|()| {})
                pending=Signal::from(true)
                test_id="mk2"
            />
        }
        .into_any()
    });
    tick().await;

    let btn = wrapper
        .query_selector("[data-testid=\"mk2\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlButtonElement>();
    assert!(btn.disabled(), "pending button must be disabled");

    let spinner = btn.query_selector(".spinner");
    assert!(
        spinner.is_ok_and(|s| s.is_some()),
        "pending state must show a spinner"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// DailyLoadSelector
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn daily_load_selector_renders_six_options() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let selected = RwSignal::new(origa::domain::DailyLoad::Medium);
        view! { <DailyLoadSelector selected_load=selected /> }.into_any()
    });
    tick().await;

    let count = wrapper.query_selector_all("button.btn").unwrap().length();
    assert_eq!(count, 6, "six daily-load options must render; got {count}");
}

#[wasm_bindgen_test]
async fn daily_load_selector_click_switches_selection() {
    let wrapper = create_wrapper();
    let (set_load, get_load) = shared_cell::<RwSignal<origa::domain::DailyLoad>>();
    mount_with_i18n(&wrapper, move || {
        let selected = RwSignal::new(origa::domain::DailyLoad::Minimal);
        set_load.set(Some(selected));
        view! { <DailyLoadSelector selected_load=selected /> }.into_any()
    });
    let selected = get_load.get().expect("captured");
    tick().await;

    // Act: click the "heavy" option (test id derived from Debug, lowercased)
    wrapper
        .query_selector("[data-testid=\"profile-load-heavy\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert_eq!(
        selected.get(),
        origa::domain::DailyLoad::Heavy,
        "click must switch the selected load"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// DailyLoadList
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn daily_load_list_selected_option_marked() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let selected = RwSignal::new(origa::domain::DailyLoad::Medium);
        view! { <DailyLoadList selected_load=selected /> }.into_any()
    });
    tick().await;

    let medium = wrapper
        .query_selector("[data-testid=\"daily-load-option-medium\"]")
        .unwrap()
        .unwrap();
    let class = medium.get_attribute("class").unwrap_or_default();
    assert!(class.contains("selected"), "got: {class}");

    let minimal = wrapper
        .query_selector("[data-testid=\"daily-load-option-minimal\"]")
        .unwrap()
        .unwrap();
    let class = minimal.get_attribute("class").unwrap_or_default();
    assert!(!class.contains("selected"), "got: {class}");
}

#[wasm_bindgen_test]
async fn daily_load_list_click_switches_selection() {
    let wrapper = create_wrapper();
    let (set_load, get_load) = shared_cell::<RwSignal<origa::domain::DailyLoad>>();
    mount_with_i18n(&wrapper, move || {
        let selected = RwSignal::new(origa::domain::DailyLoad::Light);
        set_load.set(Some(selected));
        view! { <DailyLoadList selected_load=selected /> }.into_any()
    });
    let selected = get_load.get().expect("captured");
    tick().await;

    wrapper
        .query_selector("[data-testid=\"daily-load-option-maximum\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert_eq!(
        selected.get(),
        origa::domain::DailyLoad::Maximum,
        "option click must switch the signal"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// GroupedGrid
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn grouped_grid_renders_cards_into_level_groups_in_order() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        use crate::test_support::vocabulary_study_card;

        // Two cards bucketed into different levels (N5 first, N4 second).
        let n5 = vocabulary_study_card("ねこ");
        let n4 = vocabulary_study_card("いぬ");
        let cards = vec![n5.clone(), n4.clone()];

        let mut index = std::collections::HashMap::new();
        index.insert(*n5.card_id(), Some(origa::domain::JapaneseLevel::N5));
        index.insert(*n4.card_id(), Some(origa::domain::JapaneseLevel::N4));

        let cards = Memo::new(move |_| cards.clone());
        let level_index = Memo::new(move |_| index.clone());

        view! {
            <GroupedGrid
                cards=cards
                level_index=level_index
                grid_classes="test-grid"
                test_id_prefix="gg"
                render_card=|card: origa::domain::StudyCard| {
                    let id = card.card_id().to_string();
                    view! { <div class="stub-card" data-testid=format!("gg-card-{}", id)></div> }
                        .into_any()
                }
            />
        }
        .into_any()
    });
    tick().await;

    // Both cards rendered through the stub renderer
    let stubs = wrapper.query_selector_all(".stub-card").unwrap().length();
    assert_eq!(stubs, 2, "both cards must render; got {stubs}");

    // The N5 group section must come before the N4 section (GROUP_ORDER)
    let n5_section = wrapper
        .query_selector("[data-testid=\"gg-grid-N5\"]")
        .ok()
        .flatten()
        .expect("N5 group section must render");
    let n4_section = wrapper
        .query_selector("[data-testid=\"gg-grid-N4\"]")
        .ok()
        .flatten()
        .expect("N4 group section must render");
    assert!(
        n5_section.compare_document_position(&n4_section)
            & web_sys::Node::DOCUMENT_POSITION_FOLLOWING
            != 0,
        "N5 section must precede N4"
    );
}

#[wasm_bindgen_test]
async fn grouped_grid_empty_groups_hidden() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        use crate::test_support::vocabulary_study_card;

        let card = vocabulary_study_card("ねこ");
        let mut index = std::collections::HashMap::new();
        index.insert(*card.card_id(), Some(origa::domain::JapaneseLevel::N1));

        let cards = Memo::new(move |_| vec![card.clone()]);
        let level_index = Memo::new(move |_| index.clone());

        view! {
            <GroupedGrid
                cards=cards
                level_index=level_index
                grid_classes="test-grid"
                test_id_prefix="ge"
                render_card=|_card: origa::domain::StudyCard| {
                    view! { <div class="stub-card"></div> }.into_any()
                }
            />
        }
        .into_any()
    });
    tick().await;

    let stubs = wrapper.query_selector_all(".stub-card").unwrap().length();
    assert_eq!(stubs, 1);

    // N5..N4..N3..N2 sections stay hidden when their buckets are empty
    for level in ["N5", "N4", "N3", "N2"] {
        let section = wrapper.query_selector(&format!("[data-testid=\"ge-grid-{level}\"]"));
        assert!(
            section.is_ok_and(|s| s.is_none()),
            "empty group {level} must be hidden"
        );
    }
    assert!(
        wrapper
            .query_selector("[data-testid=\"ge-grid-N1\"]")
            .unwrap()
            .is_some(),
        "the populated N1 group must render"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// part_of_speech_label (i18n probe)
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn pos_label_every_variant_yields_nonempty_string() {
    use crate::pages::lesson::pos_label::part_of_speech_label;
    use origa::domain::PartOfSpeech;

    let wrapper = create_wrapper();
    let variants = [
        PartOfSpeech::Verb,
        PartOfSpeech::Noun,
        PartOfSpeech::IAdjective,
        PartOfSpeech::NaAdjective,
        PartOfSpeech::Adverb,
        PartOfSpeech::Particle,
        PartOfSpeech::AuxiliaryVerb,
        PartOfSpeech::Pronoun,
        PartOfSpeech::ProperNoun,
        PartOfSpeech::Numeral,
        PartOfSpeech::Unspecified,
    ];
    let out = std::rc::Rc::new(std::cell::RefCell::new(Vec::<String>::new()));
    let sink = out.clone();
    mount_with_i18n(&wrapper, move || {
        let i18n = crate::i18n::use_i18n();
        let mut acc = sink.borrow_mut();
        for pos in variants {
            acc.push(part_of_speech_label(pos, &i18n));
        }
        drop(acc);
        view! { <div></div> }.into_any()
    });
    tick().await;

    let labels = out.borrow();
    assert_eq!(labels.len(), 11);
    assert!(
        labels.iter().all(|l| !l.is_empty()),
        "every POS variant must have a localized label; got: {labels:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// content_sync toast helpers (i18n probe)
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn sync_toast_helpers_push_expected_toast_types() {
    use crate::pages::home::content_sync::{
        show_sync_error_toast, show_sync_success_toast, show_sync_toast,
    };
    use crate::ui_components::ToastType;

    let wrapper = create_wrapper();
    let (set_state, get_state) = shared_cell::<(
        RwSignal<Vec<crate::ui_components::ToastData>>,
        crate::i18n::I18nContext<crate::i18n::Locale>,
    )>();
    mount_with_i18n(&wrapper, move || {
        // Both the signal and the i18n context must be created inside the
        // reactive scope; capture both out for the assertions.
        let toasts: RwSignal<Vec<crate::ui_components::ToastData>> = RwSignal::new(Vec::new());
        let i18n = crate::i18n::use_i18n();
        set_state.set(Some((toasts, i18n)));
        view! { <div></div> }.into_any()
    });
    let (toasts, i18n) = get_state.get().expect("captured");
    tick().await;

    // Act + Assert: running sync → info toast with the sentinel id
    show_sync_toast(toasts, i18n);
    tick().await;
    assert_eq!(toasts.get().len(), 1);
    assert_eq!(toasts.get()[0].toast_type, ToastType::Info);
    assert!(!toasts.get()[0].closable, "sync toast must not be closable");

    // Success replaces the sentinel with a closable success toast
    show_sync_success_toast(toasts, i18n);
    tick().await;
    let list = toasts.get();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].toast_type, ToastType::Success);
    assert!(list[0].closable);

    // Error removes the sync sentinel and pushes an error toast; the earlier
    // success toast stays (it is closable, dismissed by the user).
    let err = origa::domain::OrigaError::CardNotFound {
        card_id: ulid::Ulid::new(),
    };
    show_sync_error_toast(toasts, i18n, &err);
    tick().await;
    let list = toasts.get();
    assert_eq!(list.len(), 2);
    assert_eq!(list[1].toast_type, ToastType::Error);
}
