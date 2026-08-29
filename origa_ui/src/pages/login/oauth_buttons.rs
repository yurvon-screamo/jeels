use crate::core::tauri;
use crate::i18n::{I18nContext, Locale, t, use_i18n};
use crate::repository::OAuthProvider;
use crate::repository::set_pkce_verifier_async;
use crate::repository::trailbase_client::trailbase_url;
use crate::store::auth_store::AuthStore;
use js_sys::Promise;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos::wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

/// Compile-time switch for the on-screen OAuth diagnostics overlay.
///
/// Set `ORIGA_DEBUG_OAUTH=1` at build time to surface a running trace of the
/// OAuth URL open flow on top of the login card. When the constant is `false`
/// the `report_debug!` macro expands to a constant-`false` branch with an
/// empty body, so LLVM drops it and **no `String` is allocated** (the
/// `format!` arguments are syntactically inside the branch and never
/// evaluated). The `<Show>` overlay likewise never mounts.
///
/// Implemented via a `const fn` because `PartialEq for str` is not yet stable
/// in const context (rust-lang/rust#76499). The check is intentionally strict:
/// only the exact value `"1"` enables the overlay.
const fn debug_oauth_enabled(env_val: Option<&str>) -> bool {
    match env_val {
        Some(s) => {
            let bytes = s.as_bytes();
            bytes.len() == 1 && bytes[0] == b'1'
        },
        None => false,
    }
}

pub(crate) const DEBUG_OAUTH_ENABLED: bool = debug_oauth_enabled(option_env!("ORIGA_DEBUG_OAUTH"));

/// Reactive slot for the on-device OAuth flow trace.
///
/// Always created by the `Login` parent (cheap) but only written to when
/// `DEBUG_OAUTH_ENABLED` is `true`.
pub(crate) type OAuthDebugSink = RwSignal<Option<String>>;

/// Upper bound on the accumulated trace text. Prevents unbounded growth if
/// the user taps the OAuth button many times without reloading — older lines
/// are dropped FIFO once the cap is exceeded.
const DEBUG_TRACE_MAX_BYTES: usize = 16_384;

/// Appends a single line to the OAuth trace sink. Lines are joined by `\n` so
/// the overlay shows the **full** journey (PKCE generation → URL built →
/// opener invoked → Promise resolved/rejected → fallback) rather than only
/// the terminal state.
fn push_debug_line(sink: OAuthDebugSink, line: String) {
    sink.update(|prev: &mut Option<String>| {
        let next = match prev.take() {
            Some(existing) if existing.len() + line.len() < DEBUG_TRACE_MAX_BYTES => {
                format!("{existing}\n{line}")
            },
            _ => line,
        };
        *prev = Some(next);
    });
}

/// Trace emission macro, gated on `DEBUG_OAUTH_ENABLED` (compile-time
/// `option_env!("ORIGA_DEBUG_OAUTH") == "1"`). The `format!` arguments sit
/// inside the guard, so when the env var is unset the branch is dead and the
/// optimizer removes it in release builds — no `format!` call, no allocation.
/// Not a language guarantee: dev/debug builds may still evaluate the branch.
macro_rules! report_debug {
    ($sink:expr, $($arg:tt)*) => {
        if DEBUG_OAUTH_ENABLED {
            if let Some(s) = $sink {
                push_debug_line(s, format!($($arg)*));
            }
        }
    };
}

#[component]
pub fn OAuthButtons(
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional)] debug_sink: Option<OAuthDebugSink>,
) -> impl IntoView {
    let i18n = use_i18n();
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");
    let auth_store_apple = auth_store.clone();
    let auth_store_google = auth_store.clone();
    let auth_store_yandex = auth_store.clone();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    // Sign in with Apple is the first button per App Store HIG: it must
    // precede other third-party sign-in options wherever they are offered.
    let apple_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "oauth-apple".to_string()
        } else {
            format!("{}-apple", val)
        }
    });

    let google_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "oauth-google".to_string()
        } else {
            format!("{}-google", val)
        }
    });

    let yandex_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "oauth-yandex".to_string()
        } else {
            format!("{}-yandex", val)
        }
    });

    view! {
        <div class="space-y-3" data-testid=test_id_val>
            <button
                type="button"
                class="w-full flex items-center justify-center gap-3 px-4 py-3 border border-[var(--border-dark)] bg-[var(--bg-cream)] hover:bg-[var(--bg-aged)] transition-colors"
                data-testid=apple_test_id
                on:click=move |_: leptos::ev::MouseEvent| {
                    let auth_store = auth_store_apple.clone();
                    spawn_local(async move {
                        open_oauth_url(OAuthProvider::Apple, debug_sink, auth_store, i18n).await;
                    });
                }
            >
                <AppleIcon />
                <span class="text-[var(--fg-black)]">{t!(i18n, login.apple_login)}</span>
            </button>

            <button
                type="button"
                class="w-full flex items-center justify-center gap-3 px-4 py-3 border border-[var(--border-dark)] bg-[var(--bg-cream)] hover:bg-[var(--bg-aged)] transition-colors"
                data-testid=google_test_id
                on:click=move |_: leptos::ev::MouseEvent| {
                    let auth_store = auth_store_google.clone();
                    spawn_local(async move {
                        open_oauth_url(OAuthProvider::Google, debug_sink, auth_store, i18n).await;
                    });
                }
            >
                <GoogleIcon />
                <span class="text-[var(--fg-black)]">{t!(i18n, login.google_login)}</span>
            </button>

            <button
                type="button"
                class="w-full flex items-center justify-center gap-3 px-4 py-3 border border-[var(--border-dark)] bg-[var(--bg-cream)] hover:bg-[var(--bg-aged)] transition-colors"
                data-testid=yandex_test_id
                on:click=move |_: leptos::ev::MouseEvent| {
                    let auth_store = auth_store_yandex.clone();
                    spawn_local(async move {
                        open_oauth_url(OAuthProvider::Yandex, debug_sink, auth_store, i18n).await;
                    });
                }
            >
                <YandexIcon />
                <span class="text-[var(--fg-black)]">{t!(i18n, login.yandex_login)}</span>
            </button>
        </div>
    }
}

fn open_url_external(url: &str, debug_sink: Option<OAuthDebugSink>) {
    let Some(window) = web_sys::window() else {
        report_debug!(debug_sink, "no window available");
        return;
    };

    if !tauri::is_tauri() {
        report_debug!(debug_sink, "browser path: location.href = {url}");
        let _ = window.location().set_href(url);
        return;
    }

    let Some(open_url_fn) = tauri::opener_open_url_fn() else {
        report_debug!(
            debug_sink,
            "opener.openUrl not bound, falling back to window.open"
        );
        let _ = window.open_with_url_and_target(url, "_blank");
        return;
    };

    // `opener.openUrl` returns a Promise on Tauri v2 (mobile + desktop). The
    // synchronous `call1` only catches exceptions thrown while constructing
    // the call; async rejections (capability scope mismatch, browser launch
    // failure on Android) silently slip through and historically caused the
    // "Войти Google button does nothing" symptom on Android.
    match open_url_fn.call1(&JsValue::UNDEFINED, &JsValue::from_str(url)) {
        Ok(value) => match value.dyn_into::<Promise>() {
            Ok(promise) => {
                // URL must be owned before being moved into the 'static future.
                let url_owned = url.to_string();
                report_debug!(debug_sink, "opener invoked: {url_owned}");
                spawn_local(async move {
                    match JsFuture::from(promise).await {
                        Ok(_) => {
                            report_debug!(debug_sink, "opener resolved: {url_owned}");
                        },
                        Err(err) => {
                            report_debug!(
                                debug_sink,
                                "opener rejected ({err:?}), window.open fallback: {url_owned}",
                            );
                            if let Some(w) = web_sys::window() {
                                let _ = w.open_with_url_and_target(&url_owned, "_blank");
                            }
                        },
                    }
                });
            },
            Err(_) => {
                // Synchronous success path (some desktop runtimes return void).
                report_debug!(debug_sink, "opener sync ok: {url}");
            },
        },
        Err(sync_err) => {
            report_debug!(
                debug_sink,
                "opener sync throw ({sync_err:?}), window.open fallback: {url}",
            );
            let _ = window.open_with_url_and_target(url, "_blank");
        },
    }
}

async fn open_oauth_url(
    provider: OAuthProvider,
    debug_sink: Option<OAuthDebugSink>,
    auth_store: AuthStore,
    i18n: I18nContext<Locale>,
) {
    use crate::repository::TrailBaseClient;
    use crate::repository::trailbase_auth::{generate_pkce_challenge, generate_pkce_verifier};

    // Reuse the canonical backend host (`https://app.origa.uwuwu.net` in
    // production) instead of `ORIGA_PUBLIC_BASE_URL`, which has been empty
    // since commit eeee03ad (mobile OIDC redirect refactor) and produced a
    // relative `redirect_uri` that TrailBase could not redirect to.
    let redirect_uri = if tauri::is_tauri() {
        format!(
            "{}{}",
            trailbase_url(),
            "/public/auth/desktop-callback.html"
        )
    } else {
        let window = web_sys::window().expect("window not available");
        let base_url = window.location().origin().unwrap_or_default();
        format!("{}/login", base_url)
    };

    let verifier = generate_pkce_verifier();
    let challenge = generate_pkce_challenge(&verifier);
    report_debug!(
        debug_sink,
        "pkce generated; provider={}; redirect_uri={redirect_uri}",
        provider.as_str()
    );

    // Persist the PKCE verifier BEFORE opening the external browser.
    // On Android, the OS may kill the app process while the user is in the
    // external browser, so localStorage (which is unreliable under process
    // kills) is insufficient — we must fsync to the native store first.
    // The await ensures the IPC write + Store::save() completes before we
    // leave the app.
    if let Err(e) = set_pkce_verifier_async(&verifier).await {
        report_debug!(debug_sink, "pkce verifier persist failed: {e}");
    }

    let client = TrailBaseClient::new();
    let url = client.get_oauth_url(provider.as_str(), &redirect_uri, &challenge);
    report_debug!(debug_sink, "oauth url built: {url}");

    // Apple platforms (iOS + macOS), inside the Tauri WebView only:
    // ASWebAuthenticationSession opens the OAuth page in a dedicated auth
    // context and intercepts the `origa://` callback directly — the callback
    // URL is returned via the Tauri command's Promise, bypassing the
    // deep-link listener entirely. App Review rejects default-browser
    // sign-in on the Mac App Store (Guideline 4), so the packaged app must
    // never take the opener path. The `is_tauri()` conjunct keeps plain
    // browser sessions on macOS on the web redirect flow, where no Tauri
    // invoke exists.
    //
    // Other platforms (Android, Windows, Linux, web): opener + deep-link
    // listener flow.
    if tauri::is_tauri() && tauri::is_apple() {
        auth_store.oauth_error.set(None);
        auth_store.is_oauth_loading.set(true);
        match start_aswebauth(&url, "origa").await {
            Ok(callback_url) => {
                report_debug!(debug_sink, "aswebauth callback: {callback_url}");
                let result =
                    super::oauth_listeners::process_oauth_url(&callback_url, &auth_store, &i18n)
                        .await;
                super::oauth_listeners::handle_oauth_result(result, &auth_store);
                auth_store.is_oauth_loading.set(false);
                return;
            },
            Err(e) if e == "cancelled" => {
                report_debug!(debug_sink, "aswebauth cancelled by user");
                auth_store.is_oauth_loading.set(false);
                // User dismissed Safari — do NOT fall through to the opener;
                // re-opening the browser would go against their intent.
                return;
            },
            Err(e) => {
                report_debug!(debug_sink, "aswebauth failed: {e}");
                auth_store.is_oauth_loading.set(false);
                // Technical failure must NOT silently degrade to the default
                // browser — that is exactly the behavior App Review rejects.
                // Surface the failure to the user instead.
                let message = i18n
                    .get_keys_untracked()
                    .login()
                    .oauth_session_error()
                    .inner()
                    .replace("{}", &e);
                auth_store.oauth_error.set(Some(message));
                tracing::warn!("OAuth authentication session failed: {e}");
                return;
            },
        }
    }

    open_url_external(&url, debug_sink);

    // On Android the WebView JS is frozen while the app is backgrounded in the
    // external browser, so the deep-link callback event emitted by Rust on
    // return is lost (its webview.eval delivery never runs). Polling on a timer
    // sidesteps the missing resume signal: the timer pauses while frozen and
    // resumes on Activity onResume, recovering the pending callback URL.
    super::oauth_listeners::start_resume_polling(auth_store, i18n);
}

/// Invokes the `aswebauth` Tauri plugin to start an
/// `ASWebAuthenticationSession`.
///
/// Returns the callback URL (e.g. `origa://auth/callback?code=...`) on
/// success, or an error string if the session failed or was cancelled.
///
/// Uses `__TAURI__.core.invoke` to call `plugin:aswebauth|start_auth` with
/// `{ url, callbackScheme }`. iOS delegates to the Swift mobile plugin; macOS
/// runs the native Rust implementation — both resolve the Promise with the
/// callback URL intercepted by the session.
async fn start_aswebauth(url: &str, callback_scheme: &str) -> Result<String, String> {
    use crate::core::tauri::invoke_fn;
    use js_sys::Object as JsObject;
    use leptos::wasm_bindgen::JsCast;
    use leptos::wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;

    let invoke_fn = invoke_fn().ok_or("Tauri invoke not available")?;

    let args = JsObject::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("url"), &JsValue::from_str(url))
        .map_err(|e| format!("failed to set url arg: {e:?}"))?;
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("callbackScheme"),
        &JsValue::from_str(callback_scheme),
    )
    .map_err(|e| format!("failed to set callbackScheme arg: {e:?}"))?;

    let result = invoke_fn
        .call2(
            &JsValue::UNDEFINED,
            &JsValue::from_str("plugin:aswebauth|start_auth"),
            &args,
        )
        .map_err(|e| format!("invoke('start_auth') call failed: {e:?}"))?;

    let promise = result
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| "invoke('start_auth') did not return a Promise".to_string())?;

    let value = JsFuture::from(promise).await.map_err(|e| {
        // Preserve the raw rejection string (e.g. "cancelled") so the caller
        // can pattern-match on it. Wrapping in format!("rejected: ...")
        // would break the Err(e) if e == "cancelled" branch in open_oauth_url.
        e.as_string()
            .unwrap_or_else(|| format!("invoke('start_auth') rejected: {e:?}"))
    })?;

    js_sys::Reflect::get(&value, &JsValue::from_str("url"))
        .ok()
        .and_then(|v| v.as_string())
        .ok_or("missing 'url' field in start_auth response".to_string())
}

#[component]
fn AppleIcon() -> impl IntoView {
    // Official Apple mark (simple-icons "apple", CC0 — same source as the
    // Google/Yandex marks).
    view! {
        <svg class="w-5 h-5" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            // Explicit token (not `currentColor`): the icon must not depend
            // on the inherited `color` cascade, same pattern as YandexIcon.
            <path
                fill="var(--fg-black)"
                d="M12.152 6.896c-.948 0-2.415-1.078-3.96-1.04-2.04.027-3.91 1.183-4.961 3.014-2.117 3.675-.546 9.103 1.519 12.09 1.013 1.454 2.208 3.09 3.792 3.039 1.52-.065 2.09-.987 3.935-.987 1.831 0 2.35.987 3.96.948 1.637-.026 2.676-1.48 3.676-2.948 1.156-1.688 1.636-3.325 1.662-3.415-.039-.013-3.182-1.221-3.22-4.857-.026-3.04 2.48-4.494 2.597-4.559-1.429-2.09-3.623-2.324-4.39-2.376-2-.156-3.675 1.09-4.61 1.09zM15.53 3.83c.843-1.012 1.4-2.427 1.245-3.83-1.207.052-2.662.805-3.532 1.818-.78.896-1.454 2.338-1.273 3.714 1.338.104 2.715-.688 3.559-1.701"
            />
        </svg>
    }
}

#[component]
fn GoogleIcon() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path
                fill="#4285F4"
                d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
            />
            <path
                fill="#34A853"
                d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
            />
            <path
                fill="#FBBC05"
                d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
            />
            <path
                fill="#EA4335"
                d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
            />
        </svg>
    }
}

#[component]
fn YandexIcon() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path fill="#FC3F1D" d="M3 3h18v18H3V3z" />
            <path
                fill="var(--bg-paper)"
                d="M13.32 18.82V12.7l2.94-7.52h-2.66l-1.57 4.38-1.57-4.38H7.8l2.94 7.52v6.12h2.58z"
            />
        </svg>
    }
}
