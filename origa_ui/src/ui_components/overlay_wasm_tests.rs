//! WASM render tests for overlay components: `Drawer`, `Modal`.
//!
//! Both are conditionally rendered via `Show when=is_open` — the tests
//! drive the signal through [`crate::test_support::shared_cell`] to verify
//! open/close reactivity. `Modal` closes through a 250 ms animation timer;
//! closing asserts poll for the unmounted state via
//! [`crate::test_support::wait_until`].

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

use crate::test_support::{create_wrapper, mount_to_wrapper, shared_cell};
use crate::ui_components::{Drawer, Modal};

wasm_bindgen_test_configure!(run_in_browser);

/// Wait for the modal close animation (250 ms) to settle. Polls for the
/// unmounted state instead of a fixed sleep (CI-flakiness guard).
async fn wait_close_animation(wrapper: &web_sys::Element, test_id: &str) {
    let selector = format!("[data-testid=\"{test_id}\"]");
    let gone = crate::test_support::wait_until(
        move || wrapper.query_selector(&selector).unwrap().is_none(),
        20,
        25,
    )
    .await;
    assert!(gone, "modal must unmount after the close animation");
}

// ═══════════════════════════════════════════════════════════════════════
// Drawer
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn drawer_closed_renders_nothing() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let is_open = RwSignal::new(false);
        view! {
            <Drawer is_open=is_open title=Signal::derive(|| "Menu".to_string()) test_id="dr1">
                "drawer body"
            </Drawer>
        }
        .into_any()
    });
    tick().await;

    let el = wrapper.query_selector("[data-testid=\"dr1\"]");
    assert!(
        el.is_ok_and(|e| e.is_none()),
        "closed drawer must render nothing"
    );
}

#[wasm_bindgen_test]
async fn drawer_open_renders_title_body_and_backdrop() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let is_open = RwSignal::new(true);
        view! {
            <Drawer is_open=is_open title=Signal::derive(|| "Filters".to_string()) test_id="dr2">
                "drawer content"
            </Drawer>
        }
        .into_any()
    });
    tick().await;

    let drawer = wrapper
        .query_selector("[data-testid=\"dr2\"]")
        .unwrap()
        .unwrap();
    assert!(
        drawer.text_content().unwrap().contains("Filters"),
        "drawer must show its title"
    );
    assert!(
        drawer.text_content().unwrap().contains("drawer content"),
        "drawer must show children"
    );

    let backdrop = wrapper.query_selector("[data-testid=\"dr2-backdrop\"]");
    assert!(
        backdrop.is_ok_and(|b| b.is_some()),
        "open drawer must render backdrop"
    );

    let close = wrapper.query_selector("[data-testid=\"dr2-close\"]");
    assert!(
        close.is_ok_and(|c| c.is_some()),
        "open drawer must render close button"
    );
}

#[wasm_bindgen_test]
async fn drawer_close_button_sets_closed() {
    let wrapper = create_wrapper();
    let (set_open, get_open) = shared_cell::<RwSignal<bool>>();
    mount_to_wrapper(&wrapper, move || {
        let is_open = RwSignal::new(true);
        set_open.set(Some(is_open));
        view! {
            <Drawer is_open=is_open title=Signal::derive(|| "T".to_string()) test_id="dr3">"b"</Drawer>
        }
        .into_any()
    });
    let is_open = get_open.get().expect("captured");
    tick().await;

    // Act
    wrapper
        .query_selector("[data-testid=\"dr3-close\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    // Assert
    assert!(!is_open.get(), "close click must set is_open=false");
    let el = wrapper.query_selector("[data-testid=\"dr3\"]");
    assert!(
        el.is_ok_and(|e| e.is_none()),
        "drawer must unmount after close"
    );
}

#[wasm_bindgen_test]
async fn drawer_reactive_open_mounts_content() {
    let wrapper = create_wrapper();
    let (set_open, get_open) = shared_cell::<RwSignal<bool>>();
    mount_to_wrapper(&wrapper, move || {
        let is_open = RwSignal::new(false);
        set_open.set(Some(is_open));
        view! {
            <Drawer is_open=is_open title=Signal::derive(|| "T".to_string()) test_id="dr4">"b"</Drawer>
        }
        .into_any()
    });
    let is_open = get_open.get().expect("captured");
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"dr4\"]")
            .unwrap()
            .is_none(),
        "initially closed"
    );

    // Act
    is_open.set(true);
    tick().await;

    // Assert
    assert!(
        wrapper
            .query_selector("[data-testid=\"dr4\"]")
            .unwrap()
            .is_some(),
        "setting is_open=true must mount the drawer"
    );
}

#[wasm_bindgen_test]
async fn drawer_escape_key_closes() {
    let wrapper = create_wrapper();
    let (set_open, get_open) = shared_cell::<RwSignal<bool>>();
    mount_to_wrapper(&wrapper, move || {
        let is_open = RwSignal::new(true);
        set_open.set(Some(is_open));
        view! {
            <Drawer is_open=is_open title=Signal::derive(|| "T".to_string()) test_id="dr5">"b"</Drawer>
        }
        .into_any()
    });
    let is_open = get_open.get().expect("captured");
    tick().await;

    // Act: dispatch Escape on document
    let init = web_sys::KeyboardEventInit::new();
    init.set_key("Escape");
    let escape =
        web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap();
    let _ = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .dispatch_event(&escape);
    tick().await;

    assert!(!is_open.get(), "Escape must close the drawer");
}

// ═══════════════════════════════════════════════════════════════════════
// Modal
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn modal_closed_renders_nothing() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let is_open = RwSignal::new(false);
        view! {
            <Modal is_open=is_open title=Signal::derive(|| "Confirm".to_string()) test_id="md1">
                "modal body"
            </Modal>
        }
        .into_any()
    });
    tick().await;

    let el = wrapper.query_selector("[data-testid=\"md1\"]");
    assert!(
        el.is_ok_and(|e| e.is_none()),
        "closed modal must render nothing"
    );
}

#[wasm_bindgen_test]
async fn modal_open_renders_content_backdrop_and_close() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let is_open = RwSignal::new(true);
        view! {
            <Modal is_open=is_open title=Signal::derive(|| "Confirm".to_string()) test_id="md2">
                "modal body"
            </Modal>
        }
        .into_any()
    });
    tick().await;

    let modal = wrapper
        .query_selector("[data-testid=\"md2\"]")
        .unwrap()
        .unwrap();
    let text = modal.text_content().unwrap();
    assert!(text.contains("Confirm"), "got: {text}");
    assert!(text.contains("modal body"), "got: {text}");

    let class = modal.get_attribute("class").unwrap_or_default();
    assert!(class.contains("modal-content"), "got: {class}");
    assert!(class.contains("anima-modal-enter"), "got: {class}");

    assert!(
        wrapper
            .query_selector("[data-testid=\"md2-backdrop\"]")
            .unwrap()
            .is_some(),
        "open modal must render backdrop"
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"md2-close\"]")
            .unwrap()
            .is_some(),
        "open modal must render close button"
    );
}

#[wasm_bindgen_test]
async fn modal_close_button_closes_after_animation() {
    let wrapper = create_wrapper();
    let (set_open, get_open) = shared_cell::<RwSignal<bool>>();
    mount_to_wrapper(&wrapper, move || {
        let is_open = RwSignal::new(true);
        set_open.set(Some(is_open));
        view! {
            <Modal is_open=is_open title=Signal::derive(|| "T".to_string()) test_id="md3">"b"</Modal>
        }
        .into_any()
    });
    let is_open = get_open.get().expect("captured");
    tick().await;

    // Act
    wrapper
        .query_selector("[data-testid=\"md3-close\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    // Exit animation class applies immediately…
    let class = wrapper
        .query_selector("[data-testid=\"md3\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("anima-modal-exit"), "got: {class}");

    // …and after the animation timer the modal unmounts.
    wait_close_animation(&wrapper, "md3").await;
    tick().await;

    assert!(!is_open.get(), "close click must set is_open=false");
}
