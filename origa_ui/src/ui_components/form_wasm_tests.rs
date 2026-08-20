//! WASM render tests for form components: `Search`, `Input`, `Tabs`.

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

use crate::test_support::{create_wrapper, mount_to_wrapper, shared_cell};
use crate::ui_components::{Input, Search, TabItem, Tabs};

wasm_bindgen_test_configure!(run_in_browser);

// ═══════════════════════════════════════════════════════════════════════
// Search
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn search_renders_input_with_placeholder() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Search placeholder=Signal::derive(|| "Find a word…".to_string()) test_id="srch1" />
        }
        .into_any()
    });
    tick().await;

    let input = wrapper
        .query_selector("[data-testid=\"srch1-input\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>();
    assert_eq!(
        input.placeholder(),
        "Find a word…",
        "search input must carry the placeholder"
    );
}

#[wasm_bindgen_test]
async fn search_container_has_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || view! { <Search test_id="srch2" /> }.into_any());
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"srch2\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("search-container"), "got: {class}");
}

#[wasm_bindgen_test]
async fn search_icon_present() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || view! { <Search test_id="srch3" /> }.into_any());
    tick().await;

    let icon = wrapper.query_selector(".search-icon");
    assert!(
        icon.is_ok_and(|i| i.is_some()),
        "search must render its magnifier icon"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Input
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn input_defaults_to_text_type() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || view! { <Input test_id="in1" /> }.into_any());
    tick().await;

    let input = wrapper
        .query_selector("[data-testid=\"in1\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>();
    assert_eq!(input.type_(), "text", "default input type must be text");
}

#[wasm_bindgen_test]
async fn input_custom_type_applied() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Input input_type=Signal::derive(|| "email".to_string()) test_id="in2" />
        }
        .into_any()
    });
    tick().await;

    let input = wrapper
        .query_selector("[data-testid=\"in2\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>();
    assert_eq!(input.type_(), "email");
}

#[wasm_bindgen_test]
async fn input_disabled_state() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Input disabled=Signal::from(true) test_id="in3" /> }.into_any()
    });
    tick().await;

    let input = wrapper
        .query_selector("[data-testid=\"in3\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>();
    assert!(input.disabled(), "input must be disabled");
}

#[wasm_bindgen_test]
async fn input_with_rows_renders_textarea() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Input rows=Signal::derive(|| Some(4usize)) test_id="in4" /> }.into_any()
    });
    tick().await;

    // The test_id lands on the <textarea> itself (Input renders it directly).
    let textarea = wrapper
        .query_selector("textarea[data-testid=\"in4\"]")
        .ok()
        .flatten()
        .expect("input with rows must render a <textarea>");

    let ta = textarea.unchecked_into::<web_sys::HtmlTextAreaElement>();
    assert_eq!(ta.rows(), 4, "textarea must carry the row count");
}

#[wasm_bindgen_test]
async fn input_without_rows_renders_input() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || view! { <Input test_id="in5" /> }.into_any());
    tick().await;

    let input = wrapper.query_selector("input[data-testid=\"in5\"]");
    assert!(
        input.is_ok_and(|i| i.is_some()),
        "input without rows must render an <input>"
    );
}

#[wasm_bindgen_test]
async fn input_autocomplete_defaults_off() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || view! { <Input test_id="in6" /> }.into_any());
    tick().await;

    let input = wrapper
        .query_selector("[data-testid=\"in6\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>();
    assert_eq!(
        input.autocomplete(),
        "off",
        "default autocomplete must be off"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Tabs
// ═══════════════════════════════════════════════════════════════════════

fn two_tabs() -> Vec<TabItem> {
    vec![
        TabItem {
            id: "all".into(),
            label: "All".into(),
        },
        TabItem {
            id: "fav".into(),
            label: "Favorites".into(),
        },
    ]
}

#[wasm_bindgen_test]
async fn tabs_render_all_buttons() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let tabs = two_tabs();
        let active = RwSignal::new("all".to_string());
        view! {
            <Tabs tabs=Signal::derive(move || tabs.clone()) active=active test_id="tb1" />
        }
        .into_any()
    });
    tick().await;

    let count = wrapper.query_selector_all(".tab").unwrap().length();
    assert_eq!(count, 2, "two tabs configured; got {count}");
}

#[wasm_bindgen_test]
async fn tabs_active_tab_has_active_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let tabs = two_tabs();
        let active = RwSignal::new("fav".to_string());
        view! {
            <Tabs tabs=Signal::derive(move || tabs.clone()) active=active test_id="tb2" />
        }
        .into_any()
    });
    tick().await;

    let fav = wrapper
        .query_selector("[data-testid=\"tb2-fav\"]")
        .unwrap()
        .unwrap();
    let class = fav.get_attribute("class").unwrap_or_default();
    assert!(class.contains("active"), "got: {class}");

    let all = wrapper
        .query_selector("[data-testid=\"tb2-all\"]")
        .unwrap()
        .unwrap();
    let class = all.get_attribute("class").unwrap_or_default();
    assert!(!class.contains("active"), "got: {class}");
}

#[wasm_bindgen_test]
async fn tabs_click_switches_active() {
    let wrapper = create_wrapper();
    let (set_active, get_active) = shared_cell::<RwSignal<String>>();
    mount_to_wrapper(&wrapper, move || {
        let tabs = two_tabs();
        let active = RwSignal::new("all".to_string());
        set_active.set(Some(active));
        view! {
            <Tabs tabs=Signal::derive(move || tabs.clone()) active=active test_id="tb3" />
        }
        .into_any()
    });
    let active = get_active.get().expect("captured");
    tick().await;

    // Act: click the "Favorites" tab
    let fav = wrapper
        .query_selector("[data-testid=\"tb3-fav\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>();
    fav.click();
    tick().await;

    // Assert: active switched to the clicked tab
    assert_eq!(active.get(), "fav", "click must set the active tab id");
    let fav_class = fav.get_attribute("class").unwrap_or_default();
    assert!(fav_class.contains("active"), "got: {fav_class}");
}

#[wasm_bindgen_test]
async fn tabs_render_labels() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let tabs = two_tabs();
        let active = RwSignal::new("all".to_string());
        view! {
            <Tabs tabs=Signal::derive(move || tabs.clone()) active=active test_id="tb4" />
        }
        .into_any()
    });
    tick().await;

    let text = wrapper
        .query_selector("[data-testid=\"tb4\"]")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(text.contains("All"), "got: {text}");
    assert!(text.contains("Favorites"), "got: {text}");
}
