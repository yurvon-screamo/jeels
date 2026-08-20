//! Component-level WASM tests for components that require i18n context.
//!
//! These tests mount real Leptos components into a headless browser DOM.
//! Unlike `component_wasm_tests`, these components call `use_i18n()` so
//! we provide the i18n context inside the mount closure.
//!
//! Run locally:
//! ```bash
//! wasm-pack test --headless --chrome origa_ui --features csr -- component_i18n_wasm_tests
//! ```

#![cfg(all(target_arch = "wasm32", test))]

use std::collections::HashSet;

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen_test::*;

use crate::i18n::Locale;
use crate::test_support::{create_wrapper, mount_to_wrapper};
use crate::ui_components::{FuriganaText, LevelSelector, Stepper, StepperStep, WordTranslations};

wasm_bindgen_test_configure!(run_in_browser);

/// Mount a component that needs i18n context. The closure provides the
/// context inside the reactive scope before rendering the component.
fn mount_with_i18n<F>(wrapper: &web_sys::Element, f: F)
where
    F: FnOnce() -> AnyView + 'static,
{
    mount_to_wrapper(wrapper, move || {
        leptos_i18n::provide_i18n_context::<Locale>();
        f()
    });
}

// ═══════════════════════════════════════════════════════════════════════
// FuriganaText
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn furigana_plain_kana_no_ruby() {
    let wrapper = create_wrapper();
    let known: HashSet<char> = HashSet::new();
    mount_to_wrapper(&wrapper, move || {
        view! {
            <FuriganaText
                text="たべもの"
                known_kanji=known.clone()
                test_id="fur1"
            />
        }
        .into_any()
    });
    tick().await;

    let html = wrapper
        .query_selector("[data-testid=\"fur1\"]")
        .unwrap()
        .unwrap()
        .inner_html();
    assert!(
        !html.contains("<ruby"),
        "plain kana text must not produce ruby tags; got: {html}"
    );
}

#[wasm_bindgen_test]
async fn furigana_kanji_with_reading_produces_ruby() {
    let wrapper = create_wrapper();
    // Make 食 known so furiganize adds a reading
    let known: HashSet<char> = "食".chars().collect();
    mount_to_wrapper(&wrapper, move || {
        view! {
            <FuriganaText
                text="食べる"
                known_kanji=known.clone()
                test_id="fur2"
            />
        }
        .into_any()
    });
    tick().await;

    let html = wrapper
        .query_selector("[data-testid=\"fur2\"]")
        .unwrap()
        .unwrap()
        .inner_html();
    assert!(
        html.contains("<ruby") || html.contains("class=\"furigana"),
        "kanji text should produce furigana markup; got: {html}"
    );
}

#[wasm_bindgen_test]
async fn furigana_renders_text_content() {
    let wrapper = create_wrapper();
    let known: HashSet<char> = HashSet::new();
    mount_to_wrapper(&wrapper, move || {
        view! {
            <FuriganaText
                text="こんにちは"
                known_kanji=known.clone()
                test_id="fur3"
            />
        }
        .into_any()
    });
    tick().await;

    let text = wrapper
        .query_selector("[data-testid=\"fur3\"]")
        .unwrap()
        .unwrap()
        .text_content()
        .unwrap();
    assert!(
        text.contains("こんにちは"),
        "FuriganaText must contain original text; got: {text}"
    );
}

#[wasm_bindgen_test]
async fn furigana_test_id_set() {
    let wrapper = create_wrapper();
    let known: HashSet<char> = HashSet::new();
    mount_to_wrapper(&wrapper, move || {
        view! {
            <FuriganaText
                text="テスト"
                known_kanji=known.clone()
                test_id="fur4"
            />
        }
        .into_any()
    });
    tick().await;

    let el = wrapper.query_selector("[data-testid=\"fur4\"]");
    assert!(
        el.is_ok_and(|e| e.is_some()),
        "FuriganaText must be findable by test_id"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Stepper
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn stepper_renders_all_steps() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let steps = vec![
            StepperStep {
                number: 1,
                label: "Intro".into(),
            },
            StepperStep {
                number: 2,
                label: "Level".into(),
            },
            StepperStep {
                number: 3,
                label: "Done".into(),
            },
        ];
        let active = RwSignal::new(1usize);
        view! {
            <Stepper steps=Signal::derive(move || steps.clone()) active=active test_id="stp1" />
        }
        .into_any()
    });
    tick().await;

    let labels = wrapper
        .query_selector_all(".stepper-label")
        .unwrap()
        .length();
    assert_eq!(labels, 3, "Stepper must render 3 labels; got: {labels}");
}

#[wasm_bindgen_test]
async fn stepper_active_step_has_active_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let steps = vec![
            StepperStep {
                number: 1,
                label: "A".into(),
            },
            StepperStep {
                number: 2,
                label: "B".into(),
            },
        ];
        let active = RwSignal::new(0usize);
        view! {
            <Stepper steps=Signal::derive(move || steps.clone()) active=active test_id="stp2" />
        }
        .into_any()
    });
    tick().await;

    let html = wrapper
        .query_selector(".stepper")
        .unwrap()
        .unwrap()
        .inner_html();
    assert!(
        html.contains("stepper-step active"),
        "first step must have 'active' class; got: {html}"
    );
}

#[wasm_bindgen_test]
async fn stepper_completed_step_has_completed_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let steps = vec![
            StepperStep {
                number: 1,
                label: "A".into(),
            },
            StepperStep {
                number: 2,
                label: "B".into(),
            },
        ];
        let active = RwSignal::new(1usize);
        view! {
            <Stepper steps=Signal::derive(move || steps.clone()) active=active test_id="stp3" />
        }
        .into_any()
    });
    tick().await;

    let html = wrapper
        .query_selector(".stepper")
        .unwrap()
        .unwrap()
        .inner_html();
    assert!(
        html.contains("stepper-step completed"),
        "step 0 must have 'completed' class when active=1; got: {html}"
    );
}

#[wasm_bindgen_test]
async fn stepper_pending_step_no_special_class() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let steps = vec![
            StepperStep {
                number: 1,
                label: "A".into(),
            },
            StepperStep {
                number: 2,
                label: "B".into(),
            },
            StepperStep {
                number: 3,
                label: "C".into(),
            },
        ];
        let active = RwSignal::new(1usize);
        view! {
            <Stepper steps=Signal::derive(move || steps.clone()) active=active test_id="stp4" />
        }
        .into_any()
    });
    tick().await;

    // Step index 2 (label "C") should be pending: just "stepper-step" without active/completed
    // Count steps without active/completed
    let all_steps = wrapper
        .query_selector_all(".stepper-step")
        .unwrap()
        .length();
    let active_steps = wrapper
        .query_selector_all(".stepper-step.active")
        .unwrap()
        .length();
    let completed_steps = wrapper
        .query_selector_all(".stepper-step.completed")
        .unwrap()
        .length();
    let pending = all_steps - active_steps - completed_steps;
    assert_eq!(
        pending, 1,
        "exactly 1 pending step expected; all={all_steps} active={active_steps} completed={completed_steps}"
    );
}

#[wasm_bindgen_test]
async fn stepper_line_between_steps() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        let steps = vec![
            StepperStep {
                number: 1,
                label: "A".into(),
            },
            StepperStep {
                number: 2,
                label: "B".into(),
            },
            StepperStep {
                number: 3,
                label: "C".into(),
            },
        ];
        let active = RwSignal::new(0usize);
        view! {
            <Stepper steps=Signal::derive(move || steps.clone()) active=active test_id="stp5" />
        }
        .into_any()
    });
    tick().await;

    let lines = wrapper
        .query_selector_all(".stepper-line")
        .unwrap()
        .length();
    assert_eq!(
        lines, 2,
        "3 steps must have 2 lines between them; got: {lines}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// LevelSelector
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn level_selector_renders_all_levels() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        use origa::domain::JapaneseLevel;
        let levels = vec![JapaneseLevel::N5, JapaneseLevel::N4, JapaneseLevel::N3];
        let selected = RwSignal::new(JapaneseLevel::N5);
        view! {
            <LevelSelector
                levels=levels
                selected_level=selected
                on_select=Callback::new(|_| ())
                test_id_prefix=Signal::derive(|| "ls".to_string())
            />
        }
        .into_any()
    });
    tick().await;

    let buttons = wrapper.query_selector_all("button.btn").unwrap().length();
    assert_eq!(
        buttons, 3,
        "LevelSelector must render 3 buttons; got: {buttons}"
    );
}

#[wasm_bindgen_test]
async fn level_selector_selected_is_olive() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        use origa::domain::JapaneseLevel;
        let levels = vec![JapaneseLevel::N5, JapaneseLevel::N4];
        let selected = RwSignal::new(JapaneseLevel::N4);
        view! {
            <LevelSelector
                levels=levels
                selected_level=selected
                on_select=Callback::new(|_| ())
                test_id_prefix=Signal::derive(|| "ls".to_string())
            />
        }
        .into_any()
    });
    tick().await;

    let n4_btn = wrapper
        .query_selector("[data-testid=\"ls-n4\"]")
        .unwrap()
        .unwrap();
    let class = n4_btn.get_attribute("class").unwrap_or_default();
    assert!(
        class.contains("btn-olive"),
        "selected level N4 must have btn-olive; got: {class}"
    );
}

#[wasm_bindgen_test]
async fn level_selector_unselected_is_default() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        use origa::domain::JapaneseLevel;
        let levels = vec![JapaneseLevel::N5, JapaneseLevel::N4];
        let selected = RwSignal::new(JapaneseLevel::N4);
        view! {
            <LevelSelector
                levels=levels
                selected_level=selected
                on_select=Callback::new(|_| ())
                test_id_prefix=Signal::derive(|| "ls".to_string())
            />
        }
        .into_any()
    });
    tick().await;

    let n5_btn = wrapper
        .query_selector("[data-testid=\"ls-n5\"]")
        .unwrap()
        .unwrap();
    let class = n5_btn.get_attribute("class").unwrap_or_default();
    assert!(
        !class.contains("btn-olive"),
        "unselected level N5 must NOT have btn-olive; got: {class}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// WordTranslations
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn word_translations_renders_list_items() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <WordTranslations
                translations=Signal::derive(|| vec!["кошка".to_string()])
                test_id="wt1"
            />
        }
        .into_any()
    });
    tick().await;

    let items = wrapper
        .query_selector_all("[data-testid=\"wt1\"] .word-translations-item")
        .unwrap()
        .length();
    assert_eq!(items, 1, "1 translation → 1 list item; got: {items}");
}

#[wasm_bindgen_test]
async fn word_translations_multiple_items() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <WordTranslations
                translations=Signal::derive(|| vec![
                    "тихий".to_string(),
                    "спокойный".to_string(),
                    "мирный".to_string(),
                ])
                test_id="wt2"
            />
        }
        .into_any()
    });
    tick().await;

    let items = wrapper
        .query_selector_all("[data-testid=\"wt2\"] .word-translations-item")
        .unwrap()
        .length();
    assert_eq!(items, 3, "3 translations → 3 list items; got: {items}");
}

#[wasm_bindgen_test]
async fn word_translations_description_shown() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <WordTranslations
                translations=Signal::derive(|| vec!["тест".to_string()])
                description=Signal::derive(|| Some("a description".to_string()))
                test_id="wt3"
            />
        }
        .into_any()
    });
    tick().await;

    let desc = wrapper.query_selector("[data-testid=\"wt3\"] .word-translations-desc");
    assert!(
        desc.is_ok_and(|e| e.is_some()),
        "description must be rendered when present"
    );
}

#[wasm_bindgen_test]
async fn word_translations_no_description_hidden() {
    let wrapper = create_wrapper();
    mount_to_wrapper(&wrapper, || {
        view! {
            <WordTranslations
                translations=Signal::derive(|| vec!["тест".to_string()])
                description=Signal::derive(|| None::<String>)
                test_id="wt4"
            />
        }
        .into_any()
    });
    tick().await;

    let desc = wrapper.query_selector("[data-testid=\"wt4\"] .word-translations-desc");
    assert!(
        desc.is_ok_and(|e| e.is_none()),
        "description must NOT be rendered when absent"
    );
}
