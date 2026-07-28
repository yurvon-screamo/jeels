mod auth_store;
#[cfg(all(desktop, not(feature = "app-store")))]
mod updater_commands;

use auth_store::{auth_store_delete, auth_store_get, auth_store_set};
use tauri::{Emitter, Listener};
#[cfg(any(feature = "release-devtools", all(desktop, not(feature = "app-store"))))]
use tauri::Manager;
use tauri_plugin_deep_link::DeepLinkExt;
#[cfg(all(desktop, not(feature = "app-store")))]
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

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default();

    #[cfg(all(desktop, not(feature = "app-store")))]
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

    builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_tts::init())
        .plugin(tauri_plugin_haptics::init())
        .invoke_handler(tauri::generate_handler![
            get_current_deep_link,
            auth_store_get,
            auth_store_set,
            auth_store_delete,
            #[cfg(all(desktop, not(feature = "app-store")))]
            check_for_update,
            #[cfg(all(desktop, not(feature = "app-store")))]
            install_update
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
