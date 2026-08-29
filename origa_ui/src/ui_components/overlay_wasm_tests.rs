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
    let (set_open, get_open) = shared_cell::<RwSignal<bool>>();
    mount_to_wrapper(&wrapper, move || {
        let is_open = RwSignal::new(true);
        set_open.set(Some(is_open));
        view! {
            <Drawer is_open=is_open title=Signal::derive(|| "Filters".to_string()) test_id="dr2">
                "drawer content"
            </Drawer>
        }
        .into_any()
    });
    let is_open = get_open.get().expect("captured");
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

    // The mount leaks its owner, so the scroll lock would stay held for the
    // rest of the test run — close the drawer to release it.
    is_open.set(false);
    tick().await;
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

// The footer slot replaces the old in-scroll sticky bar (which painted a
// "white rectangle" behind the buttons and left an 8px scrollbar-gutter
// gap on its right). The footer must live OUTSIDE the scrollable body and
// carry no background of its own.
#[wasm_bindgen_test]
async fn drawer_footer_slot_renders_outside_scroll_body() {
    let wrapper = create_wrapper();
    let (set_open, get_open) = shared_cell::<RwSignal<bool>>();
    mount_to_wrapper(&wrapper, move || {
        let is_open = RwSignal::new(true);
        set_open.set(Some(is_open));
        view! {
            <Drawer
                is_open=is_open
                title=Signal::derive(|| "T".to_string())
                footer=std::sync::Arc::new(move || {
                    view! { <button data-testid="dr-footer-act">"Do it"</button> }
                        .into_any()
                })
                test_id="drf"
            >
                "b"
            </Drawer>
        }
        .into_any()
    });
    let is_open = get_open.get().expect("captured");
    tick().await;

    let footer = wrapper
        .query_selector(".drawer-footer")
        .unwrap()
        .expect("footer slot must render .drawer-footer");
    assert!(
        footer.text_content().unwrap().contains("Do it"),
        "footer slot must render its children"
    );
    // The footer is a sibling of .drawer-body (outside the scroll area),
    // not a child of it.
    let in_body = footer
        .closest(".drawer-body")
        .expect("closest() must not throw");
    assert!(
        in_body.is_none(),
        "footer must not live inside the scrollable body"
    );
    // No own background: the old sticky bar carried bg-[var(--bg-paper)].
    let bg = footer
        .dyn_ref::<web_sys::HtmlElement>()
        .expect("footer is an element");
    let style = web_sys::window()
        .expect("window")
        .get_computed_style(bg)
        .expect("getComputedStyle call")
        .expect("computed style available");
    let background = style
        .get_property_value("background-color")
        .unwrap_or_default();
    assert!(
        background == "rgba(0, 0, 0, 0)",
        "footer must have no background (got {background})"
    );

    is_open.set(false);
    tick().await;
}

// Drawers without the footer prop must render exactly as before — no
// empty .drawer-footer wrapper (and no border-top rule) for them.
#[wasm_bindgen_test]
async fn drawer_without_footer_renders_no_footer() {
    let wrapper = create_wrapper();
    let (set_open, get_open) = shared_cell::<RwSignal<bool>>();
    mount_to_wrapper(&wrapper, move || {
        let is_open = RwSignal::new(true);
        set_open.set(Some(is_open));
        view! {
            <Drawer is_open=is_open title=Signal::derive(|| "T".to_string()) test_id="dr-nf">"b"</Drawer>
        }
        .into_any()
    });
    let is_open = get_open.get().expect("captured");
    tick().await;

    let footer = wrapper.query_selector(".drawer-footer");
    assert!(
        footer.is_ok_and(|f| f.is_none()),
        "no footer prop — no .drawer-footer element"
    );

    is_open.set(false);
    tick().await;
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

    // The mount leaks its owner, so the scroll lock would stay held for the
    // rest of the test run — close the drawer to release it.
    is_open.set(false);
    tick().await;
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

#[wasm_bindgen_test]
async fn drawer_open_locks_body_scroll_and_close_restores_it() {
    crate::utils::scroll_lock::reset_for_tests();
    let wrapper = create_wrapper();
    let (set_open, get_open) = shared_cell::<RwSignal<bool>>();
    mount_to_wrapper(&wrapper, move || {
        let is_open = RwSignal::new(false);
        set_open.set(Some(is_open));
        view! {
            <Drawer is_open=is_open title=Signal::derive(|| "T".to_string()) test_id="dr6">"b"</Drawer>
        }
        .into_any()
    });
    let is_open = get_open.get().expect("captured");
    tick().await;

    let body = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap();
    assert_ne!(
        body.style().get_property_value("position").unwrap(),
        "fixed",
        "closed drawer must not lock the body"
    );

    // Act: open → locked
    is_open.set(true);
    tick().await;
    assert_eq!(
        body.style().get_property_value("position").unwrap(),
        "fixed",
        "open drawer must lock the body scroll"
    );

    // Act: close → restored
    is_open.set(false);
    tick().await;
    assert_ne!(
        body.style().get_property_value("position").unwrap(),
        "fixed",
        "closed drawer must restore the body scroll"
    );
}

#[wasm_bindgen_test]
async fn nested_drawers_keep_body_locked_until_the_last_one_closes() {
    crate::utils::scroll_lock::reset_for_tests();
    let wrapper = create_wrapper();
    let (set_outer, get_outer) = shared_cell::<RwSignal<bool>>();
    let (set_inner, get_inner) = shared_cell::<RwSignal<bool>>();
    mount_to_wrapper(&wrapper, move || {
        let outer = RwSignal::new(false);
        set_outer.set(Some(outer));
        let inner = RwSignal::new(false);
        set_inner.set(Some(inner));
        view! {
            <Drawer is_open=outer title=Signal::derive(|| "O".to_string()) test_id="dr7o">"o"</Drawer>
            <Drawer is_open=inner title=Signal::derive(|| "I".to_string()) test_id="dr7i">"i"</Drawer>
        }
        .into_any()
    });
    let outer = get_outer.get().expect("captured");
    let inner = get_inner.get().expect("captured");
    tick().await;

    let body = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap();

    outer.set(true);
    inner.set(true);
    tick().await;
    assert_eq!(
        body.style().get_property_value("position").unwrap(),
        "fixed",
        "two open drawers must lock the body"
    );

    // Close one — the other still holds the lock.
    inner.set(false);
    tick().await;
    assert_eq!(
        body.style().get_property_value("position").unwrap(),
        "fixed",
        "body must stay locked while one drawer is still open"
    );

    outer.set(false);
    tick().await;
    assert_ne!(
        body.style().get_property_value("position").unwrap(),
        "fixed",
        "closing the last drawer must restore the body"
    );
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
