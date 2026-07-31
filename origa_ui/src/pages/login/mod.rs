pub mod auth_handlers;
pub mod email_password_form;
pub mod header;
mod language_toggle;
pub mod oauth_buttons;
pub mod oauth_listeners;
pub mod password_input;
mod password_section;
mod validation;

pub use header::LoginHeader;
use language_toggle::LoginLanguageToggle;
use oauth_buttons::DEBUG_OAUTH_ENABLED;
use password_section::PasswordSection;

use crate::i18n::*;
use crate::store::auth_store::AuthStore;
use crate::ui_components::{
    Alert, AlertType, CardLayout, CardLayoutSize, Divider, DividerVariant, PageLayout,
    PageLayoutVariant, Text, TextSize, TypographyVariant, legal_links,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn Login() -> impl IntoView {
    let i18n = use_i18n();
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");
    let loading = RwSignal::new(false);
    let server_error = RwSignal::new(None::<String>);
    let disposed = StoredValue::new(());
    // Collapsed by default so the card fits a mobile viewport without
    // scrolling. Expanding reveals the email/password form. Uniform across
    // platforms (a single click to expand on desktop is a minor, accepted
    // trade-off for visual consistency).
    let password_expanded = RwSignal::new(false);

    // On-device OAuth flow trace slot. Only ever written when
    // `ORIGA_DEBUG_OAUTH=1` is set at compile time (see `oauth_buttons`);
    // otherwise stays `None` forever and the `<Show>` overlay below never
    // mounts, so there is zero runtime cost in production.
    let oauth_debug: oauth_buttons::OAuthDebugSink = RwSignal::new(None);

    let auth_store_for_view = auth_store.clone();

    // Navigation after successful email/password login is handled by natural
    // routing: user.set(Some) flips is_authenticated, ProtectedRoute (which
    // wraps the / route) renders Home, and Home's onboarding guard redirects
    // new users to /onboarding. This avoids a transient /home URL that races
    // with E2E waitForURL assertions. The App()-level Effect (see app.rs)
    // covers the OAuth path where the user is on /login (outside
    // ProtectedRoute) and must be navigated explicitly.
    let on_email_submit = Callback::new({
        move |(email, password): (String, String)| {
            let auth_store = auth_store.clone();
            loading.set(true);
            server_error.set(None);
            auth_store.oauth_error.set(None);

            spawn_local(async move {
                let result = auth_store.login(&email, &password, &i18n).await;

                if disposed.is_disposed() {
                    return;
                }

                loading.set(false);

                if let Err(e) = result {
                    tracing::error!("Login error: {:?}", e);
                    server_error.set(Some(
                        i18n.get_keys().login().login_failed().inner().to_string(),
                    ));
                }
            });
        }
    });

    // Wraps the login body (divider + form/oauth stack) in a single AnyView so
    // the CardLayout children tuple (and therefore the bin crate's
    // monomorphised type-depth) does not grow — see ADR-027 for the
    // recursion_limit rationale. Mirrors `login_footer` below.
    let login_body = move || {
        view! {
            <Divider
                variant=Signal::derive(|| DividerVariant::Single)
                class=Signal::derive(|| "my-6".to_string())
                test_id=Signal::derive(|| "login-header-divider".to_string())
            />
            <div class="space-y-6">
                <PasswordSection
                    loading=loading
                    expanded=password_expanded
                    on_submit=on_email_submit
                    server_error=server_error
                    test_id=Signal::derive(|| "login-form".to_string())
                />

                <Show when=move || auth_store_for_view.oauth_error.get().is_some()>
                    <Alert
                        test_id=Signal::derive(|| "oauth-error".to_string())
                        alert_type=Signal::derive(|| AlertType::Error)
                        message=Signal::derive(move || auth_store_for_view.oauth_error.get().unwrap_or_default())
                    />
                </Show>

                <div class="flex items-center gap-4">
                    <Divider variant=Signal::derive(|| DividerVariant::Single) class=Signal::derive(|| "flex-1".to_string()) test_id=Signal::derive(|| "login-divider-left".to_string()) />
                    <Text size=TextSize::Small variant=TypographyVariant::Muted class="whitespace-nowrap" test_id=Signal::derive(|| "login-divider-text".to_string())>
                        {t!(i18n, login.or_login_with)}
                    </Text>
                    <Divider variant=Signal::derive(|| DividerVariant::Single) class=Signal::derive(|| "flex-1".to_string()) test_id=Signal::derive(|| "login-divider-right".to_string()) />
                </div>

                <oauth_buttons::OAuthButtons debug_sink=oauth_debug />
            </div>
        }
        .into_any()
    };

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id=Signal::derive(|| "login-page".to_string())>
            <CardLayout size=CardLayoutSize::Small class="px-4 pt-4 pb-8" test_id=Signal::derive(|| "login-card".to_string())>
                <LoginLanguageToggle test_id=Signal::derive(|| "login-lang-toggle".to_string()) />
                <LoginHeader />
                {login_body()}
                {login_footer(oauth_debug)}
            </CardLayout>
        </PageLayout>
    }
}

// Combines the legal-links row and the OAuth debug overlay into a single
// AnyView so the CardLayout's children-tuple length (and therefore the bin
// crate's monomorphised type-depth) does not grow. See ADR-027 for the
// recursion_limit rationale.
fn login_footer(oauth_debug: oauth_buttons::OAuthDebugSink) -> AnyView {
    view! {
        <Divider
            variant=Signal::derive(|| DividerVariant::Single)
            class=Signal::derive(|| "my-6".to_string())
            test_id=Signal::derive(|| "login-legal-divider".to_string())
        />
        {legal_links(Signal::derive(|| "login-legal-links".to_string()))}
        {debug_overlay(oauth_debug)}
    }
    .into_any()
}

/// Builds the on-screen OAuth diagnostics overlay. Returns `None` (rendered as
/// nothing by Leptos) when `DEBUG_OAUTH_ENABLED` is `false`, so production
/// builds pay nothing for this code path.
fn debug_overlay(oauth_debug: oauth_buttons::OAuthDebugSink) -> Option<impl IntoView> {
    if !DEBUG_OAUTH_ENABLED {
        return None;
    }

    Some(view! {
        <Show when=move || oauth_debug.get().is_some()>
            <div
                data-testid=Signal::derive(|| "oauth-debug-overlay".to_string())
                style="margin-top: 12px; padding: 8px; background: rgba(0,0,0,0.06); \
                       font-family: var(--font-mono); font-size: 11px; line-height: 1.4; \
                       color: var(--fg-black); word-break: break-all; white-space: pre-wrap;"
            >
                {move || oauth_debug.get().unwrap_or_default()}
            </div>
        </Show>
    })
}
