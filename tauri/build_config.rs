//! Build-time configuration parameterization.
//!
//! Rust-side source for the CSP injected into Tauri's config via the
//! `TAURI_CONFIG` env var (RFC 7396 JSON Merge Patch). See ADR-009 for the full
//! rationale, including the residual sync points that make this a "reduced
//! duplication" rather than a true single source of truth.
//!
//! `DEFAULT_TRAILBASE` and `DEFAULT_CDN` are shared with `origa_ui` (the hosts
//! needed by BOTH crates) via the root `build_defaults.rs` `#[path]`-include.
//! `DEFAULT_LANDING` stays local — `origa_ui` does not reference the landing
//! host, and moving it to the shared file would create an unused constant there
//! (the project forbids `#[allow(dead_code)]`). See ADR-020 (TRAILBASE) and
//! ADR-023 (CDN) for the full rationale.
//!
//! This module is pure (no I/O, no env access) so that it can be unit-tested
//! via `#[path]` from `tauri/tests/build_config.rs`. All env var resolution
//! lives in `build.rs`.
//!
//! `tauri/capabilities/default.json` is NOT parameterized here — build scripts
//! must not mutate committed source files. Its opener allow-list is validated
//! against a separate template living in `tauri/tests/build_config.rs`.

#[path = "../build_defaults.rs"]
mod defaults;

pub(crate) use defaults::{DEFAULT_CDN, DEFAULT_TRAILBASE};

/// Production landing URL. Used when `ORIGA_LANDING_BASE_URL` env var is unset.
pub(crate) const DEFAULT_LANDING: &str = "https://origa.uwuwu.net";

/// Production Sentry ingest host. Used when `SENTRY_DSN` is unset OR when the
/// DSN cannot be parsed. Pinned to the exact host (not a `*.sentry.io`
/// wildcard) to avoid exfiltration to attacker-controlled Sentry projects on
/// the multi-tenant `sentry.io` SaaS — see ADR-036 §7. CI overrides this with
/// the host parsed from `SENTRY_DSN` when that env var is set.
pub(crate) const DEFAULT_SENTRY_INGEST_HOST: &str = "o4511840951992320.ingest.us.sentry.io";

/// Build the Content-Security-Policy directive by substituting env-controlled
/// hosts into the static template. The CDN host is referenced from `font-src`
/// because all fonts are self-hosted there (ADR-028). Other third-party hosts
/// (huggingface, cdn.pyke.io for ort-rs WASM, signal.pyke.io for pyke SDK
/// telemetry, OAuth providers) remain hardcoded — they are not
/// environment-dependent.
///
/// Sentry's loader host (`js.sentry-cdn.com`) is static, as is the SDK
/// bundle host (`browser.sentry-cdn.com`). The ingest host is pinned to
/// the project's exact host (extracted from the DSN at build time by
/// `build.rs` and passed here as `sentry_ingest_host`). Pinning to a
/// single host rather than `*.sentry.io` avoids the exfiltration vector
/// of a wildcard on the multi-tenant `sentry.io` SaaS — see ADR-036 §7.
///
/// Session Replay requires three extra directives beyond the loader /
/// bundle / ingest hosts:
/// - `connect-src data:` — the SDK encodes compression payloads as
///   `data:` URLs (getsentry/sentry-javascript#4468).
/// - `worker-src 'self' blob:` — Replay runs its compression in a
///   blob-URL Web Worker.
/// - `child-src 'self' blob:` — fallback for browsers that do not
///   implement `worker-src` yet.
///
/// The literal is kept on a single line because `rustfmt` does not reflow
/// string-literal contents, and byte-equality with `tauri.conf.json` must hold
/// (verified by `build_csp_with_production_defaults_matches_committed_tauri_conf`).
pub(crate) fn build_csp(
    cdn: &str,
    landing: &str,
    trailbase: &str,
    sentry_ingest_host: &str,
) -> String {
    format!(
        "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdn.pyke.io https://js.sentry-cdn.com https://browser.sentry-cdn.com; connect-src 'self' ipc: http://ipc.localhost data: {cdn} {landing} {trailbase} https://huggingface.co https://signal.pyke.io https://cdn.pyke.io https://browser.sentry-cdn.com https://{sentry_ingest_host}; img-src 'self' data: blob: {cdn}; media-src 'self' blob: data: {cdn}; style-src 'self' 'unsafe-inline'; font-src 'self' {cdn}; form-action 'self' https://accounts.google.com https://oauth.yandex.ru; frame-ancestors 'none'; worker-src 'self' blob:; child-src 'self' blob:"
    )
}

/// Apply a RFC 7396 JSON Merge Patch to `target` in place.
///
/// serde_json (1.0.150 at time of writing) does NOT expose a public `merge`
/// API on `Value`, so this is a minimal local implementation of RFC 7396 §2
/// ("MergePatch"). Lives in this pure module so that it can be unit-tested via
/// `#[path]` from `tauri/tests/build_config.rs` (build scripts themselves are
/// not covered by `cargo test`).
///
/// Semantics (RFC 7396 §2). If `patch` is not an Object, it replaces `target`
/// entirely. If `patch` is an Object, `target` is forced to an Object (a
/// non-object target becomes empty), then for each `(key, value)` in `patch`:
/// when `value == null`, `key` is removed from `target` (no-op if absent);
/// otherwise `target[key]` is recursively merged with `value`. A missing
/// `target[key]` is treated as `null`, which the recursive call promotes to an
/// empty object before merging (per RFC 7396).
///
/// Recursion depth is bounded by the nesting depth of `patch`. For the inputs
/// used here (a flat Tauri CLI merge patch + a 3-level-deep CSP patch) this is
/// trivially stack-safe.
pub(crate) fn apply_merge_patch(target: &mut serde_json::Value, patch: serde_json::Value) {
    use serde_json::{Map, Value};

    let patch_obj = match patch {
        Value::Object(map) => map,
        // Non-object patch replaces the target entirely (RFC 7396 §2).
        other => {
            *target = other;
            return;
        },
    };

    // Force target to be an Object so we can merge into it (RFC 7396 §2: a
    // non-object target is replaced by an empty object when the patch is an
    // object).
    if !target.is_object() {
        *target = Value::Object(Map::new());
    }
    let target_obj = target
        .as_object_mut()
        .expect("target is Object (forced above; invariant cannot be violated)");

    for (key, value) in patch_obj {
        if value.is_null() {
            // RFC 7396: a null value in the patch deletes the key.
            target_obj.remove(&key);
        } else {
            // `entry().or_insert(Value::Null)` yields an existing slot or
            // inserts Null for a missing key. The recursive call then merges
            // `value` into it. RFC 7396 treats a missing target key as an
            // implicit empty object, which the recursion materializes correctly
            // (non-object target → empty object on the next call).
            let slot = target_obj.entry(key).or_insert(Value::Null);
            apply_merge_patch(slot, value);
        }
    }
}

/// Resolve a build-script env var to a non-empty value, falling back to
/// `default` when the var is unset OR set to an empty string.
///
/// `env::var(...).unwrap_or_else(|_| default)` catches only the UNSET case
/// (the `Err`); for a var SET to `""` it returns `Ok("")` and the fallback is
/// skipped, yielding an empty value downstream. An empty host dropped it from
/// the CSP (WebView then blocked requests to that host). Treating empty as
/// "use default" closes the hole. This is the canonical primitive of the
/// build-script env-var pattern — see ADR-024 for the generalized principle
/// and ADR-020 for the per-var precedent (TRAILBASE in `origa_ui`).
pub(crate) fn resolve_env(env_value: Option<&str>, default: &str) -> String {
    match env_value {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => default.to_string(),
    }
}

/// Extract the ingest host from a Sentry DSN for use as a CSP `connect-src`
/// entry. DSN format: `https://<key>@<host>/<project_id>` (the `<project_id>`
/// segment may itself contain `/`). Returns `None` for malformed inputs so
/// `build.rs` can fall back to `DEFAULT_SENTRY_INGEST_HOST`.
///
/// Used to pin the CSP to the project's exact ingest host rather than a
/// `*.sentry.io` wildcard — see ADR-036 §7.
pub(crate) fn extract_sentry_ingest_host(dsn: &str) -> Option<&str> {
    let after_at = dsn.split_once('@')?.1;
    let host = after_at.split_once('/')?.0;
    if host.is_empty() {
        return None;
    }
    Some(host)
}
