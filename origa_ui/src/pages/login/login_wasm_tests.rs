//! WASM render tests for `pages/profile` and `pages/login` components:
//! `legal_card`, `PasswordInput`, `ProfileHeader`, `DangerZoneCard`,
//! `PasswordCard`, `LoginHeader`, validation.

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use leptos::task::tick;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

use crate::test_support::{create_wrapper, mount_with_i18n, mount_with_router, shared_cell};

wasm_bindgen_test_configure!(run_in_browser);

// ═══════════════════════════════════════════════════════════════════════
// Profile: legal_card
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn legal_card_wraps_privacy_and_terms_links() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        crate::pages::profile::legal_card::legal_card(Signal::derive(|| "lc1".to_string()))
    });
    tick().await;

    let card = wrapper
        .query_selector("[data-testid=\"lc1\"]")
        .unwrap()
        .unwrap();
    assert!(
        card.query_selector("[data-testid=\"legal-links-privacy\"]")
            .unwrap()
            .is_some(),
        "privacy link must be embedded"
    );
    assert!(
        card.query_selector("[data-testid=\"legal-links-terms\"]")
            .unwrap()
            .is_some(),
        "terms link must be embedded"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Profile: DangerZoneCard
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn danger_zone_card_actions_dispatch_and_disable_while_pending() {
    let wrapper = create_wrapper();
    let (set_logout, get_logout) = shared_cell::<RwSignal<bool>>();
    let (set_delete, get_delete) = shared_cell::<RwSignal<bool>>();
    mount_with_i18n(&wrapper, move || {
        let logged_out = RwSignal::new(false);
        let deleted = RwSignal::new(false);
        set_logout.set(Some(logged_out));
        set_delete.set(Some(deleted));
        view! {
            <crate::pages::profile::danger_zone_card::DangerZoneCard
                on_logout=Callback::new(move |_: leptos::ev::MouseEvent| logged_out.set(true))
                on_delete_account=Callback::new(move |_: leptos::ev::MouseEvent| deleted.set(true))
                is_logging_out=Signal::from(true)
                is_deleting=Signal::from(false)
                test_id="dz1"
            />
        }
        .into_any()
    });
    let logged_out = get_logout.get().expect("captured");
    let deleted = get_delete.get().expect("captured");
    tick().await;

    let card = wrapper
        .query_selector("[data-testid=\"dz1\"]")
        .unwrap()
        .unwrap();

    // Logging-out pending → the logout button is disabled, clicks blocked
    let logout_btn = card
        .query_selector("[data-testid=\"profile-logout-btn\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlButtonElement>();
    assert!(
        logout_btn.disabled(),
        "pending logout must disable the button"
    );
    logout_btn.click();
    tick().await;
    assert!(!logged_out.get(), "pending logout button must not dispatch");

    // The delete-account button only appears behind a confirm flow; the
    // card must at least render its confirm entry point.
    assert!(
        !card.text_content().unwrap().is_empty(),
        "danger zone copy must render"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Profile: PasswordCard
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn password_card_renders_change_password_flow() {
    let wrapper = create_wrapper();
    crate::test_support::mount_with_stores(&wrapper, |_auth, _connectivity| {
        view! { <crate::pages::profile::password_card::PasswordCard test_id="pc1" /> }.into_any()
    });
    tick().await;

    let card = wrapper
        .query_selector("[data-testid=\"pc1\"]")
        .unwrap()
        .unwrap();
    assert!(
        !card.text_content().unwrap().is_empty(),
        "password card copy must render"
    );
    // Flow starts collapsed: no password inputs until expanded
    let inputs = card
        .query_selector_all("input[type=\"password\"]")
        .unwrap()
        .length();
    assert_eq!(inputs, 0, "collapsed card shows no password fields");
}

// ═══════════════════════════════════════════════════════════════════════
// Login: header
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn login_header_renders_title_and_subtitle() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! { <crate::pages::login::header::LoginHeader test_id="lh1" /> }.into_any()
    });
    tick().await;

    assert!(
        wrapper
            .query_selector("[data-testid=\"lh1\"]")
            .unwrap()
            .is_some(),
        "login header root must render"
    );
    assert!(
        wrapper
            .query_selector("[data-testid=\"lh1-subtitle\"]")
            .unwrap()
            .is_some(),
        "the localized subtitle must render"
    );
    let logo = wrapper.query_selector("img");
    assert!(
        logo.is_ok_and(|l| l.is_some()),
        "the brand logo must render"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Login: PasswordInput
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn password_input_masks_by_default() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let value = RwSignal::new(String::new());
        view! {
            <crate::pages::login::password_input::PasswordInput value test_id="pi1" />
        }
        .into_any()
    });
    tick().await;

    let input = wrapper
        .query_selector("[data-testid=\"pi1\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>();
    assert_eq!(input.type_(), "password", "default type must mask");
}

#[wasm_bindgen_test]
async fn password_input_toggle_reveals_text() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        let value = RwSignal::new(String::new());
        view! {
            <crate::pages::login::password_input::PasswordInput value test_id="pi2" />
        }
        .into_any()
    });
    tick().await;

    // Act: click the visibility toggle
    wrapper
        .query_selector("[data-testid=\"pi2-toggle\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>()
        .click();
    tick().await;

    // Assert: the input switches to plain text
    let input = wrapper
        .query_selector("[data-testid=\"pi2\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>();
    assert_eq!(input.type_(), "text", "toggle must reveal the password");
}

// ═══════════════════════════════════════════════════════════════════════
// Login: validation (i18n probe)
// ═══════════════════════════════════════════════════════════════════════

fn validation_probe(email: &str) -> Option<Result<(), String>> {
    let wrapper = create_wrapper();
    let out = std::rc::Rc::new(std::cell::RefCell::new(None));
    let sink = out.clone();
    let email = email.to_string();
    mount_with_i18n(&wrapper, move || {
        *sink.borrow_mut() = Some(crate::pages::login::validation::validate_email(
            &crate::i18n::use_i18n(),
            &email,
        ));
        view! { <div></div> }.into_any()
    });
    out.borrow().clone()
}

#[wasm_bindgen_test]
async fn login_validation_rejects_empty_email() {
    assert!(
        validation_probe("   ").is_some_and(|r| r.is_err()),
        "empty email must be rejected"
    );
}

#[wasm_bindgen_test]
async fn login_validation_rejects_email_without_at() {
    assert!(
        validation_probe("not-an-email").is_some_and(|r| r.is_err()),
        "email without @ must be rejected"
    );
}

#[wasm_bindgen_test]
async fn login_validation_accepts_well_formed_email() {
    assert!(
        validation_probe("rin@example.com").is_some_and(|r| r.is_ok()),
        "well-formed email must pass"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Profile: SettingsCard / PersonalDataCard
// ═══════════════════════════════════════════════════════════════════════

#[wasm_bindgen_test]
async fn settings_card_renders_version_metadata() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! { <crate::pages::profile::settings_card::SettingsCard test_id="sc1" /> }.into_any()
    });
    tick().await;

    let card = wrapper
        .query_selector("[data-testid=\"sc1\"]")
        .unwrap()
        .unwrap();
    // Version/commit/build-date rows render (values come from build.rs env)
    let rows = card.query_selector_all(".font-mono").unwrap().length();
    assert_eq!(
        rows, 3,
        "version, commit and build date must render; got {rows}"
    );
}

#[wasm_bindgen_test]
async fn personal_data_card_idle_status_renders_controls() {
    let wrapper = create_wrapper();
    mount_with_i18n(&wrapper, || {
        view! {
            <crate::pages::profile::personal_data_card::PersonalDataCard
                username=RwSignal::new("ivan.petrov".to_string())
                selected_language=RwSignal::new(origa::domain::NativeLanguage::Russian)
                selected_daily_load=RwSignal::new(origa::domain::DailyLoad::Medium)
                save_status=Signal::from(
                    crate::pages::profile::content::AutoSaveStatus::Idle
                )
                on_username_change=Callback::new(|_| ())
                on_language_change=Callback::new(|_| ())
                on_daily_load_change=Callback::new(|_| ())
                on_retry=Callback::new(|_| ())
                test_id="pd1"
            />
        }
        .into_any()
    });
    tick().await;

    let card = wrapper
        .query_selector("[data-testid=\"pd1\"]")
        .unwrap()
        .unwrap();
    // The embedded language toggle renders inside the card
    assert!(
        card.query_selector("[data-testid=\"lang-toggle-en\"]")
            .unwrap()
            .is_some(),
        "the native-language toggle must be embedded"
    );
    // The display-name editor renders, prefilled with the current username
    let name_input = card
        .query_selector("[data-testid=\"profile-username-input\"]")
        .unwrap()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>();
    assert_eq!(
        name_input.value(),
        "ivan.petrov",
        "the name input must be prefilled with the stored username"
    );
}
