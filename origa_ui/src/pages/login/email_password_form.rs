use crate::i18n::*;
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, Input, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;

use super::password_input::PasswordInput;
use super::validation;

#[component]
pub fn EmailPasswordForm(
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional, into)] loading: Signal<bool>,
    #[prop(optional)] server_error: Option<RwSignal<Option<String>>>,
    on_submit: Callback<(String, String)>,
) -> impl IntoView {
    let i18n = use_i18n();
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let validation_error = RwSignal::new(None::<String>);

    let display_error = move || {
        if let Some(server_err) = server_error.as_ref().and_then(|s| s.get()) {
            Some(server_err)
        } else {
            validation_error.get()
        }
    };

    let handle_submit = move || {
        let email_val = email.get();
        let password_val = password.get();

        if let Some(ref se) = server_error {
            se.set(None);
        }

        if let Err(e) = validation::validate_credentials(&i18n, &email_val, &password_val) {
            validation_error.set(Some(e));
            return;
        }

        validation_error.set(None);
        on_submit.run((email_val, password_val));
    };

    let on_submit_form = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        // Guards against double-submit: the submit button is disabled while
        // loading, but browsers still fire a form submit on Enter even when
        // the submit button is disabled. The signal is idempotent, so this
        // early-return is belt-and-suspenders, not data-safety.
        if loading.get() {
            return;
        }
        handle_submit();
    };

    let form_test_id = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let error_test_id = Signal::derive(move || {
        let base = test_id.get();
        if base.is_empty() {
            "login-error".to_string()
        } else {
            format!("{}-error", base)
        }
    });

    view! {
        <form class="space-y-4" on:submit=on_submit_form data-testid=form_test_id>
            <Show when=move || display_error().is_some()>
                <Alert
                    test_id=error_test_id
                    alert_type=Signal::derive(|| AlertType::Error)
                    message=Signal::derive(move || display_error().unwrap_or_default())
                />
            </Show>

            <div>
                <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true class="block mb-2">
                    {t!(i18n, login.email_placeholder)}
                </Text>
                <Input
                    value=email
                    input_type=Signal::derive(|| "email".to_string())
                    autocomplete=Signal::derive(|| "email".to_string())
                    id=Signal::derive(|| "email".to_string())
                    name=Signal::derive(|| "email".to_string())
                    placeholder=Signal::derive(|| "example@mail.com".to_string())
                    test_id=Signal::derive(|| "email-input".to_string())
                />
            </div>

            <PasswordInput
                value=password
                id=Signal::derive(|| "password".to_string())
                name=Signal::derive(|| "password".to_string())
                test_id=Signal::derive(|| "password-input".to_string())
            />

            <Button
                variant=Signal::derive(|| ButtonVariant::Olive)
                loading=loading
                disabled=loading
                button_type=Signal::derive(|| "submit".to_string())
                class=Signal::derive(|| "w-full".to_string())
                test_id=Signal::derive(|| "login-submit".to_string())
            >
                {t!(i18n, login.login_button)}
            </Button>
        </form>
    }
}
