use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{OrigaError, User};
use origa::traits::UserRepository;

use crate::i18n::{I18nContext, Locale};
use crate::pages::login::auth_handlers::get_or_create_profile;
use crate::repository::{
    AuthError, HybridUserRepository, TrailBaseClient, clear_session, clear_session_async,
    get_session_async, set_session_async,
    trailbase_session::{is_refresh_in_progress, set_refresh_in_progress, should_refresh_session},
};

/// AuthStore - centralized authentication state management
/// Single source of truth for:
/// - Current user (None = not authenticated)
/// - Loading states
/// - Auth actions (login, logout, delete)
#[derive(Clone)]
pub struct AuthStore {
    client: TrailBaseClient,
    repository: HybridUserRepository,

    /// Current authenticated user (None = not logged in)
    pub user: RwSignal<Option<User>>,

    /// App is checking session on startup
    pub is_checking_session: RwSignal<bool>,

    /// OAuth flow in progress
    pub is_oauth_loading: RwSignal<bool>,

    /// Last OAuth error message (shown on login page)
    pub oauth_error: RwSignal<Option<String>>,

    /// Sync operation in progress
    pub is_syncing: RwSignal<bool>,

    // --- Granular data loading signals ---
    pub is_vocabulary_loaded: RwSignal<bool>,
    pub is_kanji_loaded: RwSignal<bool>,
    pub is_grammar_loaded: RwSignal<bool>,
    pub is_radicals_loaded: RwSignal<bool>,
    pub is_phrases_loaded: RwSignal<bool>,
    pub is_pitch_audio_loaded: RwSignal<bool>,
    /// Dictionary tokenizer (SudachiDict) loaded
    pub is_dictionary_loaded: RwSignal<bool>,
    pub is_furigana_loaded: RwSignal<bool>,
    pub is_jlpt_content_loaded: RwSignal<bool>,

    /// Guard against triggering start_dictionary_loading multiple times
    pub is_data_loading_started: RwSignal<bool>,

    /// Logout in progress (prevents race conditions)
    is_logging_out: RwSignal<bool>,

    /// Delete account in progress
    is_deleting_account: RwSignal<bool>,
}

impl AuthStore {
    /// Create new AuthStore with default dependencies
    pub fn new() -> Self {
        Self {
            client: TrailBaseClient::new(),
            repository: HybridUserRepository::new(),
            user: RwSignal::new(None),
            is_checking_session: RwSignal::new(true),
            is_oauth_loading: RwSignal::new(false),
            oauth_error: RwSignal::new(None),
            is_syncing: RwSignal::new(false),
            is_vocabulary_loaded: RwSignal::new(false),
            is_kanji_loaded: RwSignal::new(false),
            is_grammar_loaded: RwSignal::new(false),
            is_radicals_loaded: RwSignal::new(false),
            is_phrases_loaded: RwSignal::new(false),
            is_pitch_audio_loaded: RwSignal::new(false),
            is_dictionary_loaded: RwSignal::new(false),
            is_furigana_loaded: RwSignal::new(false),
            is_jlpt_content_loaded: RwSignal::new(false),
            is_data_loading_started: RwSignal::new(false),
            is_logging_out: RwSignal::new(false),
            is_deleting_account: RwSignal::new(false),
        }
    }

    // ========================================
    // Computed / Derived State
    // ========================================

    /// Returns a reactive Memo indicating if user is authenticated
    /// Use this in views: <Show when=move || auth_store.is_authenticated().get()>
    pub fn is_authenticated(&self) -> Memo<bool> {
        let user = self.user;
        Memo::new(move |_| user.with(|u| u.is_some()))
    }

    /// Returns a reactive Memo indicating if we're in loading state
    pub fn is_loading(&self) -> Memo<bool> {
        let is_checking_session = self.is_checking_session;
        let is_syncing = self.is_syncing;
        Memo::new(move |_| is_checking_session.get() || is_syncing.get())
    }

    /// Returns a reactive Memo indicating if ALL data resources are loaded
    pub fn is_all_data_loaded(&self) -> Memo<bool> {
        let v = self.is_vocabulary_loaded;
        let k = self.is_kanji_loaded;
        let g = self.is_grammar_loaded;
        let r = self.is_radicals_loaded;
        let p = self.is_phrases_loaded;
        let pa = self.is_pitch_audio_loaded;
        let d = self.is_dictionary_loaded;
        let f = self.is_furigana_loaded;
        let j = self.is_jlpt_content_loaded;
        Memo::new(move |_| {
            v.get()
                && k.get()
                && g.get()
                && r.get()
                && p.get()
                && pa.get()
                && d.get()
                && f.get()
                && j.get()
        })
    }

    /// Get repository for use cases
    pub fn repository(&self) -> &HybridUserRepository {
        &self.repository
    }

    /// Get TrailBase client for auth operations
    pub fn client(&self) -> &TrailBaseClient {
        &self.client
    }

    /// Load user after successful authentication
    async fn load_user_after_auth(
        &self,
        user_signal: RwSignal<Option<User>>,
    ) -> Result<(), OrigaError> {
        match self.repository.get_current_user().await {
            Ok(Some(user)) => {
                user_signal.set(Some(user));
                Ok(())
            },
            Ok(None) => {
                if self.repository.merge_current_user().await.is_ok()
                    && let Ok(Some(user)) = self.repository.get_current_user().await
                {
                    user_signal.set(Some(user));
                }
                Ok(())
            },
            Err(e) => {
                tracing::error!("Failed to load user: {:?}", e);
                Err(e)
            },
        }
    }

    // ========================================
    // Initialization
    // ========================================

    /// Check existing session from LocalStorage on app start.
    /// Offline-first: loads user from IndexedDB immediately, validates session in background.
    pub fn check_session(&self) {
        let user_signal = self.user;
        let is_checking = self.is_checking_session;

        if is_refresh_in_progress() {
            tracing::debug!("Refresh already in progress, skipping check_session");
            is_checking.set(false);
            return;
        }

        let client = self.client.clone();
        let repository = self.repository.clone();
        let store = self.clone();

        spawn_local(async move {
            let session = match get_session_async().await {
                Some(s) => s,
                None => {
                    is_checking.set(false);
                    return;
                },
            };

            let local_user = repository.get_current_user().await;

            match local_user {
                Ok(Some(user)) => {
                    if session.email != user.email() {
                        tracing::warn!(
                            "Session email ({}) does not match local user email ({}), fetching from server",
                            session.email,
                            user.email()
                        );
                        let _ = store.load_user_after_auth(user_signal).await;
                        is_checking.set(false);
                        return;
                    }
                    tracing::debug!("Loaded user from local storage: {}", user.id());
                    user_signal.set(Some(user));
                    is_checking.set(false);

                    if should_refresh_session(session.expires_at) {
                        let client_bg = client.clone();
                        let user_signal_bg = user_signal;
                        let is_checking_bg = is_checking;
                        spawn_local(async move {
                            if session.refresh_token.is_empty() {
                                tracing::warn!(
                                    "Session needs refresh but no refresh_token available"
                                );
                                return;
                            }

                            tracing::debug!("Background session refresh started");
                            set_refresh_in_progress(true);

                            match client_bg.refresh_session(&session.refresh_token).await {
                                Ok(new_session) => {
                                    tracing::info!("Background session refresh succeeded");
                                    if let Err(e) = set_session_async(&new_session).await {
                                        tracing::error!(
                                            "Failed to save refreshed session: {:?}",
                                            e
                                        );
                                    }
                                    let _ = store.load_user_after_auth(user_signal_bg).await;
                                },
                                Err(AuthError::SessionExpired) => {
                                    tracing::warn!("Session definitively expired, logging out");
                                    clear_session_async().await;
                                    user_signal_bg.set(None);
                                    is_checking_bg.set(false);
                                },
                                Err(e) => {
                                    tracing::warn!(
                                        "Background session refresh failed (will retry later): {:?}",
                                        e
                                    );
                                },
                            }

                            set_refresh_in_progress(false);
                        });
                    }
                },
                Ok(None) => {
                    tracing::debug!("No local user found, fetching from server");
                    let _ = store.load_user_after_auth(user_signal).await;
                    is_checking.set(false);
                },
                Err(e) => {
                    tracing::error!("Failed to read local user: {:?}", e);
                    let _ = store.load_user_after_auth(user_signal).await;
                    is_checking.set(false);
                },
            }
        });
    }

    // ========================================
    // Auth Actions (Atomic)
    // ========================================

    /// Login with email/password
    pub async fn login(
        &self,
        email: &str,
        password: &str,
        i18n: &I18nContext<Locale>,
    ) -> Result<(), OrigaError> {
        self.is_syncing.set(true);

        let result = self.client.login_with_email_password(email, password).await;

        match result {
            Ok(_) => {
                let session =
                    get_session_async()
                        .await
                        .ok_or_else(|| OrigaError::RepositoryError {
                            reason: "Session not found after login".to_string(),
                        })?;

                match get_or_create_profile(self, &session.email, i18n).await {
                    Ok(user) => {
                        self.user.set(Some(user));
                        self.is_syncing.set(false);
                        Ok(())
                    },
                    Err(e) => {
                        self.is_syncing.set(false);
                        Err(OrigaError::NetworkError {
                            url: "/api/auth/v1/login".to_string(),
                            reason: e,
                        })
                    },
                }
            },
            Err(e) => {
                self.is_syncing.set(false);
                Err(OrigaError::NetworkError {
                    url: "/api/auth/v1/login".to_string(),
                    reason: e.to_string(),
                })
            },
        }
    }

    /// Set session after OAuth callback
    pub async fn set_oauth_session(
        &self,
        code: &str,
        pkce_verifier: &str,
        i18n: &I18nContext<Locale>,
    ) -> Result<(), OrigaError> {
        if self.user.with(|u| u.is_some()) {
            self.is_oauth_loading.set(false);
            return Ok(());
        }

        self.is_oauth_loading.set(true);

        match self
            .client
            .exchange_auth_code_for_session(code, pkce_verifier)
            .await
        {
            Ok(session) => {
                if session.email.is_empty() {
                    self.is_oauth_loading.set(false);
                    return Err(OrigaError::NetworkError {
                        url: "/api/auth/v1/token".to_string(),
                        reason: "Email not found in OAuth token".to_string(),
                    });
                }

                match get_or_create_profile(self, &session.email, i18n).await {
                    Ok(user) => {
                        self.user.set(Some(user));
                        self.is_oauth_loading.set(false);
                        Ok(())
                    },
                    Err(e) => {
                        self.is_oauth_loading.set(false);
                        Err(OrigaError::InvalidValues { reason: e })
                    },
                }
            },
            Err(e) => {
                self.is_oauth_loading.set(false);
                Err(OrigaError::NetworkError {
                    url: "/api/auth/v1/token".to_string(),
                    reason: e.to_string(),
                })
            },
        }
    }

    /// Logout - atomically clears all auth state
    pub async fn logout(&self) -> Result<(), OrigaError> {
        if self.is_logging_out.get() {
            return Ok(());
        }

        self.is_logging_out.set(true);

        let _ = self.client.logout().await;

        self.clear_auth_state().await;

        self.is_logging_out.set(false);
        Ok(())
    }

    /// Delete account - atomically removes account and all local data
    pub async fn delete_account(&self) -> Result<(), OrigaError> {
        if self.is_deleting_account.get() {
            return Ok(());
        }

        self.is_deleting_account.set(true);

        // Capture user id before clearing auth state — we need it for the
        // local data cleanup after the remote record is gone. If there is no
        // loaded user, the account is in an anomalous state (authenticated
        // session but no domain User) — abort before touching the server.
        let user_id = self.user.with(|u| u.clone().map(|user| user.id()));

        if let Err(e) = self.repository.delete_remote(user_id).await {
            tracing::error!("Server account delete failed: {:?}", e);
            self.clear_auth_state().await;
            self.is_deleting_account.set(false);
            return Err(OrigaError::AccountDeletionFailed {
                reason: e.to_string(),
            });
        }

        // Invalidate the auth session on the server side. Best-effort — the
        // record is already deleted, so a lingering session token is harmless.
        let _ = self.client.logout().await;

        self.clear_auth_state().await;

        if let Some(id) = user_id
            && let Err(e) = self.repository.delete(id).await
        {
            tracing::error!(
                "Local user data delete failed after successful server deletion: {:?}",
                e
            );
        }

        self.is_deleting_account.set(false);
        Ok(())
    }

    fn reset_data_loading_signals(&self) {
        self.is_vocabulary_loaded.set(false);
        self.is_kanji_loaded.set(false);
        self.is_grammar_loaded.set(false);
        self.is_radicals_loaded.set(false);
        self.is_phrases_loaded.set(false);
        self.is_pitch_audio_loaded.set(false);
        self.is_dictionary_loaded.set(false);
        self.is_furigana_loaded.set(false);
        self.is_jlpt_content_loaded.set(false);
        self.is_data_loading_started.set(false);
    }

    /// Internal: Clear all authentication-related state
    async fn clear_auth_state(&self) {
        clear_session_async().await;

        if let Some(user) = self.user.get() {
            let _ = self.repository.delete(user.id()).await;
        }

        self.user.set(None);
        self.reset_data_loading_signals();
    }

    // ========================================
    // Data Sync
    // ========================================

    /// Refresh user data from local storage
    pub async fn refresh_user(&self) -> Result<(), OrigaError> {
        match self.repository.get_current_user().await {
            Ok(Some(user)) => {
                self.user.set(Some(user));
                Ok(())
            },
            Ok(None) => Err(OrigaError::CurrentUserNotExist),
            Err(e) => Err(e),
        }
    }

    // ========================================
    // Session Expiry Handling
    // ========================================

    /// Handle session expiry - clears all auth state
    /// Call this when AuthError::SessionExpired is received
    ///
    /// NOTE: This function uses the sync `clear_session()` which clears the
    /// cache + localStorage but NOT the persistent Tauri store. When this
    /// handler is eventually wired to real error paths, it should use
    /// `clear_session_async()` instead (see ADR-010).
    pub fn handle_session_expiry(&self) {
        tracing::debug!("Handling session expiry - clearing auth state");

        clear_session();
        self.user.set(None);
        self.reset_data_loading_signals();
        self.is_checking_session.set(false);
    }

    #[expect(dead_code, reason = "prepared for future error handling")]
    pub fn handle_error_if_session_expired(&self, error: &AuthError) -> bool {
        match error {
            AuthError::SessionExpired => {
                self.handle_session_expiry();
                true
            },
            _ => false,
        }
    }
}

impl Default for AuthStore {
    fn default() -> Self {
        Self::new()
    }
}
