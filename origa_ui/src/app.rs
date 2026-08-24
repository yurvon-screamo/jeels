use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use tracing::{debug, error, trace};

use crate::core::updater;
use crate::i18n::{native_language_to_locale, use_i18n};
use crate::pages::login::oauth_listeners::{check_url_oauth_callback, setup_oauth_listener};
use crate::repository::migrate_session_to_store_if_needed;
use crate::routes::AppRoutes;
use crate::store::auth_store::AuthStore;
use crate::store::connectivity::ConnectivityStore;
use crate::store::offline_bundle_store::OfflineBundleStore;
use crate::ui_components::{
    ConnectivityBanner, LoadingOverlay, ToastContainer, ToastData, UpdateDrawer, hide_boot_splash,
    inject_font_faces,
};

#[component]
pub fn App() -> impl IntoView {
    inject_font_faces();
    hide_boot_splash();
    let auth_store = AuthStore::new();
    let connectivity = ConnectivityStore::new();
    let offline_bundle_store = OfflineBundleStore::new();
    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());
    let disposed = StoredValue::new(());

    provide_context(auth_store.repository().clone());
    provide_context(auth_store.clone());
    provide_context(connectivity);
    provide_context(offline_bundle_store);

    let i18n = use_i18n();
    let navigate = use_navigate();

    // AD-1 + AD-4: migrate localStorage → Tauri store (one-time), then check
    // session. Migration MUST complete before check_session reads the store so
    // existing users upgrading from pre-ADR-010 builds keep their session.
    let auth_store_for_session = auth_store.clone();
    spawn_local(async move {
        debug!("session lifecycle: starting migration + check_session");
        migrate_session_to_store_if_needed().await;
        auth_store_for_session.check_session();
    });

    // AD-4: OAuth callback checks run concurrently with session check.
    // check_url_oauth_callback handles the web-build URL-fragment path;
    // setup_oauth_listener handles the Tauri deep-link path (both cold-start
    // pending links and live events).
    check_url_oauth_callback(&auth_store, &i18n);
    setup_oauth_listener(auth_store.clone(), i18n);

    // Intercept reload accelerators (Ctrl+R / Cmd+R / F5) so that on Android
    // (physical keyboard) they reload the WebView instead of leaking the
    // tauri.localhost origin to the system browser. No-op in the browser.
    crate::hooks::keyboard_shortcuts::install_reload_guard();
    // Unlock HTMLMediaElement/speechSynthesis autoplay on the first user
    // gesture so lesson-card audio (auto-played from a gesture-less Effect)
    // is not blocked. See hooks::audio_unlock.
    crate::hooks::audio_unlock::install_audio_unlock();

    let update_info = RwSignal::new(None::<updater::UpdateInfo>);
    let download_progress = RwSignal::new(None::<f32>);

    let update_info_clone = update_info;
    spawn_local(async move {
        // Store builds ship without self-update machinery (policy 10.2.5):
        // skip the startup check instead of firing IPC commands that are not
        // registered in that flavor.
        if updater::is_store_build().await {
            tracing::debug!("updater: skipping startup check in store build");
            return;
        }
        if let Some(info) = updater::check_for_updates().await {
            if disposed.is_disposed() {
                return;
            }
            update_info_clone.set(Some(info));
        }
    });

    let i18n_for_lang = i18n;
    Effect::new(move |_| {
        if let Some(user) = auth_store.user.get() {
            let locale = native_language_to_locale(user.native_language());
            tracing::debug!(
                ?locale,
                "App Effect: syncing i18n locale from auth_store.user"
            );
            i18n_for_lang.set_locale(locale);
        }
    });

    // AD-3: SPA navigate Effect — always mounted in App() (unlike the Login
    // page which can unmount). Watches is_authenticated() and navigates to
    // /home ONLY when the user is on /login. This covers the OAuth path where
    // set_oauth_session writes the session asynchronously via IPC + Store and
    // the /login route renders Login directly (outside ProtectedRoute), so
    // without an explicit navigate the user would stay on /login after
    // authentication.
    //
    // The email/password path does NOT need this Effect: the user is on /
    // (root), which renders <ProtectedRoute><Home/></ProtectedRoute>. After
    // user.set(Some), ProtectedRoute renders Home, and Home's onboarding guard
    // redirects new users to /onboarding. Navigating to /home from / would
    // create a transient /home URL that races with E2E waitForURL assertions.
    //
    // The URL guard also prevents the Effect from clobbering deep links when a
    // user reloads on a protected route (/words, /kanji, ...): check_session
    // flips is_authenticated false→true, but current_path is /words (not
    // /login), so no navigate fires. The pathname is read via web_sys (no
    // reactive subscription) so the Effect only re-runs when is_authenticated
    // actually changes.
    //
    // KNOWN LIMITATION: the OAuth path still produces a transient /home URL
    // (/login → /home → /onboarding for new users). This is the same race
    // window that was fixed for email/password, but there are currently no
    // OAuth E2E tests with URL assertions that would catch it. If such tests
    // are added, the OAuth callback handler should navigate directly to
    // /onboarding for new users (or the Effect should be replaced with a
    // route-level guard).
    const LOGIN_REDIRECT_PATH: &str = "/login";
    let auth_store_for_nav = auth_store.clone();
    Effect::new(move |_| {
        if auth_store_for_nav.is_authenticated().get() {
            let current_path = web_sys::window()
                .and_then(|w| w.location().pathname().ok())
                .unwrap_or_default();
            if current_path == LOGIN_REDIRECT_PATH {
                debug!(
                    current_path,
                    "redirect_decision: on /login → navigate to /home"
                );
                navigate("/home", Default::default());
            } else {
                trace!(
                    current_path,
                    "redirect_decision: authenticated but not on /login, no navigate"
                );
            }
        }
    });

    let on_update = Callback::new(move |_| {
        let disposed = StoredValue::new(());
        spawn_local(async move {
            download_progress.set(Some(0.0));

            let result = updater::download_and_install(move |progress| {
                download_progress.set(Some(progress as f32));
            })
            .await;

            if disposed.is_disposed() {
                return;
            }
            if let Err(e) = result {
                error!("Update failed: {}", e);
                download_progress.set(None);
            }
        });
    });

    let auth_store_for_oauth = auth_store.clone();

    view! {
        <ConnectivityBanner />
        {move || update_info.get().map(|info| view! {
            <UpdateDrawer
                current_version=info.current_version
                new_version=info.version
                on_update=on_update
                download_progress=Signal::from(download_progress)
            />
        })}
        <ToastContainer toasts=toasts duration_ms=5000 />
        <Show when=move || auth_store_for_oauth.is_oauth_loading.get()>
            {move || {
                let message = i18n.get_keys().app().logging_in().inner().to_string();
                view! { <LoadingOverlay message=message /> }
            }}
        </Show>
        <AppRoutes />
    }
}
