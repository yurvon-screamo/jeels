// Copyright 2026 yurvon-screamo
// SPDX-License-Identifier: MIT

use serde::Serialize;

/// Successful response from `start_auth`.
///
/// Serialized to the frontend as `{ "url": "origa://auth/callback?code=..." }`.
#[derive(Debug, Serialize)]
pub struct AuthResult {
    /// The full callback URL intercepted by ASWebAuthenticationSession.
    pub url: String,
}

/// Tauri command: start an ASWebAuthenticationSession.
///
/// **iOS only.** On non-iOS targets this command is unreachable (the frontend
/// gates the call behind `tauri::is_ios()`), but it must compile. The Swift
/// plugin overrides this handler on iOS via `run_mobile_plugin`; the Rust
/// stub exists solely so `generate_handler!` can resolve the command name.
#[tauri::command]
pub async fn start_auth(_url: String, _callback_scheme: String) -> Result<AuthResult, String> {
    Err("ASWebAuthenticationSession is only available on iOS".to_string())
}
