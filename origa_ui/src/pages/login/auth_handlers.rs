use crate::i18n::{I18nContext, Locale, locale_to_native_language};
use crate::repository::{
    TrailBaseClient, get_session, set_session_async, take_pkce_verifier_async, uuid_to_ulid,
};
use crate::store::auth_store::AuthStore;
use chrono::Utc;
use origa::domain::{DailyLoad, JlptProgress, KnowledgeSet, User};
use origa::traits::UserRepository;
use std::collections::HashSet;

pub async fn get_or_create_profile(
    auth_store: &AuthStore,
    email: &str,
    i18n: &I18nContext<Locale>,
) -> Result<User, String> {
    auth_store
        .repository()
        .merge_current_user()
        .await
        .map_err(|e| {
            i18n.get_keys_untracked()
                .login()
                .sync_profile_error()
                .inner()
                .replace("{}", &e.to_string())
        })?;

    match auth_store.repository().get_current_user().await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => {
            let new_user = create_new_user_from_session(email, i18n)?;

            // First-time profile creation is an explicit sync checkpoint: the
            // canonical user must exist remotely before any other device can
            // log in and merge against it.
            auth_store
                .repository()
                .save_sync(&new_user)
                .await
                .map_err(|e| {
                    i18n.get_keys_untracked()
                        .login()
                        .create_profile_error()
                        .inner()
                        .replace("{}", &e.to_string())
                })?;

            auth_store
                .repository()
                .get_current_user()
                .await
                .map_err(|e| {
                    i18n.get_keys_untracked()
                        .login()
                        .load_profile_error()
                        .inner()
                        .replace("{}", &e.to_string())
                })?
                .ok_or_else(|| {
                    i18n.get_keys_untracked()
                        .login()
                        .profile_not_found()
                        .inner()
                        .to_string()
                })
        },
        Err(e) => Err(i18n
            .get_keys_untracked()
            .login()
            .load_profile_error()
            .inner()
            .replace("{}", &e.to_string())),
    }
}

/// Build a brand-new local user whose identity is pinned to the TrailBase
/// session id (decoded from the JWT `sub`).
///
/// `User::new` mints a random ULID, which would diverge from the server-assigned
/// id and re-introduce the cross-device sync bug where every device ended up
/// with its own key. We instead derive the id from `session.trailbase_id` so the
/// first save is already attributed to the canonical user. Field defaults mirror
/// `User::new`; they are duplicated here only because identity is restricted to
/// the `merge` change in the domain layer.
///
/// The native language is inherited from the login screen's language selection:
/// the caller passes the very `I18nContext` the login page rendered with, so a
/// brand-new profile starts in the language the user has already picked —
/// otherwise the App-level locale sync would immediately overwrite that choice
/// with this default (see the login language race). The context is an explicit
/// argument (not `use_context`) because production calls this after `.await`
/// points inside `spawn_local`, where no reactive owner is scoped.
pub(super) fn create_new_user_from_session(
    email: &str,
    i18n: &I18nContext<Locale>,
) -> Result<User, String> {
    let session = get_session().ok_or_else(|| {
        "No active session: cannot create user profile without a TrailBase id".to_string()
    })?;

    let user_id = uuid_to_ulid(&session.trailbase_id);
    if user_id == ulid::Ulid::nil() {
        tracing::error!(
            "Session trailbase_id did not decode to a valid user id: {}",
            crate::repository::redact_id(&session.trailbase_id)
        );
        return Err("Invalid TrailBase session id: please log in again".to_string());
    }

    let username = email.split('@').next().unwrap_or(email).to_string();

    let native_language = locale_to_native_language(&i18n.get_locale_untracked());

    Ok(User::from_row(
        user_id,
        email.to_string(),
        username,
        JlptProgress::new(),
        native_language,
        None,
        KnowledgeSet::new(),
        Utc::now(),
        HashSet::new(),
        DailyLoad::default(),
        0,
    ))
}

pub async fn handle_oauth_callback(
    url_fragment: &str,
    auth_store: &AuthStore,
    i18n: &I18nContext<Locale>,
) -> Result<User, String> {
    let session = TrailBaseClient::parse_tokens_from_url(url_fragment)?;
    set_session_async(&session).await.map_err(|e| {
        i18n.get_keys_untracked()
            .login()
            .save_session_error()
            .inner()
            .replace("{}", &e.to_string())
    })?;

    if session.email.is_empty() {
        return Err(i18n
            .get_keys_untracked()
            .login()
            .email_not_in_token()
            .inner()
            .to_string());
    }

    get_or_create_profile(auth_store, &session.email, i18n).await
}

pub async fn handle_oauth_callback_desktop(
    url: &str,
    auth_store: &AuthStore,
    i18n: &I18nContext<Locale>,
) -> Result<Option<User>, String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;

    let code = parsed
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| {
            i18n.get_keys()
                .login()
                .auth_code_not_found()
                .inner()
                .to_string()
        })?;

    let Some(verifier) = take_pkce_verifier_async().await else {
        // Verifier already consumed → this callback was already processed
        // (redelivered after Activity recreate / WASM reload). Silent skip, not
        // an error, so a replayed callback does not surface pkce_not_found.
        return Ok(None);
    };

    let client = TrailBaseClient::new();
    let session = client
        .exchange_auth_code_for_session(&code, &verifier)
        .await
        .map_err(|e| {
            i18n.get_keys_untracked()
                .login()
                .token_exchange_error()
                .inner()
                .replace("{}", &e.to_string())
        })?;

    set_session_async(&session).await.map_err(|e| {
        i18n.get_keys_untracked()
            .login()
            .save_session_error()
            .inner()
            .replace("{}", &e.to_string())
    })?;

    if session.email.is_empty() {
        return Err(i18n
            .get_keys_untracked()
            .login()
            .email_not_in_token()
            .inner()
            .to_string());
    }

    get_or_create_profile(auth_store, &session.email, i18n)
        .await
        .map(Some)
}
