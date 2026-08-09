//! Shared build-time production defaults for hosts needed by BOTH the `tauri`
//! and `origa_ui` build scripts.
//!
//! Single source of truth for `DEFAULT_TRAILBASE` and `DEFAULT_CDN`,
//! `#[path]`-included by both `tauri/build_config.rs` and
//! `origa_ui/build_config.rs`. Build scripts cannot `use` workspace crates —
//! they compile with `std` + `[build-dependencies] only — so the existing
//! `#[path]`-include pattern (see `tauri/build.rs:20-21`) is reused instead of
//! introducing a workspace crate.
//!
//! `DEFAULT_LANDING` stays local to `tauri/build_config.rs` because only the
//! `tauri` build script references the landing host (for CSP injection).
//! Moving it here would create an unused constant inside the `origa_ui` build
//! binary — and the project forbids `#[allow(dead_code)]` (see `AGENTS.md`).
//! See ADR-020 (TRAILBASE) and ADR-023 (CDN) for the full rationale.

/// Production TrailBase URL. Used as the fallback when the `TRAILBASE_URL` env
/// var is unset OR empty (an empty value previously produced a relative fetch
/// URL the WebView resolved against its own origin — see ADR-020).
pub(crate) const DEFAULT_TRAILBASE: &str = "https://app.origa.uwuwu.net";

/// Production CDN base URL. Used as the fallback when the
/// `ORIGA_CDN_BASE_URL` env var is unset OR empty. An empty value previously
/// produced relative fetch URLs (`/grammar/grammar.json`) that the WebView
/// resolved against its own origin (`tauri.localhost`) instead of the CDN,
/// causing resources to be served from the local bundle and wrapped in blob
/// URLs by `CacheFirstCdnProvider`. See ADR-023.
pub(crate) const DEFAULT_CDN: &str = "https://s3.origa.uwuwu.net";
