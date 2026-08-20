//! WASM render tests for components that need a real `<Router>`:
//! `PageHeader` (`use_navigate` → `RouterContext`, which is `pub(crate)`
//! in leptos_router 0.8, so a real router must wrap the mount).
//!
//! The mounts are disposable: the router's global history listeners are
//! removed when the handle drops at the end of each test.

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

use crate::test_support::{create_wrapper, mount_with_router};
use crate::ui_components::PageHeader;

wasm_bindgen_test_configure!(run_in_browser);

// ═══════════════════════════════════════════════════════════════════════
// PageHeader
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn page_header_without_back_path_hides_back_button() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router(&wrapper, || {
        view! {
            <PageHeader title=Signal::derive(|| "Words".to_string()) test_id="ph1" />
        }
        .into_any()
    });
    tick().await;

    let back = wrapper.query_selector("[data-testid=\"ph1-back-btn\"]");
    assert!(
        back.is_ok_and(|b| b.is_none()),
        "no back_path → back button hidden"
    );

    let title = wrapper
        .query_selector("[data-testid=\"ph1-title\"]")
        .ok()
        .flatten()
        .expect("title must render when provided");
    let text = title.text_content().unwrap();
    assert!(text.contains("Words"), "got: {text}");
}

#[wasm_bindgen_test]
async fn page_header_with_back_path_shows_back_button() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router(&wrapper, || {
        view! {
            <PageHeader
                back_path=Signal::derive(|| "/home".to_string())
                back_label=Signal::derive(|| "Back".to_string())
                title=Signal::derive(|| "Detail".to_string())
                test_id="ph2"
            />
        }
        .into_any()
    });
    tick().await;

    let back = wrapper
        .query_selector("[data-testid=\"ph2-back-btn\"]")
        .unwrap()
        .unwrap();
    let text = back.text_content().unwrap();
    assert!(text.contains("Back"), "back label rendered; got: {text}");
}

#[wasm_bindgen_test]
async fn page_header_header_test_id_present() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router(&wrapper, || {
        view! {
            <PageHeader title=Signal::derive(|| "X".to_string()) test_id="ph3" />
        }
        .into_any()
    });
    tick().await;

    let header = wrapper.query_selector("[data-testid=\"ph3-header\"]");
    assert!(
        header.is_ok_and(|h| h.is_some()),
        "root element must carry ph3-header"
    );
}

#[wasm_bindgen_test]
async fn page_header_back_click_navigates() {
    let wrapper = create_wrapper();
    let original_path = web_sys::window()
        .unwrap()
        .location()
        .pathname()
        .unwrap_or_default();
    let _mount = mount_with_router(&wrapper, || {
        view! {
            <PageHeader
                back_path=Signal::derive(|| "/words".to_string())
                back_label=Signal::derive(|| "Back".to_string())
                test_id="ph4"
            />
        }
        .into_any()
    });
    tick().await;

    // Act
    wrapper
        .query_selector("[data-testid=\"ph4-back-btn\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    // Assert: the router followed the navigation
    let path = web_sys::window().unwrap().location().pathname().unwrap();
    assert!(
        path.ends_with("/words"),
        "back click must navigate to back_path; got: {path}"
    );

    // Cleanup: restore the deterministic starting pathname for later tests
    crate::test_support::restore_pathname(&original_path);
}

#[wasm_bindgen_test]
async fn page_header_children_render_in_actions_area() {
    let wrapper = create_wrapper();
    let _mount = mount_with_router(&wrapper, || {
        view! {
            <PageHeader title=Signal::derive(|| "List".to_string()) test_id="ph5">
                <button data-testid="ph5-extra-action">"Extra"</button>
            </PageHeader>
        }
        .into_any()
    });
    tick().await;

    let extra = wrapper.query_selector("[data-testid=\"ph5-extra-action\"]");
    assert!(
        extra.is_ok_and(|e| e.is_some()),
        "children must render in the actions area"
    );
}
