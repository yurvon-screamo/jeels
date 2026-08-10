use crate::repository::session::{
    TrailBaseSession, clear_session_async, get_session, set_session_async,
};
use crate::repository::trailbase_auth::decode_jwt_claims;
use crate::repository::trailbase_client::{
    AuthError, AuthRequestClient, AuthTokenResponse, TrailBaseClient,
};

use gloo_net::http::{Method, Response};
use gloo_timers::future::TimeoutFuture;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::debug;

// Session lifecycle: refresh, get_fresh_session, logout, password change.
// HTTP transport layer: see trailbase_client.rs
//
// INVARIANT: On Tauri, the sync `get_session()` reads ONLY from the
// in-memory cache (no localStorage fallback). The cache is populated by
// `get_session_async()` during `check_session()` at app start, before
// `ProtectedRoute` renders any authenticated page. Therefore, by the time
// these methods run, the cache is guaranteed to be populated if the user
// is authenticated. See ADR-010 for details.

const REFRESH_THRESHOLD_SECONDS: u64 = 300;
const REFRESH_TIMEOUT_MS: u32 = 30000;
const REFRESH_RETRY_DELAY_MS: u32 = 100;

static REFRESH_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

pub fn is_refresh_in_progress() -> bool {
    REFRESH_IN_PROGRESS.load(Ordering::SeqCst)
}

pub fn set_refresh_in_progress(value: bool) {
    REFRESH_IN_PROGRESS.store(value, Ordering::SeqCst)
}

pub fn should_refresh_session(expires_at: u64) -> bool {
    let now = current_timestamp();
    now.saturating_add(REFRESH_THRESHOLD_SECONDS) >= expires_at
}

pub fn current_timestamp() -> u64 {
    chrono::Utc::now().timestamp().max(0) as u64
}

fn build_auth_headers(auth_token: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert(
        "Authorization".to_string(),
        format!("Bearer {}", auth_token),
    );
    headers
}

#[derive(serde::Serialize)]
struct Empty {}

impl TrailBaseClient {
    pub async fn refresh_session(
        &self,
        refresh_token: &str,
    ) -> Result<TrailBaseSession, AuthError> {
        #[derive(Serialize)]
        struct RefreshRequest<'a> {
            refresh_token: &'a str,
        }

        let response = self
            .fetch(
                "/api/auth/v1/refresh",
                Method::POST,
                Some(&RefreshRequest { refresh_token }),
                None,
            )
            .await?;

        if !response.ok() {
            return Err(AuthError::SessionExpired);
        }

        let token_response: AuthTokenResponse = Self::json(response).await?;
        let claims = decode_jwt_claims(&token_response.auth_token)
            .map_err(|e| AuthError::ApiError(format!("Failed to decode JWT: {}", e)))?;

        let now = current_timestamp();
        let expires_at = claims.expires_at(now.saturating_add(3600));

        let session = TrailBaseSession {
            auth_token: token_response.auth_token,
            refresh_token: token_response
                .refresh_token
                .unwrap_or_else(|| refresh_token.to_string()),
            email: claims.email.clone().unwrap_or_default(),
            trailbase_id: claims.sub.clone(),
            record_id: None,
            expires_at,
        };

        set_session_async(&session)
            .await
            .map_err(AuthError::ApiError)?;
        Ok(session)
    }

    async fn wait_for_refresh_completion(&self, timeout_ms: u32) -> Result<(), AuthError> {
        let start = current_timestamp();
        let timeout_secs = timeout_ms as u64 / 1000;

        loop {
            if !is_refresh_in_progress() {
                return Ok(());
            }

            let elapsed = current_timestamp().saturating_sub(start);
            if elapsed >= timeout_secs {
                return Err(AuthError::ApiError(
                    "Refresh coordination timeout".to_string(),
                ));
            }

            debug!("Waiting for refresh completion, elapsed: {}s", elapsed);
            TimeoutFuture::new(REFRESH_RETRY_DELAY_MS).await;
        }
    }

    async fn get_fresh_session(
        &self,
        original_session: TrailBaseSession,
    ) -> Result<TrailBaseSession, AuthError> {
        let session = get_session().ok_or(AuthError::SessionExpired)?;

        if session.expires_at != original_session.expires_at {
            debug!("Session refreshed by concurrent request");
            Ok(session)
        } else if should_refresh_session(session.expires_at) {
            if session.refresh_token.is_empty() {
                return Err(AuthError::SessionExpired);
            }
            self.refresh_session(&session.refresh_token).await
        } else {
            Ok(session)
        }
    }

    async fn ensure_fresh_session(
        &self,
        session: TrailBaseSession,
        label: &str,
    ) -> Result<TrailBaseSession, AuthError> {
        if !should_refresh_session(session.expires_at) {
            return Ok(session);
        }

        if session.refresh_token.is_empty() {
            return Err(AuthError::SessionExpired);
        }

        self.wait_for_refresh_completion(REFRESH_TIMEOUT_MS).await?;

        let session_after_wait = get_session().ok_or(AuthError::SessionExpired)?;

        if session_after_wait.expires_at != session.expires_at {
            debug!("{}: Session refreshed by concurrent request", label);
            return Ok(session_after_wait);
        }

        let acquired =
            REFRESH_IN_PROGRESS.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);

        if acquired.is_ok() {
            debug!("{}: Acquired refresh lock", label);

            let result = self
                .refresh_session(&session_after_wait.refresh_token)
                .await;

            set_refresh_in_progress(false);
            debug!("{}: Released refresh lock", label);

            result
        } else {
            debug!("{}: Another concurrent refresh in progress, waiting", label);
            self.wait_for_refresh_completion(REFRESH_TIMEOUT_MS).await?;
            self.get_fresh_session(session).await
        }
    }

    async fn _request_with_auth_impl<T: Serialize>(
        &self,
        path: &str,
        method: Method,
        body: Option<&T>,
    ) -> Result<Response, AuthError> {
        let session = get_session().ok_or(AuthError::SessionExpired)?;
        let session = self.ensure_fresh_session(session, "pre-request").await?;

        let headers = build_auth_headers(&session.auth_token);

        let response = self
            .fetch(path, method.clone(), body, Some(headers))
            .await?;

        if response.status() != 401 {
            return Ok(response);
        }

        let session = get_session().ok_or(AuthError::SessionExpired)?;
        let refreshed = self.ensure_fresh_session(session, "401-retry").await?;

        let headers = build_auth_headers(&refreshed.auth_token);

        self.fetch(path, method, body, Some(headers)).await
    }

    pub async fn logout(&self) -> Result<(), String> {
        if let Some(session) = get_session() {
            let headers = build_auth_headers(&session.auth_token);
            let _ = self
                .fetch(
                    "/api/auth/v1/logout",
                    Method::POST,
                    Some(&Empty {}),
                    Some(headers),
                )
                .await;
        }
        clear_session_async().await;
        Ok(())
    }

    pub async fn change_password(
        &self,
        old_password: &str,
        new_password: &str,
        new_password_repeat: &str,
    ) -> Result<(), AuthError> {
        #[derive(Serialize)]
        struct ChangePasswordRequest<'a> {
            old_password: &'a str,
            new_password: &'a str,
            new_password_repeat: &'a str,
        }

        let response = self
            .request_with_auth(
                "/api/auth/v1/change_password",
                Method::POST,
                Some(&ChangePasswordRequest {
                    old_password,
                    new_password,
                    new_password_repeat,
                }),
            )
            .await?;

        if !response.ok() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AuthError::ApiError(format!(
                "Password change failed: {}",
                error_text
            )));
        }

        Ok(())
    }
}

impl AuthRequestClient for TrailBaseClient {
    async fn request_with_auth<T: Serialize>(
        &self,
        path: &str,
        method: Method,
        body: Option<&T>,
    ) -> Result<Response, AuthError> {
        self._request_with_auth_impl(path, method, body).await
    }
}
