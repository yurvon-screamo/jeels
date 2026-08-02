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
// `app_store` cfg is set by `tauri/build.rs` when the `ORIGA_APP_STORE`
// env var is present. `feature = "app-store"` covers local `cargo check
// --features app-store` invocations. The OR allows either mechanism:
// tauri-cli does NOT pass `--features` through to cargo, so the env-var
// path is the only way to gate during `cargo tauri ios build`.
#[cfg(all(desktop, not(any(feature = "app-store", app_store))))]
mod updater_commands;

use auth_store::{auth_store_delete, auth_store_get, auth_store_set};
#[cfg(any(
    feature = "release-devtools",
    all(desktop, not(any(feature = "app-store", app_store)))
))]
use tauri::Manager;
use tauri::{Emitter, Listener};
use tauri_plugin_deep_link::DeepLinkExt;
#[cfg(all(desktop, not(any(feature = "app-store", app_store))))]
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

    // Initialize Sentry AFTER the rustls crypto provider is installed: the
    // sentry transport uses `rustls-no-provider` and reuses this ring provider.
    // The guard must outlive `tauri::Builder::run` — kept on the stack, dropped
    // only when `run()` returns (i.e. process exit). See ADR-036.
    let _sentry_guard = init_sentry();

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default();

    #[cfg(all(desktop, not(any(feature = "app-store", app_store))))]
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
            #[cfg(all(desktop, not(any(feature = "app-store", app_store))))]
            check_for_update,
            #[cfg(all(desktop, not(any(feature = "app-store", app_store))))]
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

            #[cfg(any(windows, target_os = "linux"))]
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

            #[cfg(feature = "release-devtools")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                    tracing::info!("[devtools] DevTools opened (release-devtools feature enabled)");
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
