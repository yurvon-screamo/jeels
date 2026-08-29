// Copyright 2026 yurvon-screamo
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Arguments for the `start_auth` command.
///
/// Serialized from Rust to Swift via `run_mobile_plugin` as
/// `{ "url": "...", "callbackScheme": "origa" }`.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(not(target_os = "ios"), expect(dead_code))]
pub struct StartAuthArgs {
    /// Full OAuth provider URL (PKCE challenge included).
    pub url: String,
    /// Custom URL scheme to intercept (e.g. "origa").
    #[serde(rename = "callbackScheme")]
    pub callback_scheme: String,
}

/// Successful response from `start_auth`.
///
/// Deserialized from Swift's `invoke.resolve(["url": ...])` via
/// `run_mobile_plugin`, then serialized to the frontend as
/// `{ "url": "origa://auth/callback?code=..." }`.
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthResult {
    /// The full callback URL intercepted by ASWebAuthenticationSession.
    pub url: String,
}

/// iOS: delegates to the Swift `AsWebAuthPlugin.startAuth` via
/// `PluginHandle::run_mobile_plugin`. The Swift completion handler resolves
/// with `{ "url": "origa://auth/callback?code=..." }`.
#[cfg(target_os = "ios")]
#[tauri::command]
pub async fn start_auth<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    url: String,
    callback_scheme: String,
) -> Result<AuthResult, String> {
    use crate::AsWebAuthState;
    use tauri::Manager;

    let state = app
        .try_state::<AsWebAuthState<R>>()
        .ok_or("aswebauth plugin not initialized")?;

    let result: AuthResult = state
        .handle
        .run_mobile_plugin(
            "startAuth",
            StartAuthArgs {
                url,
                callback_scheme,
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(result)
}

/// macOS: native `ASWebAuthenticationSession` via objc2. The session must be
/// created and started on the main thread; the async command dispatches there
/// and awaits the completion through a oneshot channel.
#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn start_auth<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    url: String,
    callback_scheme: String,
) -> Result<AuthResult, String> {
    let (tx, rx) = std::sync::mpsc::channel::<Result<AuthResult, String>>();

    let app_for_session = app.clone();
    let mut tx_for_session = Some(tx);
    app.run_on_main_thread(move || {
        // `start_session` owns the sender and guarantees exactly one send:
        // from the completion handler, or synchronously with the real setup
        // failure cause when the session cannot even be created.
        if let Some(tx) = tx_for_session.take() {
            crate::macos::start_session(&app_for_session, &url, &callback_scheme, tx);
        }
    })
    .map_err(|e| format!("failed to dispatch onto the main thread: {e}"))?;

    // The completion handler fires when the user finishes the flow — seconds
    // later. A blocking recv on a dedicated worker thread is the cheapest
    // bridge to async here (same pattern as device_ai_commands).
    let received = tauri::async_runtime::spawn_blocking(move || rx.recv())
        .await
        .map_err(|e| format!("authentication worker failed: {e}"))?;

    received.map_err(|_| "authentication session dropped before completing".to_string())?
}

/// Other platforms stub — always returns an error. Unreachable because the
/// frontend only invokes the command on Apple platforms and falls back to the
/// opener flow elsewhere.
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
#[tauri::command]
pub async fn start_auth(_url: String, _callback_scheme: String) -> Result<AuthResult, String> {
    Err("ASWebAuthenticationSession is only available on Apple platforms".to_string())
}

#[cfg(target_os = "ios")]
use tauri::Runtime;
