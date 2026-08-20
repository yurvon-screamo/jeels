//! WASM render tests for feedback components: `Alert`, `Skeleton`,
//! `Spinner` / `LoadingOverlay`, `ProgressBar`, `Tooltip`.
//!
//! Run locally:
//! ```bash
//! wasm-pack test --headless --chrome origa_ui --features csr -- feedback_wasm_tests
//! ```

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

use crate::test_support::{create_wrapper, mount_to_wrapper};
use crate::ui_components::{
    Alert, AlertType, LoadingOverlay, ProgressBar, Skeleton, Spinner, Tooltip,
};

wasm_bindgen_test_configure!(run_in_browser);

// ═══════════════════════════════════════════════════════════════════════
// Alert
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn alert_info_variant_default_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Alert message=Signal::derive(|| "Body".to_string()) test_id="al1" /> }.into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"al1\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("alert-info"), "got: {class}");
}

#[wasm_bindgen_test]
async fn alert_success_variant_adds_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Alert alert_type=Signal::from(AlertType::Success) message=Signal::derive(|| "Saved".to_string()) test_id="al2" />
        }
        .into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"al2\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("alert-success"), "got: {class}");
}

#[wasm_bindgen_test]
async fn alert_warning_variant_adds_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Alert alert_type=Signal::from(AlertType::Warning) message=Signal::derive(|| "Careful".to_string()) test_id="al3" />
        }
        .into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"al3\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("alert-warning"), "got: {class}");
}

#[wasm_bindgen_test]
async fn alert_error_variant_adds_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Alert alert_type=Signal::from(AlertType::Error) message=Signal::derive(|| "Failed".to_string()) test_id="al4" />
        }
        .into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"al4\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("alert-error"), "got: {class}");
}

#[wasm_bindgen_test]
async fn alert_renders_title_and_message() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Alert
                title=Signal::derive(|| "Attention".to_string())
                message=Signal::derive(|| "Something happened".to_string())
                test_id="al5"
            />
        }
        .into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"al5\"]")
        .unwrap()
        .unwrap();
    let text = el.text_content().unwrap();
    assert!(text.contains("Attention"), "got: {text}");
    assert!(text.contains("Something happened"), "got: {text}");
}

#[wasm_bindgen_test]
async fn alert_error_variant_shows_cross_icon() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Alert alert_type=Signal::from(AlertType::Error) test_id="al6" />
        }
        .into_any()
    });
    tick().await;

    let html = wrapper
        .query_selector("[data-testid=\"al6\"]")
        .unwrap()
        .unwrap()
        .inner_html();
    assert!(
        html.contains("M15 9l-6 6m0-6l6 6"),
        "error alert must render the cross icon path; got: {html}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Skeleton
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn skeleton_renders_paper_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || view! { <Skeleton test_id="sk1" /> }.into_any());
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"sk1\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("anima-skeleton-paper"), "got: {class}");
}

#[wasm_bindgen_test]
async fn skeleton_applies_dimensions_style() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Skeleton
                width="120px".to_string()
                height="24px".to_string()
                test_id="sk2"
            />
        }
        .into_any()
    });
    tick().await;

    let style = wrapper
        .query_selector("[data-testid=\"sk2\"]")
        .unwrap()
        .unwrap()
        .get_attribute("style")
        .unwrap_or_default();
    assert!(style.contains("width: 120px"), "got: {style}");
    assert!(style.contains("height: 24px"), "got: {style}");
}

#[wasm_bindgen_test]
async fn skeleton_without_dimensions_has_empty_style() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || view! { <Skeleton test_id="sk3" /> }.into_any());
    tick().await;

    // Tachys may render the style attribute itself as an empty string; the
    // observable contract is that no width/height declarations are set.
    let el = wrapper
        .query_selector("[data-testid=\"sk3\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>();
    let style = el.style();
    assert!(
        style.length() == 0,
        "skeleton without dimensions must not set CSS declarations; got: {}",
        style.css_text()
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Spinner / LoadingOverlay
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn spinner_default_no_size_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || view! { <Spinner test_id="sp1" /> }.into_any());
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"sp1\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(
        !class.contains("spinner-sm") && !class.contains("spinner-lg"),
        "got: {class}"
    );
}

#[wasm_bindgen_test]
async fn spinner_small_size_adds_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Spinner size=Signal::derive(|| "sm".to_string()) test_id="sp2" /> }.into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"sp2\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("spinner-sm"), "got: {class}");
}

#[wasm_bindgen_test]
async fn spinner_large_size_adds_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Spinner size=Signal::derive(|| "lg".to_string()) test_id="sp3" /> }.into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"sp3\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("spinner-lg"), "got: {class}");
}

#[wasm_bindgen_test]
async fn loading_overlay_renders_message_and_spinner() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <LoadingOverlay message=Signal::derive(|| "Loading dictionaries…".to_string()) test_id="lo1" />
        }
        .into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"lo1\"]")
        .unwrap()
        .unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("loading-overlay"), "got: {class}");

    let text = el.text_content().unwrap();
    assert!(text.contains("Loading dictionaries…"), "got: {text}");

    let spinner = el.query_selector(".spinner");
    assert!(
        spinner.is_ok_and(|s| s.is_some()),
        "overlay must contain a spinner"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// ProgressBar
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn progress_bar_renders_percentage() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let value = RwSignal::new(25u32);
        view! { <ProgressBar value=value max=100 test_id="pb1" /> }.into_any()
    });
    tick().await;

    let value_text = wrapper
        .query_selector("[data-testid=\"pb1\"] .progress-value")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(value_text.contains("25%"), "got: {value_text}");
}

#[wasm_bindgen_test]
async fn progress_bar_caps_at_100_percent() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let value = RwSignal::new(150u32);
        view! { <ProgressBar value=value max=100 test_id="pb2" /> }.into_any()
    });
    tick().await;

    let value_text = wrapper
        .query_selector("[data-testid=\"pb2\"] .progress-value")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(value_text.contains("100%"), "got: {value_text}");
}

#[wasm_bindgen_test]
async fn progress_bar_renders_label() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let value = RwSignal::new(10u32);
        view! { <ProgressBar value=value max=100 label=Signal::derive(|| "Cards".to_string()) test_id="pb3" /> }
            .into_any()
    });
    tick().await;

    let label = wrapper
        .query_selector("[data-testid=\"pb3\"] .progress-label")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(label.contains("Cards"), "got: {label}");
}

#[wasm_bindgen_test]
async fn progress_bar_reactive_value_change_updates_dom() {
    let wrapper = create_wrapper();
    // The signal must be created inside the mount's reactive scope; capture
    // it out via shared state so the test can drive it after the first render.
    let (set, get) = crate::test_support::shared_cell::<RwSignal<u32>>();
    mount_to_wrapper(&wrapper, move || {
        let value = RwSignal::new(10u32);
        set.set(Some(value));
        view! { <ProgressBar value=value max=100 test_id="pb4" /> }.into_any()
    });
    let value = get.get().expect("signal captured");
    tick().await;

    let initial = wrapper
        .query_selector("[data-testid=\"pb4\"] .progress-value")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(initial.contains("10%"), "got: {initial}");

    // Act
    value.set(70);
    tick().await;

    let updated = wrapper
        .query_selector("[data-testid=\"pb4\"] .progress-value")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(updated.contains("70%"), "got: {updated}");
}

// ═══════════════════════════════════════════════════════════════════════
// Tooltip
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn tooltip_renders_text_and_children() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Tooltip text=Signal::derive(|| "Helpful hint".to_string()) test_id="tt1">
                <button>"Hover me"</button>
            </Tooltip>
        }
        .into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"tt1\"]")
        .unwrap()
        .unwrap();
    let text = el.text_content().unwrap();
    assert!(text.contains("Hover me"), "got: {text}");
    assert!(text.contains("Helpful hint"), "got: {text}");

    let inner = el
        .query_selector(".tooltip")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(inner.contains("Helpful hint"), "got: {inner}");
}

#[wasm_bindgen_test]
async fn tooltip_container_class_present() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Tooltip text=Signal::derive(|| "hint".to_string())>
                "trigger"
            </Tooltip>
        }
        .into_any()
    });
    tick().await;

    let container = wrapper.query_selector(".tooltip-container");
    assert!(
        container.is_ok_and(|c| c.is_some()),
        "tooltip must render its container"
    );
}
