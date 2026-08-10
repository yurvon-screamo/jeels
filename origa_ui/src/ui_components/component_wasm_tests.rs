//! Component-level tests for UI components using `wasm_bindgen_test`.
//!
//! These tests mount real Leptos components into a headless browser DOM
//! and assert on the rendered HTML — catching bugs that `cargo test`
//! (no rendering) and BDD (testid-only assertions) miss.
//!
//! Run locally:
//! ```bash
//! wasm-pack test --headless --chrome origa_ui --features csr -- component_wasm_tests
//! ```

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

use crate::ui_components::{Tag, TagVariant};

wasm_bindgen_test_configure!(run_in_browser);

/// Creates a `<div>` appended to `<body>` for test isolation.
fn create_wrapper() -> web_sys::Element {
    console_error_panic_hook::set_once();
    let document = web_sys::window().unwrap().document().unwrap();
    let wrapper = document.create_element("div").unwrap();
    let _ = document.body().unwrap().append_child(&wrapper);
    wrapper
}

/// Mount a view closure into `wrapper`'s reactive scope.
/// The returned `UnmountHandle` keeps the component alive — drop it to dispose.
fn mount_to_wrapper<F>(wrapper: &web_sys::Element, f: F)
where
    F: FnOnce() -> AnyView + 'static,
{
    // `_dispose` is an UnmountHandle — must stay alive for the test's duration.
    // We leak it intentionally (the whole page is torn down after the test run).
    let _dispose = leptos::mount::mount_to(wrapper.clone().unchecked_into(), f);
    std::mem::forget(_dispose);
}

// ─── Tag: variant → CSS class mapping ──────────────────────────────────

#[wasm_bindgen_test]
async fn tag_default_variant_no_special_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Tag variant=TagVariant::Default>"N5"</Tag>
        }
        .into_any()
    });
    tick().await;

    let tag_el = wrapper.query_selector(".tag").unwrap().unwrap();
    let class = tag_el.get_attribute("class").unwrap_or_default();
    assert!(
        !class.contains("tag-filled") && !class.contains("tag-olive"),
        "default variant must not add special class; got: {class}"
    );
}

#[wasm_bindgen_test]
async fn tag_olive_variant_adds_tag_olive_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Tag variant=TagVariant::Olive>"Kanji"</Tag>
        }
        .into_any()
    });
    tick().await;

    let tag_el = wrapper.query_selector(".tag").unwrap().unwrap();
    let class = tag_el.get_attribute("class").unwrap_or_default();
    assert!(
        class.contains("tag-olive"),
        "olive variant must add tag-olive class; got: {class}"
    );
}

#[wasm_bindgen_test]
async fn tag_terracotta_variant_adds_tag_terracotta_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Tag variant=TagVariant::Terracotta>"Grammar"</Tag>
        }
        .into_any()
    });
    tick().await;

    let tag_el = wrapper.query_selector(".tag").unwrap().unwrap();
    let class = tag_el.get_attribute("class").unwrap_or_default();
    assert!(
        class.contains("tag-terracotta"),
        "terracotta variant must add tag-terracotta class; got: {class}"
    );
}

#[wasm_bindgen_test]
async fn tag_sage_variant_adds_tag_sage_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Tag variant=TagVariant::Sage>"Phrase"</Tag>
        }
        .into_any()
    });
    tick().await;

    let tag_el = wrapper.query_selector(".tag").unwrap().unwrap();
    let class = tag_el.get_attribute("class").unwrap_or_default();
    assert!(
        class.contains("tag-sage"),
        "sage variant must add tag-sage class; got: {class}"
    );
}

#[wasm_bindgen_test]
async fn tag_filled_variant_adds_tag_filled_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Tag variant=TagVariant::Filled>"Vocab"</Tag>
        }
        .into_any()
    });
    tick().await;

    let tag_el = wrapper.query_selector(".tag").unwrap().unwrap();
    let class = tag_el.get_attribute("class").unwrap_or_default();
    assert!(
        class.contains("tag-filled"),
        "filled variant must add tag-filled class; got: {class}"
    );
}

// ─── Tag: renders children text ────────────────────────────────────────

#[wasm_bindgen_test]
async fn tag_renders_children_text() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Tag variant=TagVariant::Olive test_id="my-tag">"漢字"</Tag>
        }
        .into_any()
    });
    tick().await;

    let tag_el = wrapper
        .query_selector("[data-testid=\"my-tag\"]")
        .unwrap()
        .unwrap();
    let text = tag_el.text_content().unwrap();
    assert!(
        text.contains("漢字"),
        "tag must render children text; got: {text}"
    );
}

// ─── Tag: test_id attribute ────────────────────────────────────────────

#[wasm_bindgen_test]
async fn tag_test_id_attribute_set() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Tag variant=TagVariant::Default test_id="level-tag">"N4"</Tag>
        }
        .into_any()
    });
    tick().await;

    let tag_el = wrapper.query_selector("[data-testid=\"level-tag\"]");
    assert!(
        tag_el.is_ok_and(|e| e.is_some()),
        "tag must be findable by test_id"
    );
}

// ─── Tag: empty test_id does not add data-testid ───────────────────────

#[wasm_bindgen_test]
async fn tag_empty_test_id_does_not_add_attribute() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Tag variant=TagVariant::Default>"N3"</Tag>
        }
        .into_any()
    });
    tick().await;

    let tag_el = wrapper.query_selector(".tag").unwrap().unwrap();
    assert!(
        !tag_el.has_attribute("data-testid"),
        "tag without explicit test_id must not have data-testid attribute"
    );
}

// ─── Tag: on_click renders as <button> ─────────────────────────────────

#[wasm_bindgen_test]
async fn tag_with_on_click_renders_as_button() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Tag
                variant=TagVariant::Default
                on_click=Callback::new(|_ev: leptos::ev::MouseEvent| {})
                test_id="clickable-tag"
            >"Click me"</Tag>
        }
        .into_any()
    });
    tick().await;

    let button = wrapper
        .query_selector("button.tag")
        .unwrap()
        .unwrap();
    let class = button.get_attribute("class").unwrap_or_default();
    assert!(
        class.contains("tag-clickable"),
        "clickable tag must have tag-clickable class; got: {class}"
    );
}

// ─── Tag: without on_click renders as <span> ───────────────────────────

#[wasm_bindgen_test]
async fn tag_without_on_click_renders_as_span() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Tag variant=TagVariant::Default>"Plain"</Tag>
        }
        .into_any()
    });
    tick().await;

    let span = wrapper.query_selector("span.tag").unwrap().unwrap();
    let class = span.get_attribute("class").unwrap_or_default();
    assert!(
        !class.contains("tag-clickable"),
        "non-clickable tag must NOT have tag-clickable class; got: {class}"
    );
}
