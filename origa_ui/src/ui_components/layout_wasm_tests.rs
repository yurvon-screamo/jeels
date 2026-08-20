//! WASM render tests for layout & typography components: `PageLayout`,
//! `CardLayout`, `Heading`, `Text`, `DisplayText`, `ReadingGroup`.

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

use crate::test_support::{create_wrapper, mount_to_wrapper};
use crate::ui_components::{
    CardLayout, CardLayoutSize, DisplayText, Heading, HeadingLevel, PageLayout, PageLayoutVariant,
    ReadingGroup, ReadingItem, Text, TextSize, TypographyVariant,
};

wasm_bindgen_test_configure!(run_in_browser);

// ═══════════════════════════════════════════════════════════════════════
// PageLayout
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn page_layout_default_centered() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <PageLayout test_id="pl1">"child"</PageLayout> }.into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"pl1\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("page-layout-centered"), "got: {class}");
}

#[wasm_bindgen_test]
async fn page_layout_full_variant() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <PageLayout variant=Signal::from(PageLayoutVariant::Full) test_id="pl2">"x"</PageLayout>
        }
        .into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"pl2\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("page-layout-full"), "got: {class}");
}

#[wasm_bindgen_test]
async fn page_layout_renders_children() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <PageLayout test_id="pl3">"inside-layout"</PageLayout> }.into_any()
    });
    tick().await;

    let text = wrapper
        .query_selector("[data-testid=\"pl3\"]")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(text.contains("inside-layout"), "got: {text}");
}

// ═══════════════════════════════════════════════════════════════════════
// CardLayout
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn card_layout_default_adaptive() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <CardLayout test_id="cl1">"c"</CardLayout> }.into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"cl1\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("card-layout-adaptive"), "got: {class}");
}

#[wasm_bindgen_test]
async fn card_layout_small_size() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <CardLayout size=Signal::from(CardLayoutSize::Small) test_id="cl2">"c"</CardLayout>
        }
        .into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"cl2\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("card-layout-small"), "got: {class}");
}

#[wasm_bindgen_test]
async fn card_layout_wraps_content_div() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <CardLayout test_id="cl3">"payload"</CardLayout> }.into_any()
    });
    tick().await;

    let content = wrapper
        .query_selector("[data-testid=\"cl3\"] .card-layout-content")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(content.contains("payload"), "got: {content}");
}

// ═══════════════════════════════════════════════════════════════════════
// Heading
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn heading_h1_level_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Heading level=Signal::from(HeadingLevel::H1) test_id="h1">"Title"</Heading>
        }
        .into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"h1\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("heading-h1"), "got: {class}");
}

#[wasm_bindgen_test]
async fn heading_h3_level_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Heading level=Signal::from(HeadingLevel::H3) test_id="h3">"Section"</Heading>
        }
        .into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"h3\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("heading-h3"), "got: {class}");
}

#[wasm_bindgen_test]
async fn heading_muted_variant_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Heading
                level=Signal::from(HeadingLevel::H2)
                variant=Signal::from(TypographyVariant::Muted)
                test_id="hm"
            >
                "Quiet"
            </Heading>
        }
        .into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"hm\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("text-muted"), "got: {class}");
}

// ═══════════════════════════════════════════════════════════════════════
// Text
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn text_small_size_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <Text size=Signal::from(TextSize::Small) test_id="tx1">"small"</Text> }.into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"tx1\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("text-xs"), "got: {class}");
}

#[wasm_bindgen_test]
async fn text_uppercase_and_tracking() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Text uppercase=Signal::from(true) tracking_widest=Signal::from(true) test_id="tx2">
                "label"
            </Text>
        }
        .into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"tx2\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("uppercase"), "got: {class}");
    assert!(class.contains("tracking-widest"), "got: {class}");
}

#[wasm_bindgen_test]
async fn text_olive_variant_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <Text variant=Signal::from(TypographyVariant::Olive) test_id="tx3">"accent"</Text>
        }
        .into_any()
    });
    tick().await;

    let class = wrapper
        .query_selector("[data-testid=\"tx3\"]")
        .unwrap()
        .unwrap()
        .get_attribute("class")
        .unwrap_or_default();
    assert!(class.contains("text-olive"), "got: {class}");
}

#[wasm_bindgen_test]
async fn display_text_class_and_content() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! { <DisplayText test_id="dt1">"42"</DisplayText> }.into_any()
    });
    tick().await;

    let el = wrapper
        .query_selector("[data-testid=\"dt1\"]")
        .unwrap()
        .unwrap();
    let class = el.get_attribute("class").unwrap_or_default();
    assert!(class.contains("display-text"), "got: {class}");
    assert!(el.text_content().unwrap().contains("42"));
}

// ═══════════════════════════════════════════════════════════════════════
// ReadingGroup
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn reading_group_hidden_when_no_readings() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <ReadingGroup
                label=Signal::derive(|| "On".to_string())
                readings=StoredValue::new(None::<Vec<ReadingItem>>)
                test_id="rg1"
            />
        }
        .into_any()
    });
    tick().await;

    let el = wrapper.query_selector("[data-testid=\"rg1\"]");
    assert!(
        el.is_ok_and(|e| e.is_none()),
        "ReadingGroup with no data must render nothing"
    );
}

#[wasm_bindgen_test]
async fn reading_group_renders_sorted_readings() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let readings: Option<Vec<ReadingItem>> = Some(vec![
            ReadingItem {
                reading: "ショウ".into(),
                freq: 120,
                is_rare: false,
            },
            ReadingItem {
                reading: "セイ".into(),
                freq: 900,
                is_rare: false,
            },
            ReadingItem {
                reading: "サン".into(),
                freq: 0,
                is_rare: true,
            },
        ]);
        view! {
            <ReadingGroup
                label=Signal::derive(|| "On".to_string())
                readings=StoredValue::new(readings)
                rare_hint=Signal::derive(|| "rare reading".to_string())
                test_id="rg2"
            />
        }
        .into_any()
    });
    tick().await;

    let tags = wrapper
        .query_selector_all("[data-testid=\"rg2\"] .reading-tag")
        .unwrap();
    assert_eq!(tags.length(), 3, "three readings rendered");

    // Sorted by freq desc: セイ (900) first, ショウ (120) second, サン (0) last
    let first = tags.get(0).unwrap().text_content().unwrap();
    let last = tags.get(2).unwrap().text_content().unwrap();
    assert!(first.contains("セイ"), "highest freq first; got: {first}");
    assert!(last.contains("サン"), "lowest freq last; got: {last}");

    // Rare flag rendered as data-rare attribute
    let rare_el = tags.get(2).unwrap();
    let rare_attr = rare_el
        .dyn_ref::<web_sys::Element>()
        .and_then(|el| el.get_attribute("data-rare"));
    assert_eq!(rare_attr.as_deref(), Some("true"), "rare reading flagged");
    let common_el = tags.get(0).unwrap();
    let common_attr = common_el
        .dyn_ref::<web_sys::Element>()
        .and_then(|el| el.get_attribute("data-rare"));
    assert_eq!(
        common_attr.as_deref(),
        Some("false"),
        "common reading not flagged"
    );
}

#[wasm_bindgen_test]
async fn reading_group_rare_hint_shown_only_with_rare_items() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let readings: Option<Vec<ReadingItem>> = Some(vec![ReadingItem {
            reading: "セイ".into(),
            freq: 900,
            is_rare: false,
        }]);
        view! {
            <ReadingGroup
                label=Signal::derive(|| "On".to_string())
                readings=StoredValue::new(readings)
                rare_hint=Signal::derive(|| "rare reading".to_string())
                test_id="rg3"
            />
        }
        .into_any()
    });
    tick().await;

    let hint = wrapper.query_selector("[data-testid=\"rg3\"] .reading-rare-hint");
    assert!(
        hint.is_ok_and(|h| h.is_none()),
        "no rare items → hint must be hidden"
    );
}
