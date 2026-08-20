//! WASM render tests for `pages/onboarding` step components: `IntroStep`.

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

use crate::pages::onboarding::apps_step::AppsStep;
use crate::pages::onboarding::intro_step::IntroStep;
use crate::pages::onboarding::jlpt_step::JlptStep;
use crate::pages::onboarding::load_step::LoadStep;
use crate::pages::onboarding::onboarding_state::OnboardingState;
use crate::test_support::{create_wrapper, mount_with_i18n, shared_cell};

wasm_bindgen_test_configure!(run_in_browser);

// ═══════════════════════════════════════════════════════════════════════
// IntroStep
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn intro_step_renders_title_subtitle_and_language_bar() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let lang = RwSignal::new(origa::domain::NativeLanguage::Russian);
        view! { <IntroStep selected_language=lang test_id="is1" /> }.into_any()
    });
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"intro-step-language-bar\"]")
            .unwrap()
            .is_some(),
        "language bar must render"
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"intro-step-title\"]")
            .unwrap()
            .is_some(),
        "title must render"
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"intro-step-subtitle\"]")
            .unwrap()
            .is_some(),
        "subtitle must render"
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"intro-lang-toggle\"]")
            .unwrap()
            .is_some(),
        "the language toggle must be embedded"
    );
}

#[wasm_bindgen_test]
async fn intro_step_language_toggle_switches_signal() {
    let wrapper = create_wrapper();
    let (set_lang, get_lang) = shared_cell::<RwSignal<origa::domain::NativeLanguage>>();
    mount_with_i18n(&wrapper, move || {
        let lang = RwSignal::new(origa::domain::NativeLanguage::Russian);
        set_lang.set(Some(lang));
        view! { <IntroStep selected_language=lang test_id="is2" /> }.into_any()
    });
    let lang = get_lang.get().expect("captured");
    tick().await;

    wrapper
        .query_selector("[data-testid=\"lang-toggle-en\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert_eq!(
        lang.get(),
        origa::domain::NativeLanguage::English,
        "the embedded toggle must drive the signal"
    );
}

/// Mount helper providing the `RwSignal<OnboardingState>` context the step
/// components require.
fn mount_onboarding_step<F>(wrapper: &web_sys::Element, f: F)
where
    F: FnOnce() -> AnyView + 'static,
{
    use crate::test_support::mount_with_i18n;
    mount_with_i18n(wrapper, move || {
        let state = RwSignal::new(OnboardingState::new());
        provide_context(state);
        f()
    });
}

// ═══════════════════════════════════════════════════════════════════════
// JlptStep
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn jlpt_step_renders_level_options() {
    let wrapper = create_wrapper();
    mount_onboarding_step(&wrapper, || view! { <JlptStep test_id="js1" /> }.into_any());
    tick().await;

    let step = wrapper
        .query_selector("[data-testid=\"js1\"]")
        .unwrap()
        .unwrap();
    assert!(
        step.query_selector("[data-testid=\"jlpt-step-title\"]")
            .unwrap()
            .is_some(),
        "title must render"
    );
    // Level options: none + N5..N1 = 6 clickable rows. The "none" label is
    // localized (its test id differs per locale); N5..N1 are stable.
    for code in ["n5", "n4", "n3", "n2", "n1"] {
        let found = step
            .query_selector(&format!("[data-testid=\"jlpt-option-{code}\"]"))
            .unwrap()
            .is_some();
        assert!(found, "level option {code} must render");
    }
    let all_options = step
        .query_selector_all("[data-testid^=\"jlpt-option-\"]")
        .unwrap()
        .length();
    assert_eq!(
        all_options, 6,
        "none + five levels must render; got {all_options}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// AppsStep
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn apps_step_renders_without_available_sets() {
    let wrapper = create_wrapper();
    mount_onboarding_step(&wrapper, || view! { <AppsStep test_id="as1" /> }.into_any());
    tick().await;

    // No available sets → the step still renders its heading copy.
    let step = wrapper
        .query_selector("[data-testid=\"as1\"]")
        .unwrap()
        .unwrap();
    assert!(!step.text_content().unwrap().is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// LoadStep
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn load_step_selection_updates_onboarding_state() {
    let wrapper = create_wrapper();
    let (set_state, get_state) = shared_cell::<RwSignal<OnboardingState>>();
    mount_with_i18n(&wrapper, move || {
        let state = RwSignal::new(OnboardingState::new());
        set_state.set(Some(state));
        provide_context(state);
        view! { <LoadStep test_id="ls1" /> }.into_any()
    });
    let state = get_state.get().expect("captured");
    tick().await;

    // The daily-load list offers six options; clicking one syncs the state
    wrapper
        .query_selector("[data-testid=\"daily-load-option-maximum\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    assert_eq!(
        state.get().daily_load,
        origa::domain::DailyLoad::Maximum,
        "load selection must propagate into the onboarding state"
    );
}
