//! WASM tests for `pages/login/auth_handlers`: the native language seeded
//! into a brand-new profile must follow the language picked on the login
//! screen (the i18n context the login page rendered with), not a hardcoded
//! default.

use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen_test::*;

use crate::i18n::Locale;
use crate::repository::session::{TrailBaseSession, set_session};
use crate::test_support::{create_wrapper, mount_with_i18n};

wasm_bindgen_test_configure!(run_in_browser);

fn seed_session(email: &str) {
    let session = TrailBaseSession {
        auth_token: "token".to_string(),
        refresh_token: "refresh".to_string(),
        email: email.to_string(),
        trailbase_id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
        record_id: None,
        expires_at: 0,
    };
    set_session(&session).expect("seed session");
}

/// Exercises the production contract: the caller hands over the very
/// `I18nContext` the login page rendered with (explicit argument — the
/// function must not read ambient context because it runs after `.await`
/// inside `spawn_local`, where no reactive owner is scoped).
fn new_user_for_locale(locale: Locale, email: &str) -> origa::domain::User {
    let wrapper = create_wrapper();
    let captured = Rc::new(RefCell::new(None));
    let sink = captured.clone();
    let email = email.to_string();
    mount_with_i18n(&wrapper, move || {
        let i18n = crate::i18n::use_i18n();
        i18n.set_locale(locale);
        seed_session(&email);
        *sink.borrow_mut() = Some(
            super::auth_handlers::create_new_user_from_session(&email, &i18n)
                .expect("user creation must succeed"),
        );
        view! { <div></div> }.into_any()
    });
    captured.borrow().clone().expect("user must be created")
}

#[wasm_bindgen_test]
async fn new_user_language_follows_english_login_selection() {
    let user = new_user_for_locale(Locale::en, "new.user@example.com");

    assert_eq!(
        user.native_language(),
        &origa::domain::NativeLanguage::English,
        "a brand-new profile must inherit the language picked on the login screen"
    );
}

#[wasm_bindgen_test]
async fn new_user_language_follows_russian_login_selection() {
    let user = new_user_for_locale(Locale::ru, "new.user@example.com");

    assert_eq!(
        user.native_language(),
        &origa::domain::NativeLanguage::Russian,
        "a brand-new profile must inherit the language picked on the login screen"
    );
}

#[wasm_bindgen_test]
async fn new_user_from_regular_email_seeds_email_prefix_as_name() {
    let user = new_user_for_locale(Locale::en, "ivan.petrov@example.com");

    assert_eq!(
        user.username(),
        "ivan.petrov",
        "a regular email must seed its local part as the display name"
    );
}

#[wasm_bindgen_test]
async fn new_user_from_apple_relay_email_seeds_empty_name() {
    let user = new_user_for_locale(Locale::en, "g55jkfzf5p@privaterelay.appleid.com");

    assert_eq!(
        user.username(),
        "",
        "an Apple relay local part is opaque noise and must not become the display name"
    );
    assert_eq!(
        user.email(),
        "g55jkfzf5p@privaterelay.appleid.com",
        "the relay email stays the account identity"
    );
}
