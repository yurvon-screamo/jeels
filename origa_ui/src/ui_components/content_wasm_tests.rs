//! WASM render tests for content components: `MarkdownText`,
//! `AudioButtons`, the `derive_test_id` helper, and `hide_boot_splash`.

#![cfg(all(target_arch = "wasm32", test))]

use std::collections::HashSet;

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen_test::*;

use crate::test_support::{create_wrapper, mount_to_wrapper};
use crate::ui_components::{AudioButtons, MarkdownText, MarkdownVariant, derive_test_id};

wasm_bindgen_test_configure!(run_in_browser);

// ═══════════════════════════════════════════════════════════════════════
// MarkdownText
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn markdown_text_renders_html_content() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <MarkdownText
                content=Signal::derive(|| "**bold** statement".to_string())
                known_kanji=HashSet::new()
                furigana=false
                test_id="mdx1"
            />
        }
        .into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"mdx1\"]")
        .unwrap()
        .unwrap();
    assert!(
        el.inner_html().contains("<strong>bold</strong>"),
        "bold markdown must render as <strong>; got: {}",
        el.inner_html()
    );
}

#[wasm_bindgen_test]
async fn markdown_text_default_variant_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <MarkdownText
                content=Signal::derive(|| "x".to_string())
                known_kanji=HashSet::new()
                furigana=false
                test_id="mdx2"
            />
        }
        .into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"mdx2\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("markdown-text"), "got: {class}");
    assert!(class.contains("prose-sm"), "got: {class}");
}

#[wasm_bindgen_test]
async fn markdown_text_large_variant_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <MarkdownText
                content=Signal::derive(|| "x".to_string())
                known_kanji=HashSet::new()
                furigana=false
                variant=Signal::from(MarkdownVariant::Large)
                test_id="mdx3"
            />
        }
        .into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"mdx3\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("prose-lg"), "got: {class}");
}

#[wasm_bindgen_test]
async fn markdown_text_reactive_content_change() {
    let wrapper = create_wrapper();
    let (set, get) = crate::test_support::shared_cell::<RwSignal<String>>();
    mount_to_wrapper(&wrapper, move || {
        let content = RwSignal::new("first".to_string());
        set.set(Some(content));
        view! {
            <MarkdownText content=content known_kanji=HashSet::new() furigana=false test_id="mdx4" />
        }
        .into_any()
    });
    let content = get.get().expect("captured");
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"mdx4\"]")
            .unwrap()
            .unwrap()
            .text_content()
            .unwrap()
            .contains("first")
    );

    content.set("**second**".to_string());
    tick().await;

    let html = wrapper
        .query_selector("[data-testid=\"mdx4\"]")
        .unwrap()
        .unwrap()
        .inner_html();
    assert!(
        html.contains("<strong>second</strong>"),
        "content change must re-render; got: {html}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// AudioButtons
// ══════════════════════════════════════════════════════════════════════
// NOT TESTED HERE: rendering with Japanese text requires the lindera
// dictionary (`SEGMENTER` static) which is CDN-loaded at app startup and
// unavailable in component tests — `get_reading_from_text` panics with
// `RuntimeError: unreachable` without it. Covered by E2E instead.

#[wasm_bindgen_test]
async fn audio_buttons_empty_text_render_nothing() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <AudioButtons text="" audio_path=None test_id="ab3" /> }.into_any()
    });
    tick().await;

    let el = wrapper.query_selector(".audio-buttons");
    assert!(
        el.is_ok_and(|e| e.is_none()),
        "empty text and no audio path → nothing rendered"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// derive_test_id helper
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn derive_test_id_combines_base_and_suffix() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let derived = derive_test_id(Signal::derive(|| "base".to_string()), "action");
        view! { <span data-testid=move || derived.get()>"x"</span> }.into_any()
    });
    tick().await;

    let el = wrapper.query_selector("[data-testid=\"base-action\"]");
    assert!(
        el.is_ok_and(|e| e.is_some()),
        "base + suffix must combine into base-action"
    );
}

#[wasm_bindgen_test]
async fn derive_test_id_empty_base_uses_suffix_alone() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let derived = derive_test_id(Signal::from(String::new()), "action");
        view! { <span data-testid=move || derived.get()>"x"</span> }.into_any()
    });
    tick().await;

    let el = wrapper.query_selector("[data-testid=\"action\"]");
    assert!(
        el.is_ok_and(|e| e.is_some()),
        "empty base must fall back to the bare suffix"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// boot_splash
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn hide_boot_splash_fades_then_removes_splash_node() {
    // Arrange: a static splash node as in index.html
    let document = web_sys::window().unwrap().document().unwrap();
    let splash = document.create_element("div").unwrap();
    splash.set_id("origa-boot-splash");
    let _ = document.body().unwrap().append_child(&splash);

    // Act
    crate::ui_components::hide_boot_splash();
    // Poll: SETTLE_MS (60) applies the hidden class, FADE_OUT_MS (300)
    // removes the node.
    let hidden = crate::test_support::wait_until(
        || {
            splash
                .get_attribute("class")
                .is_some_and(|c| c.contains("origa-boot-splash--hidden"))
        },
        20,
        25,
    )
    .await;
    let removed = crate::test_support::wait_until(
        || document.get_element_by_id("origa-boot-splash").is_none(),
        20,
        25,
    )
    .await;

    // Assert
    assert!(hidden, "splash must get the hidden class during fade");
    assert!(removed, "splash node must be removed after the fade window");
}
