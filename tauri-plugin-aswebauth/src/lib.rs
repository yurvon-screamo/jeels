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
// Non-iOS targets: the plugin compiles but the Swift code is excluded.
// `init()` returns a no-op plugin that rejects `start_auth` with an error
// — unreachable because the frontend only invokes it under `is_ios()`.

use tauri::Runtime;
use tauri::plugin::{Builder, TauriPlugin};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_aswebauth);

mod commands;

/// Initializes the plugin.
///
/// On iOS, registers the Swift `AsWebAuthPlugin` that wraps
/// `ASWebAuthenticationSession`. On all other targets the plugin is a no-op
/// shell whose `start_auth` command always returns an error.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let builder = Builder::<R>::new("aswebauth");

    #[cfg(target_os = "ios")]
    {
        builder = builder.ios_plugin_binding(init_plugin_aswebauth);
    }

    builder
        .invoke_handler(tauri::generate_handler![commands::start_auth])
        .build()
}
