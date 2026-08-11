// Copyright 2026 yurvon-screamo
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Arguments for the `start_auth` command.
///
/// Serialized from the frontend as `{ "url": "...", "callbackScheme": "origa" }`.
/// Only constructed on iOS (passed to `run_mobile_plugin`).
#[derive(Debug, Clone, Deserialize)]
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
/// Serialized to the frontend as `{ "url": "origa://auth/callback?code=..." }`.
#[derive(Debug, Serialize)]
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

/// Non-iOS stub — always returns an error. Unreachable because the frontend
/// gates the call behind `tauri::is_ios()`.
#[cfg(not(target_os = "ios"))]
#[tauri::command]
pub async fn start_auth(_url: String, _callback_scheme: String) -> Result<AuthResult, String> {
    Err("ASWebAuthenticationSession is only available on iOS".to_string())
}
