//! Display-name helpers shared by login, onboarding, profile and home.
//!
//! The username is a free-form display name, not a login identifier. It can
//! legitimately be empty (e.g. Apple "Hide My Email" accounts, whose opaque
//! relay local part makes for an unreadable name) — in that case the UI shows
//! the account email instead and the name inputs start blank with a
//! placeholder question.

/// Apple's "Hide My Email" relay domain: the local part is an opaque random
/// string (e.g. `g55jkfzf5p@privaterelay.appleid.com`).
const APPLE_RELAY_DOMAIN: &str = "@privaterelay.appleid.com";

pub fn is_apple_relay_email(email: &str) -> bool {
    email.to_ascii_lowercase().ends_with(APPLE_RELAY_DOMAIN)
}

/// Derives the initial `username` (display name) for a brand-new profile.
///
/// Regular emails seed the local part (`ivan.petrov@…` → `ivan.petrov`);
/// Apple relay emails seed an empty name — the user is then asked during
/// onboarding (the input's placeholder shows the question).
pub fn default_username_for_email(email: &str) -> String {
    if is_apple_relay_email(email) {
        return String::new();
    }

    email.split('@').next().unwrap_or(email).to_string()
}

/// True when the stored username is just the untouched local part of an Apple
/// relay email — the opaque noise these helpers replace. Covers legacy
/// profiles created before the onboarding name question existed.
pub fn is_untouched_relay_username(username: &str, email: &str) -> bool {
    is_apple_relay_email(email) && username == email.split('@').next().unwrap_or(email)
}

/// Initial value for a username input: an untouched relay local part is
/// treated as absent so the placeholder question shows instead of the noise.
pub fn editable_username_for(username: &str, email: &str) -> String {
    if is_untouched_relay_username(username, email) {
        return String::new();
    }

    username.to_string()
}

/// Name displayed in place of the raw username: falls back to the account
/// email when the name is blank or is still an untouched relay local part.
pub fn display_name_for(username: &str, email: &str) -> String {
    if username.trim().is_empty() || is_untouched_relay_username(username, email) {
        return email.to_string();
    }

    username.to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        default_username_for_email, display_name_for, editable_username_for,
        is_untouched_relay_username,
    };

    #[test]
    fn regular_email_seeds_local_part_as_name() {
        assert_eq!(
            default_username_for_email("ivan.petrov@example.com"),
            "ivan.petrov"
        );
    }

    #[test]
    fn apple_relay_email_seeds_empty_name() {
        assert_eq!(
            default_username_for_email("g55jkfzf5p@privaterelay.appleid.com"),
            ""
        );
    }

    #[test]
    fn apple_relay_detection_ignores_domain_case() {
        assert_eq!(
            default_username_for_email("g55jkfzf5p@PrivateRelay.AppleID.com"),
            ""
        );
    }

    #[test]
    fn email_without_at_sign_seeds_whole_string() {
        assert_eq!(default_username_for_email("no-at-sign"), "no-at-sign");
    }

    #[test]
    fn legacy_relay_prefix_counts_as_untouched() {
        assert!(is_untouched_relay_username(
            "g55jkfzf5p",
            "g55jkfzf5p@privaterelay.appleid.com"
        ));
        assert!(!is_untouched_relay_username(
            "Ivan",
            "g55jkfzf5p@privaterelay.appleid.com"
        ));
        // A regular email's prefix is a legitimate name, not noise.
        assert!(!is_untouched_relay_username("ivan", "ivan@example.com"));
    }

    #[test]
    fn editable_name_is_blank_for_untouched_relay_prefix() {
        assert_eq!(
            editable_username_for("g55jkfzf5p", "g55jkfzf5p@privaterelay.appleid.com"),
            ""
        );
        assert_eq!(
            editable_username_for("ivan.petrov", "x@example.com"),
            "ivan.petrov"
        );
    }

    #[test]
    fn display_name_falls_back_to_email_for_blank_or_untouched_relay_name() {
        assert_eq!(display_name_for("", "a@b.com"), "a@b.com");
        assert_eq!(
            display_name_for("g55jkfzf5p", "g55jkfzf5p@privaterelay.appleid.com"),
            "g55jkfzf5p@privaterelay.appleid.com"
        );
        assert_eq!(display_name_for("Иван", "a@b.com"), "Иван");
    }
}
