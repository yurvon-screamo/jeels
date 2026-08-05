//! Drift-detection tests for `tauri/build_config.rs` and the committed
//! `tauri/capabilities/default.json`.
//!
//! Strategy: include the COMMITTED config files at compile time via
//! `include_str!` and compare against the generated output. If anyone edits
//! `tauri/tauri.conf.json` or `tauri/capabilities/default.json` without also
//! updating the templates, these tests will fail — this is intentional drift
//! detection, not a brittle snapshot.
//!
//! `build_capabilities_content` lives here (not in `build_config.rs`) because
//! `tauri/build.rs` must NOT mutate committed source files (Cargo contract):
//! the capabilities file stays static, and this template only serves as a
//! reference shape that drift-detection asserts against.

#[path = "../build_config.rs"]
mod build_config;

use build_config::{
    DEFAULT_CDN, DEFAULT_LANDING, DEFAULT_SENTRY_INGEST_HOST, DEFAULT_TRAILBASE, apply_merge_patch,
    build_csp, extract_sentry_ingest_host, resolve_env,
};

/// Reference template for `tauri/capabilities/default.json`. Byte-identical to
/// the committed file when called with production defaults. Lives in this test
/// module (not in `build_config.rs`) so that no production code path can mutate
/// the committed capabilities file.
///
/// CRITICAL: each `\t` here is a literal TAB character — the committed file is
/// tab-indented. `\` line-continuations strip only the newline + leading
/// whitespace on the next source line, so the `\t` that follows survives into
/// the output. There is intentionally NO trailing whitespace before any `\`
/// (rustfmt-safe and editor auto-trim-safe).
fn build_capabilities_content(landing: &str, trailbase: &str) -> String {
    let landing_url = format!("{landing}/*");
    let trailbase_url = format!("{trailbase}/*");
    format!(
        "{{\n\
\t\"$schema\": \"../gen/schemas/desktop-schema.json\",\n\
\t\"identifier\": \"default\",\n\
\t\"description\": \"Capability for the main window\",\n\
\t\"windows\": [\"main\"],\n\
\t\"permissions\": [\n\
\t\t\"core:default\",\n\
\t\t\"core:event:default\",\n\
\t\t\"tts:default\",\n\
\t\t\"deep-link:default\",\n\
\t\t\"updater:default\",\n\
\t\t{{\n\
\t\t\t\"identifier\": \"opener:default\",\n\
\t\t\t\"allow\": [\n\
\t\t\t\t{{ \"url\": \"{landing_url}\" }},\n\
\t\t\t\t{{ \"url\": \"{trailbase_url}\" }}\n\
\t\t\t]\n\
\t\t}}\n\
\t]\n\
}}\n"
    )
}

/// Verifies that `build_csp` with production defaults reproduces the exact CSP
/// string currently committed in `tauri/tauri.conf.json:24` (single line).
///
/// If this test fails, either the template drifted from the committed CSP or
/// the committed CSP was updated without updating the template.
#[test]
fn build_csp_with_production_defaults_matches_committed_tauri_conf() {
    let csp = build_csp(
        DEFAULT_CDN,
        DEFAULT_LANDING,
        DEFAULT_TRAILBASE,
        DEFAULT_SENTRY_INGEST_HOST,
    );

    let config: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json"))
        .expect("tauri.conf.json must be valid JSON");
    let committed_csp = config["app"]["security"]["csp"]
        .as_str()
        .expect("tauri.conf.json must contain app.security.csp");

    assert_eq!(csp, committed_csp);
}

/// Verifies that env-controlled hosts are substituted into CSP while
/// third-party hosts (huggingface, OAuth providers) are preserved. Fonts are
/// self-hosted on the CDN (ADR-028), so font-src carries the env CDN host.
#[test]
fn build_csp_substitutes_staging_hosts() {
    let csp = build_csp(
        "https://cdn.staging.example.com",
        "https://landing.staging.example.com",
        "https://api.staging.example.com",
        "o-staging.ingest.sentry.io",
    );

    assert!(csp.contains("https://cdn.staging.example.com"));
    assert!(csp.contains("https://landing.staging.example.com"));
    assert!(csp.contains("https://api.staging.example.com"));

    // Fonts are now self-hosted on the CDN (ADR-028), so font-src carries the
    // CDN host. Google Fonts CDN hosts are no longer referenced anywhere.
    assert!(csp.contains("font-src 'self' https://cdn.staging.example.com"));
    assert!(!csp.contains("fonts.googleapis.com"));
    assert!(!csp.contains("fonts.gstatic.com"));

    // Other third-party hosts must survive any host substitution.
    assert!(csp.contains("https://huggingface.co"));
    assert!(csp.contains("https://cdn.pyke.io"));
    assert!(csp.contains("https://signal.pyke.io"));
    assert!(csp.contains("https://accounts.google.com"));
    assert!(csp.contains("https://oauth.yandex.ru"));

    // Sentry: loader host is static, ingest host is parameterised. The ingest
    // host must be pinned to the exact staging value — NOT a wildcard — to
    // avoid the multi-tenant SaaS exfil vector. See ADR-036 §7.
    //
    // The loader (js.sentry-cdn.com) is a thin bootstrap that fetches the
    // actual SDK bundle from browser.sentry-cdn.com, so BOTH hosts must be in
    // script-src — listing only the loader leaves the bundle blocked by CSP.
    //
    // Session Replay additionally needs connect-src data: (compression
    // payload encoding), worker-src 'self' blob: (compression Web Worker),
    // and child-src 'self' blob: (worker-src fallback for older browsers).
    assert!(csp.contains("https://js.sentry-cdn.com"));
    assert!(csp.contains("https://browser.sentry-cdn.com"));
    assert!(csp.contains("https://o-staging.ingest.sentry.io"));
    assert!(
        !csp.contains("*.sentry.io"),
        "CSP must NOT contain a sentry.io wildcard — see ADR-036 §7"
    );
    assert!(
        csp.contains("connect-src 'self' ipc: http://ipc.localhost data:"),
        "connect-src must allow data: for Sentry Replay compression payloads"
    );
    assert!(
        csp.contains("worker-src 'self' blob:"),
        "worker-src must allow blob: for the Sentry Replay Web Worker"
    );
    assert!(
        csp.contains("child-src 'self' blob:"),
        "child-src must allow blob: as a worker-src fallback for older browsers"
    );

    // Production hosts must NOT leak into the staging build.
    assert!(!csp.contains(DEFAULT_CDN));
    assert!(!csp.contains(DEFAULT_TRAILBASE));
    assert!(!csp.contains(DEFAULT_SENTRY_INGEST_HOST));
}

/// Verifies that the reference template for `capabilities/default.json`,
/// invoked with production defaults, reproduces the exact bytes committed in
/// `tauri/capabilities/default.json` (tab-indented, 429 bytes, trailing
/// newline). Drift here means the committed file was hand-edited without
/// updating the template (or vice versa).
#[test]
fn capabilities_template_with_production_defaults_matches_committed_file() {
    let content = build_capabilities_content(DEFAULT_LANDING, DEFAULT_TRAILBASE);
    let committed = include_str!("../capabilities/default.json");

    assert_eq!(content, committed);
}

/// The mobile capability file must NOT carry `updater:default`: the updater
/// plugin is registered under `#[cfg(desktop)]` in `tauri/src/lib.rs`, so on
/// Android the capability is dead — it references a permission set that no
/// compiled plugin serves. Shipping it pollutes the security-review surface
/// (Play reviewers inspect the full capability set) without enabling any
/// runtime behaviour. This drift guard prevents a regression that reintroduces
/// `"updater:default"` into the mobile file.
#[test]
fn capabilities_mobile_has_no_updater_permission() {
    let committed = include_str!("../capabilities/mobile.json");

    assert!(
        !committed.contains("\"updater:default\""),
        "`tauri/capabilities/mobile.json` must not contain \"updater:default\": \
         the updater plugin is desktop-only (#[cfg(desktop)] in tauri/src/lib.rs), \
         so the mobile capability is dead and should not be committed. Got:\n{committed}"
    );

    serde_json::from_str::<serde_json::Value>(committed)
        .expect("capabilities/mobile.json must be valid JSON");
}

/// Verifies that env-controlled hosts are substituted into the capabilities
/// opener allow-list while preserving the surrounding permission structure,
/// and that the output is always valid JSON.
#[test]
fn capabilities_template_substitutes_staging_hosts() {
    let content = build_capabilities_content(
        "https://landing.staging.example.com",
        "https://api.staging.example.com",
    );

    assert!(content.contains("https://landing.staging.example.com/*"));
    assert!(content.contains("https://api.staging.example.com/*"));

    // The full permission structure must survive host substitution — these
    // permissions are env-independent and must never be dropped.
    assert!(content.contains("\"core:default\""));
    assert!(content.contains("\"core:event:default\""));
    assert!(content.contains("\"tts:default\""));
    assert!(content.contains("\"deep-link:default\""));
    assert!(content.contains("\"updater:default\""));

    // Production hosts must NOT leak into the staging build.
    assert!(!content.contains(DEFAULT_LANDING));
    assert!(!content.contains(DEFAULT_TRAILBASE));

    serde_json::from_str::<serde_json::Value>(&content)
        .expect("capabilities content must be valid JSON");
}

/// RFC 7396 §2: a non-object patch replaces the target entirely.
#[test]
fn apply_merge_patch_non_object_patch_replaces_target() {
    let mut target: serde_json::Value = serde_json::json!({"a": 1, "b": 2});
    apply_merge_patch(&mut target, serde_json::json!("replacement"));

    assert_eq!(target, serde_json::json!("replacement"));
}

/// RFC 7396 §2: a null value in the patch deletes the key from the target.
#[test]
fn apply_merge_patch_null_value_deletes_key() {
    let mut target: serde_json::Value = serde_json::json!({"a": 1, "b": 2, "c": 3});
    // `b: null` deletes `b`; `d: null` is a no-op (key absent).
    apply_merge_patch(&mut target, serde_json::json!({"b": null, "d": null}));

    assert_eq!(target, serde_json::json!({"a": 1, "c": 3}));
}

/// RFC 7396 §2: a non-object target is replaced by an empty object when the
/// patch is an object, then the patch is merged into it.
#[test]
fn apply_merge_patch_object_patch_forces_non_object_target_to_object() {
    let mut target: serde_json::Value = serde_json::json!("not-an-object");
    apply_merge_patch(&mut target, serde_json::json!({"x": 42}));

    assert_eq!(target, serde_json::json!({"x": 42}));
}

/// RFC 7396 §2: objects are merged recursively — sibling nested keys survive,
/// and a nested object key present only in the patch is added wholesale.
#[test]
fn apply_merge_patch_deep_merge_preserves_nested_keys() {
    let mut target: serde_json::Value = serde_json::json!({
        "app": { "windows": [{"title": "old"}], "version": "1.0" },
        "bundle": { "identifier": "com.origa" }
    });
    apply_merge_patch(
        &mut target,
        serde_json::json!({
            "app": { "security": { "csp": "new-csp" } }
        }),
    );

    // Nested `app.windows` and `app.version` survive; `app.security.csp` is added.
    assert_eq!(target["app"]["windows"][0]["title"], "old");
    assert_eq!(target["app"]["version"], "1.0");
    assert_eq!(target["app"]["security"]["csp"], "new-csp");
    // Sibling top-level key survives untouched.
    assert_eq!(target["bundle"]["identifier"], "com.origa");
}

/// End-to-end simulation of the `tauri/build.rs` scenario: the Tauri CLI sets
/// `TAURI_CONFIG` with a flavor/beta config (productName, bundle, devUrl), and
/// our build script merges a CSP patch INTO it. The result must carry BOTH the
/// external overrides AND the CSP — flavor overrides must NOT be silently
/// dropped. This is the regression guard for the GATE-2 review finding.
#[test]
fn apply_merge_patch_csp_into_tauri_cli_config_preserves_overrides() {
    // Simulates what `tauri-cli/src/helpers/config.rs::load_config` produces
    // from `cargo tauri build --config '{"productName":"Origa Beta",...}'`.
    let tauri_cli_config = serde_json::json!({
        "productName": "Origa Beta",
        "version": "2.0.0-beta",
        "bundle": {
            "identifier": "net.uwuwu.origa.beta",
            "targets": ["nsis", "dmg"]
        },
        "build": {
            "devUrl": "http://localhost:1420"
        }
    });
    let mut target = tauri_cli_config.clone();

    // The CSP patch our build.rs builds (only `app.security.csp`).
    let csp_patch = serde_json::json!({ "app": { "security": { "csp": "default-src 'self'" } } });
    apply_merge_patch(&mut target, csp_patch);

    // External overrides must survive.
    assert_eq!(target["productName"], "Origa Beta");
    assert_eq!(target["version"], "2.0.0-beta");
    assert_eq!(target["bundle"]["identifier"], "net.uwuwu.origa.beta");
    assert_eq!(target["bundle"]["targets"][0], "nsis");
    assert_eq!(target["build"]["devUrl"], "http://localhost:1420");
    // CSP must be present (the whole point of the patch).
    assert_eq!(target["app"]["security"]["csp"], "default-src 'self'");
}

/// Regression guard for the no-signing-key patch applied when
/// `TAURI_SIGNING_PRIVATE_KEY` is empty (Dependabot PRs, local dev without a
/// key). The committed `tauri.conf.json` has `createUpdaterArtifacts: true`;
/// the build script patches it to `false` so the bundler skips signing.
/// Without this patch, Tauri fails with "failed to decode secret key:
/// … Missing comment in secret key" because the empty key is invalid.
///
/// This test simulates the exact merge: load the committed config, apply the
/// CSP patch, then apply the no-sign patch, and assert the final result.
#[test]
fn apply_merge_patch_no_signing_key_disables_updater_artifacts() {
    // Start from the committed tauri.conf.json (includes
    // createUpdaterArtifacts: true).
    let mut target: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json"))
        .expect("tauri.conf.json must be valid JSON");

    // The CSP patch our build.rs applies first.
    let csp_patch = serde_json::json!({
        "app": { "security": { "csp": "default-src 'self'" } }
    });
    apply_merge_patch(&mut target, csp_patch);

    // The no-signing-key patch our build.rs applies when
    // TAURI_SIGNING_PRIVATE_KEY is empty.
    let no_sign_patch = serde_json::json!({
        "bundle": { "createUpdaterArtifacts": false }
    });
    apply_merge_patch(&mut target, no_sign_patch);

    // The patch must flip createUpdaterArtifacts to false.
    assert_eq!(
        target["bundle"]["createUpdaterArtifacts"], false,
        "no-signing-key patch must disable createUpdaterArtifacts"
    );
    // CSP must survive (applied first, not overwritten).
    assert_eq!(target["app"]["security"]["csp"], "default-src 'self'");
    // Other bundle keys (icon, targets) must survive the merge.
    assert_eq!(
        target["bundle"]["targets"], "all",
        "bundle.targets must survive the no-signing-key patch"
    );
}

/// `resolve_env` falls back to the default when the env var is unset (`None`).
/// Asserting against the literal CDN host (not the `DEFAULT_CDN` constant)
/// makes this a real drift guard: changing the constant breaks the assertion.
#[test]
fn resolve_env_uses_default_when_unset() {
    assert_eq!(resolve_env(None, DEFAULT_CDN), "https://s3.origa.uwuwu.net");
}

/// `resolve_env` falls back to the default when the env var is SET to an empty
/// string — the empty-shell-var bug case. A naive `unwrap_or_else` returns `""`
/// here (it only catches the `Err` of an unset var); `resolve_env` treats empty
/// as "use default", so the host is never dropped from the CSP.
#[test]
fn resolve_env_uses_default_when_empty() {
    assert_eq!(
        resolve_env(Some(""), DEFAULT_CDN),
        "https://s3.origa.uwuwu.net"
    );
}

/// `resolve_env` passes a non-empty env value through unchanged.
#[test]
fn resolve_env_uses_explicit_value_when_set() {
    assert_eq!(
        resolve_env(Some("https://cdn.staging.example.com"), DEFAULT_CDN),
        "https://cdn.staging.example.com"
    );
}

/// Drift guard for `DEFAULT_LANDING`: the `None` case asserts the literal
/// canonical landing host. A revert to the stale `origa.app` breaks this.
#[test]
fn default_landing_drift_guard() {
    assert_eq!(
        resolve_env(None, DEFAULT_LANDING),
        "https://origa.uwuwu.net"
    );
}

/// Drift guard for `DEFAULT_TRAILBASE`.
#[test]
fn default_trailbase_drift_guard() {
    assert_eq!(
        resolve_env(None, DEFAULT_TRAILBASE),
        "https://app.origa.uwuwu.net"
    );
}

/// Drift guard for `DEFAULT_SENTRY_INGEST_HOST`: pins the production ingest
/// host. A change of Sentry project / region requires updating this constant
/// AND the committed `tauri.conf.json` (the byte-equality drift guard above
/// catches a mismatch). See ADR-036 §7.
#[test]
fn default_sentry_ingest_host_drift_guard() {
    assert_eq!(
        DEFAULT_SENTRY_INGEST_HOST,
        "o4511840951992320.ingest.us.sentry.io"
    );
}

/// `extract_sentry_ingest_host` parses a standard Sentry SaaS DSN.
#[test]
fn extract_sentry_ingest_host_parses_us_region_dsn() {
    let dsn = "https://aef7d1190ac77f89ae112b6a97e93d01@o4511840951992320.ingest.us.sentry.io/4511840959201280";
    assert_eq!(
        extract_sentry_ingest_host(dsn),
        Some("o4511840951992320.ingest.us.sentry.io")
    );
}

/// `extract_sentry_ingest_host` parses an EU region DSN (no region segment).
#[test]
fn extract_sentry_ingest_host_parses_eu_region_dsn() {
    let dsn = "https://abc123@o42.ingest.sentry.io/99";
    assert_eq!(
        extract_sentry_ingest_host(dsn),
        Some("o42.ingest.sentry.io")
    );
}

/// `extract_sentry_ingest_host` rejects malformed DSNs (missing `@`).
#[test]
fn extract_sentry_ingest_host_rejects_missing_at() {
    assert_eq!(extract_sentry_ingest_host("https://sentry.io/1"), None);
}

/// `extract_sentry_ingest_host` rejects malformed DSNs (missing `/`).
#[test]
fn extract_sentry_ingest_host_rejects_missing_slash() {
    assert_eq!(extract_sentry_ingest_host("https://key@host"), None);
}

/// `extract_sentry_ingest_host` rejects an empty DSN.
#[test]
fn extract_sentry_ingest_host_rejects_empty() {
    assert_eq!(extract_sentry_ingest_host(""), None);
}
