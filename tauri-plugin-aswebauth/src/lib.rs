// Copyright 2026 yurvon-screamo
// SPDX-License-Identifier: MIT
//
// iOS-only Tauri plugin that wraps ASWebAuthenticationSession for OAuth flows.
//
// ASWebAuthenticationSession is Apple's recommended API for browser-based
// authentication from native apps. Unlike UIApplication.shared.open (which
// leaves the user stranded in Safari after the OAuth redirect),
// ASWebAuthenticationSession:
//
// 1. Opens Safari in a dedicated authentication context.
// 2. Intercepts the custom-scheme callback URL directly (no CFBundleURLTypes
//    registration needed — the tauri-plugin-deep-link build.rs actively
//    removes CFBundleURLTypes for non-appLink custom schemes on iOS).
// 3. Returns the callback URL via a completion handler.
// 4. Closes Safari automatically.
//
// Non-iOS targets: the plugin compiles but registers no mobile handle.
// `start_auth` returns an error — unreachable because the frontend only
// invokes it under `is_ios()`.

use tauri::Runtime;
use tauri::plugin::{Builder, TauriPlugin};

#[cfg(target_os = "ios")]
use tauri::{Manager, plugin::PluginHandle};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_aswebauth);

mod commands;

/// Plugin state holding the mobile plugin handle (iOS only).
///
/// On iOS, `setup` stores the `PluginHandle` returned by
/// `register_ios_plugin` so that `start_auth` can delegate to the Swift
/// `AsWebAuthPlugin` via `run_mobile_plugin`.
//
// On non-iOS targets this type is not used — `start_auth` returns an error
// without touching plugin state.
#[cfg(target_os = "ios")]
pub struct AsWebAuthState<R: Runtime> {
    pub(crate) handle: PluginHandle<R>,
}

/// Initializes the plugin.
///
/// On iOS, registers the Swift `AsWebAuthPlugin` via
/// `api.register_ios_plugin()` and stores the handle in plugin state.
/// The `start_auth` command retrieves the handle and calls
/// `run_mobile_plugin("startAuth", ...)` to invoke the Swift
/// `ASWebAuthenticationSession` wrapper.
///
/// On non-iOS targets the plugin is a no-op — `start_auth` returns an
/// error because the frontend only invokes it under `is_ios()`.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("aswebauth")
        .setup(|app, api| {
            #[cfg(target_os = "ios")]
            {
                let handle = api.register_ios_plugin(init_plugin_aswebauth)?;
                app.manage(AsWebAuthState::<R> { handle });
            }

            // Non-iOS: no mobile handle to register. start_auth is a stub.
            #[cfg(not(target_os = "ios"))]
            {
                let _ = (app, api);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::start_auth])
        .build()
}
