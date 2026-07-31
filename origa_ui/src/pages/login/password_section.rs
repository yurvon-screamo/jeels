use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

use super::email_password_form::EmailPasswordForm;

/// Collapsible email/password login section.
///
/// Collapsed by default: only a single "Sign in with password" button is
/// shown, keeping the login card short enough to fit a mobile viewport without
/// scrolling. Expanding reveals the email/password form with a "back" link that
/// collapses it again (so an accidental expand is recoverable, and the user can
/// return to the OAuth-first layout). Behaviour is uniform across platforms.
///
/// Lives in the lib crate (recursion_limit 512) as a sub-component so the
/// `Login` view-tree depth stays flat for the bin crate (limit 128, see ADR-027).
#[component]
pub fn PasswordSection(
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional, into)] loading: Signal<bool>,
    expanded: RwSignal<bool>,
    server_error: RwSignal<Option<String>>,
    on_submit: Callback<(String, String)>,
) -> impl IntoView {
    let i18n = use_i18n();

    // Fixed semantic testids: the toggle/back are the canonical login-form
    // entry points regardless of the optional base test_id (which only scopes
    // the inner form). E2E relies on these stable names.
    const TOGGLE_TEST_ID: &str = "login-password-toggle";
    const BACK_TEST_ID: &str = "login-password-back";

    let form_test_id = Signal::derive(move || {
        let base = test_id.get();
        if base.is_empty() {
            "login-form".to_string()
        } else {
            base
        }
    });

    view! {
        <Show
            when=move || expanded.get()
            fallback=move || {
                view! {
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Ghost)
                        button_type=Signal::derive(|| "button".to_string())
                        class=Signal::derive(|| "w-full".to_string())
                        test_id=Signal::derive(|| TOGGLE_TEST_ID.to_string())
                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                            expanded.set(true);
                        })
                    >
                        {t!(i18n, login.login_with_password)}
                    </Button>
                }
            }
        >
            <div class="space-y-4">
                <EmailPasswordForm
                    loading=loading
                    on_submit=on_submit
                    server_error=server_error
                    test_id=form_test_id
                />
                <button
                    type="button"
                    class=move || {
                        // Mirror the cursor to the actual disabled state so the
                        // control doesn't read "clickable" while a request is
                        // in flight.
                        let cursor = if loading.get() {
                            "cursor-not-allowed"
                        } else {
                            "cursor-pointer"
                        };
                        format!("w-full text-center {}", cursor)
                    }
                    data-testid=BACK_TEST_ID
                    disabled=move || loading.get()
                    on:click=move |_: leptos::ev::MouseEvent| {
                        expanded.set(false);
                    }
                >
                    <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true>
                        {t!(i18n, login.back_to_oauth)}
                    </Text>
                </button>
            </div>
        </Show>
    }
}
