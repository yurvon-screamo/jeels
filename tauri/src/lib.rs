mod auth_store;
// device-ai native file-based speech recognition. macOS-only — the device-ai
// Rust backend is compiled in only on macOS/iOS/Android (see tauri/Cargo.toml)
// and the speech-recognize backend is implemented for macOS. The
// `disable-device-ai` feature is a compile-time kill-switch that drops the
// command too, so flipping the switch cannot leave a dangling registered
// handler. On other targets the command is absent and the frontend falls
// back to Whisper WASM.
#[cfg(all(target_os = "macos", not(feature = "disable-device-ai")))]
mod device_ai_commands;
// Auto-updater is Windows/Linux-only. macOS App Store handles updates, and
// non-App-Store macOS builds are intentionally excluded too — the updater
// endpoint (GitHub Releases) would conflict with App Store distribution.
#[cfg(any(windows, target_os = "linux"))]
mod updater_commands;

use auth_store::{auth_store_delete, auth_store_get, auth_store_set};
#[cfg(any(feature = "release-devtools", any(windows, target_os = "linux")))]
use tauri::Manager;
use tauri::{Emitter, Listener};
use tauri_plugin_deep_link::DeepLinkExt;
#[cfg(any(windows, target_os = "linux"))]
use updater_commands::{PendingUpdate, check_for_update, install_update};

/// Returns the deep-link URL that launched (or last targeted) the current
/// Activity. The frontend polls this on mount because the `deep-link://new-url`
/// event fires only on warm `onNewIntent`; see ADR-010 for the Android lifecycle.
#[tauri::command]
fn get_current_deep_link(app: tauri::AppHandle) -> Option<String> {
    match app.deep_link().get_current() {
        Ok(Some(urls)) => urls.first().map(|url| url.to_string()),
        Ok(None) => None,
        Err(e) => {
            tracing::warn!("[deep-link] get_current error: {:?}", e);
            None
        },
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    // On Android, rustls-platform-verifier (pulled in via reqwest → sentry
    // transport) must be initialized with a JNI context before any TLS
    // handshake. Without this, the first HTTPS request panics with
    // "Expect rustls-platform-verifier to be initialized" and aborts the
    // process. We dispatch to the Android event loop to obtain the JNI
    // env + Activity context, blocking until init completes.
    //
    // This must happen BEFORE `init_sentry()`, because the Sentry transport
    // creates a reqwest client that performs the first TLS handshake.
    #[cfg(target_os = "android")]
    init_platform_verifier_android();

    // Initialize Sentry AFTER the rustls crypto provider is installed: the
    // sentry transport uses `rustls-no-provider` and reuses this ring provider.
    // The guard must outlive `tauri::Builder::run` — kept on the stack, dropped
    // only when `run()` returns (i.e. process exit). See ADR-036.
    // 1. Initialize Sentry FIRST — it creates the global Hub that
    //    `sentry::integrations::tracing::layer()` relies on to dispatch
    //    events/logs/breadcrumbs. The tracing subscriber (step 2) must be
    //    registered AFTER the Hub exists, otherwise tracing events fire
    //    into a void.
    let _sentry_guard = init_sentry();

    // 2. Register the tracing subscriber with the Sentry layer. The
    //    sentry-tracing layer routes `tracing::error!` → Sentry events,
    //    `tracing::warn!/info!` → breadcrumbs, and (with `enable_logs`)
    //    structured logs — instead of silently dropping them.
    init_tracing();

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default();

    // Auto-updater + single-instance are Windows/Linux-only. macOS App Store
    // handles updates; macOS non-App-Store is intentionally excluded too.
    #[cfg(any(windows, target_os = "linux"))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tracing::info!("[deep-link] single-instance activated (app was already running)");
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }));
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
        builder = builder.plugin(tauri_plugin_process::init());
        builder = builder.manage(PendingUpdate::new());
    }

    #[cfg(mobile)]
    {
        builder = builder.plugin(tauri_plugin_haptics::init());
    }

    // iOS-only: ASWebAuthenticationSession for OAuth. The plugin wraps Apple's
    // native auth session so Safari returns to the app after the OAuth redirect
    // — the custom-scheme `origa://` is not registered in Info.plist by
    // tauri-plugin-deep-link (its build.rs removes CFBundleURLTypes for
    // non-appLink schemes), so ASWebAuthenticationSession intercepts the
    // callback directly. Android and desktop keep the existing opener +
    // deep-link listener flow.
    #[cfg(target_os = "ios")]
    {
        builder = builder.plugin(tauri_plugin_aswebauth::init());
    }

    builder = builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_tts::init());

    // device-ai (native ASR/OCR/TTS) is primary on macOS/iOS/Android.
    // Excluded on Windows (upstream windows.rs does not compile against
    // windows 0.58) and Linux (no device-ai backend). See tauri/Cargo.toml.
    // The `disable-device-ai` feature is a compile-time kill-switch: it drops
    // the plugin entirely, forcing the fallback stack on every platform.
    #[cfg(all(
        any(target_os = "macos", target_os = "ios", target_os = "android"),
        not(feature = "disable-device-ai")
    ))]
    {
        builder = builder.plugin(tauri_plugin_device_ai_apis::init());
    }

    builder
        .invoke_handler(tauri::generate_handler![
            get_current_deep_link,
            auth_store_get,
            auth_store_set,
            auth_store_delete,
            #[cfg(any(windows, target_os = "linux"))]
            check_for_update,
            #[cfg(any(windows, target_os = "linux"))]
            install_update,
            #[cfg(all(target_os = "macos", not(feature = "disable-device-ai")))]
            device_ai_commands::device_ai_recognize_file
        ])
        .setup(|app| {
            tracing::info!("[deep-link] setup started");

            let handle_for_event = app.handle().clone();

            app.listen("deep-link://new-url", move |event: tauri::Event| {
                let payload = event.payload();
                tracing::info!(
                    "[deep-link] received 'deep-link://new-url' event, payload: {}",
                    payload
                );

                if let Ok(urls) = serde_json::from_str::<Vec<String>>(payload) {
                    tracing::info!("[deep-link] parsed {} url(s) from payload", urls.len());
                    for url in &urls {
                        tracing::info!("[deep-link] checking url: {}", url);
                        if url.starts_with("origa://") {
                            tracing::info!(
                                "[deep-link] emitting 'deep-link-received' with url: {}",
                                url
                            );
                            let _ = handle_for_event.emit("deep-link-received", url);
                        }
                    }
                } else {
                    tracing::error!(
                        "[deep-link] failed to parse payload as Vec<String>: {}",
                        payload
                    );
                }
            });

            tracing::info!("[deep-link] listener for 'deep-link://new-url' registered");

            #[cfg(any(windows, target_os = "linux", target_os = "macos"))]
            {
                match app.deep_link().register_all() {
                    Ok(()) => {
                        tracing::info!(
                            "[deep-link] register_all() succeeded — scheme 'origa://' is registered"
                        );
                    },
                    Err(e) => {
                        tracing::error!(
                            "[deep-link] register_all() FAILED: {:?} — deep links will NOT work!",
                            e
                        );
                    },
                }
            }

            tracing::info!("[deep-link] setup complete");

            // release-devtools feature enables the DevTools panel (F12 /
            // right-click → Inspect) but does NOT auto-open it. Opening the
            // panel at startup was annoying users on every launch.
            #[cfg(feature = "release-devtools")]
            {
                tracing::info!("[devtools] DevTools available (F12 / right-click → Inspect)");
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Register the tracing subscriber with a Sentry layer.
///
/// **Must be called AFTER `init_sentry()`** — the `sentry::integrations::tracing::layer()`
/// dispatches events through the global Hub created by `sentry::init`. Registering
/// the subscriber before the Hub exists causes tracing events to fire into a void.
///
/// The Sentry layer routes tracing events as follows (with `logs` feature):
/// - `ERROR` → Sentry event (issue) + structured log
/// - `WARN`/`INFO` → breadcrumb + structured log
/// - `DEBUG`/`TRACE` → ignored
///
/// The `fmt` layer (stdout/stderr) is desktop-only: on iOS/Android there is no
/// terminal, and writing to stdout/stderr may pollute the system log or be silently
/// dropped depending on the WebView configuration.
fn init_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let registry = tracing_subscriber::registry().with(sentry::integrations::tracing::layer());

    #[cfg(desktop)]
    let registry = registry.with(tracing_subscriber::fmt::layer());

    registry.init();
}

/// Initialize the Sentry client for native (Rust) crash reporting.
///
/// Returns `None` (no-op) when `SENTRY_DSN_TAURI` is empty/unset — this is the
/// dev/Dependabot path where no DSN is configured. The returned guard must be
/// held for the entire application lifetime; dropping it flushes pending
/// events and shuts down the transport background thread.
///
/// The `layer = "tauri"` scope tag distinguishes native events from WASM
/// events in the shared Sentry project (ADR-036).
///
/// `default_integrations` stays `true` (default) so the standard integrations
/// (backtrace, contexts, debug-images, panic, release-health) are registered.
/// The panic integration calls `client.flush(None)` synchronously inside the
/// panic hook — bounded by the OS TCP timeout rather than infinite. Accepting
/// this trade-off keeps the integration simple and is preferred over a custom
/// panic hook with manual integration registration (ADR-036 §6).
fn init_sentry() -> Option<sentry::ClientInitGuard> {
    let dsn: &str = env!("SENTRY_DSN_TAURI");
    if dsn.is_empty() {
        tracing::info!("[sentry] disabled (no SENTRY_DSN)");
        return None;
    }

    // Validate the DSN up-front: `ClientOptions::dsn` panics on an invalid
    // DSN string (sentry::init docs). Parse once to reject garbage here, then
    // pass the original string to the builder (which re-parses internally).
    if let Err(e) = dsn.parse::<sentry::types::Dsn>() {
        tracing::error!("[sentry] invalid DSN, disabling: {e}");
        return None;
    }

    let guard = sentry::init(
        sentry::ClientOptions::new()
            .dsn(dsn)
            .release(env!("SENTRY_RELEASE"))
            .environment(env!("SENTRY_ENVIRONMENT"))
            .send_default_pii(false)
            .enable_logs(true)
            .traces_sample_rate(1.0)
            .in_app_include(["origa", "origa_ui", "origa_app"]),
    );

    sentry::configure_scope(|scope| {
        scope.set_tag("layer", "tauri");
    });

    tracing::info!(
        "[sentry] enabled (release={}, environment={})",
        env!("SENTRY_RELEASE"),
        env!("SENTRY_ENVIRONMENT")
    );

    Some(guard)
}

/// Initialize `rustls-platform-verifier` with the Android JVM/Activity context.
///
/// On Android, reqwest (used by the Sentry transport) uses
/// `rustls-platform-verifier` for TLS certificate validation. The verifier
/// panics ("Expect rustls-platform-verifier to be initialized") unless
/// `init_with_env` has been called with a JNI `JNIEnv` and Activity context.
///
/// We obtain the JavaVM and Activity context via `ndk_context`, which tao
/// initializes before `main()` is called (see tao's `ndk_glue::create`).
/// This avoids the deadlock risk of `wry::dispatch` (which posts to the
/// event loop on the same thread that `run()` blocks).
///
/// This must happen BEFORE `init_sentry()`, because the Sentry transport
/// creates a reqwest client that performs the first TLS handshake.
#[cfg(target_os = "android")]
fn init_platform_verifier_android() {
    use jni::JNIEnv;
    use jni::objects::JObject;

    let ctx = ndk_context::android_context();

    // Safety: ndk_context stores a valid JavaVM pointer initialized by tao.
    let vm = match unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) } {
        Ok(vm) => vm,
        Err(e) => {
            tracing::error!("[rustls] failed to get JavaVM: {e:?}");
            return;
        },
    };

    // Attach the current thread to the JVM. The returned guard detaches
    // automatically when dropped, but we only need the env for the init call.
    let mut env = match vm.attach_current_thread() {
        Ok(env) => env,
        Err(e) => {
            tracing::error!("[rustls] failed to attach thread to JVM: {e:?}");
            return;
        },
    };

    // Safety: ndk_context stores a valid Android Context (Activity) jobject.
    let context = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

    match rustls_platform_verifier::android::init_with_env(&mut env, context) {
        Ok(()) => {
            tracing::info!("[rustls] platform-verifier initialized for Android");
        },
        Err(e) => {
            tracing::error!("[rustls] platform-verifier init failed: {e:?}");
        },
    }
}
