//! Tauri build script with CSP parameterization.
//!
//! Reads env vars (`ORIGA_CDN_BASE_URL`, `TRAILBASE_URL`, `ORIGA_LANDING_BASE_URL`)
//! with fallback to production defaults, builds the CSP string, and injects it
//! into Tauri's config via the native `TAURI_CONFIG` env var (RFC 7396 JSON
//! Merge Patch). `TAURI_CONFIG` is an internal codegen detail; the closest
//! public-facing docs are the [`--config` flag](https://v2.tauri.app/develop/configuration-files/),
//! which uses the same RFC 7396 merge mechanism. See ADR-009 for full details
//! and verification.
//!
//! `tauri/capabilities/default.json` is intentionally NOT regenerated here:
//! build scripts must not mutate committed source files (Cargo contract). Its
//! opener allow-list is validated byte-for-byte against a template by
//! `tauri/tests/build_config.rs` instead.
//!
//! See ADR-009 (`docs/decisions/ADR-009-tauri-config-parameterization.md`) for
//! the full rationale, including why `unsafe { env::set_var }` is a sanctioned
//! exception to the project-wide "no unsafe" rule.

#[path = "build_config.rs"]
mod build_config;

use std::env;

use serde_json::json;

fn main() {
    // Declare custom cfg so rustc's check-cfg pass (Rust 2024+) recognizes
    // it as expected and doesn't emit "unexpected cfg condition name" errors
    // when clippy runs with `-D warnings`. The cfg is set below when the
    // `ORIGA_APP_STORE` env var is present.
    println!("cargo::rustc-check-cfg=cfg(app_store)");

    let cdn = build_config::resolve_env(
        env::var("ORIGA_CDN_BASE_URL").ok().as_deref(),
        build_config::DEFAULT_CDN,
    );
    let trailbase = build_config::resolve_env(
        env::var("TRAILBASE_URL").ok().as_deref(),
        build_config::DEFAULT_TRAILBASE,
    );
    let landing = build_config::resolve_env(
        env::var("ORIGA_LANDING_BASE_URL").ok().as_deref(),
        build_config::DEFAULT_LANDING,
    );

    inject_csp_via_tauri_config(&cdn, &landing, &trailbase);

    tauri_build::build();
}

/// Build a `TAURI_CONFIG` JSON Merge Patch (RFC 7396) overriding only the
/// `app.security.csp` field and expose it to:
///   1. `tauri_build::build()` — in-process (via `set_var`)
///   2. `tauri::generate_context!()` macro in `origa-app` — out-of-process
///      rustc compilation (via `cargo:rustc-env`)
///
/// Both paths are required because `tauri-codegen` may be invoked from either
/// context depending on which crate is being built.
///
/// If `TAURI_CONFIG` is already set (e.g., by `cargo tauri build/dev --config
/// <merge>`, which sets it via `set_var()` internally in
/// `tauri-cli/src/helpers/config.rs::load_config`), the CSP patch is MERGED
/// INTO the existing value via a local RFC 7396 merge (`apply_merge_patch`,
/// since serde_json does NOT expose a public `merge` API — verified against
/// serde_json 1.0.150) instead of replacing it. This preserves flavor/beta/
/// staging overrides (productName, identifier, bundle, plugins, devUrl, etc.).
/// The CSP wins because it is applied last.
fn inject_csp_via_tauri_config(cdn: &str, landing: &str, trailbase: &str) {
    let csp = build_config::build_csp(cdn, landing, trailbase);

    // Only `app.security.csp` is overridden — all other fields in
    // `tauri.conf.json` (productName, windows, plugins, bundle, etc.) remain
    // untouched.
    let csp_patch = json!({
        "app": {
            "security": {
                "csp": csp
            }
        }
    });

    // Preserve any existing `TAURI_CONFIG` set by `cargo tauri build/dev --config
    // <merge>` — the standard Tauri CLI mechanism for flavor/beta/staging
    // configs. Tauri CLI sets `TAURI_CONFIG` via `set_var()` internally (see
    // `tauri-cli/src/helpers/config.rs::load_config`), so we must MERGE our CSP
    // patch INTO the existing value, not replace it — otherwise flavor/beta
    // overrides passed via `--config` (productName, identifier, bundle, plugins,
    // devUrl, etc.) would be silently dropped. Both inputs are RFC 7396 JSON
    // Merge Patches, so `apply_merge_patch` (a local RFC 7396 implementation —
    // serde_json has no public `merge` API) composes them correctly; the CSP
    // wins because it is applied last.
    let final_config = match env::var("TAURI_CONFIG") {
        Ok(existing) => {
            let mut existing_value: serde_json::Value = serde_json::from_str(&existing)
                .expect("TAURI_CONFIG env var must be valid JSON (RFC 7396 merge patch)");
            build_config::apply_merge_patch(&mut existing_value, csp_patch);
            existing_value
        },
        Err(_) => csp_patch,
    };
    let final_config_str = final_config.to_string();

    // App Store builds: disable updater artifact generation entirely.
    // Triggered by the `ORIGA_APP_STORE` env var (NOT a cargo feature —
    // tauri-cli does not propagate `--features`/`--no-default-features`
    // to cargo). The env var is also used by build.rs to emit the
    // `app_store` rustc cfg flag, which `lib.rs` uses to gate out
    // `tauri-plugin-updater` registration (Mac App Store 2.4.5(vii)).
    //
    // Two patches applied to TAURI_CONFIG:
    // 1. `bundle.createUpdaterArtifacts: false` — stop emitting .sig files
    // 2. `plugins.updater: null` (RFC 7396 deletion) — tauri-bundler refuses
    //    to build macOS bundle if `plugins.updater.pubkey` is present but
    //    `TAURI_SIGNING_PRIVATE_KEY` env var is missing. Removing the
    //    section entirely disables updater integration in the bundler.
    //
    // RFC 7396 merge semantics: last write wins.
    if env::var("ORIGA_APP_STORE").is_ok() {
        // Emit cfg for lib.rs to consume.
        println!("cargo:rustc-cfg=app_store");

        let updater_patch = json!({
            "bundle": {
                "createUpdaterArtifacts": false
            },
            "plugins": {
                "updater": null
            }
        });
        let mut cfg: serde_json::Value = serde_json::from_str(&final_config_str)
            .expect("TAURI_CONFIG must be valid JSON for app-store patch");
        build_config::apply_merge_patch(&mut cfg, updater_patch);
        let patched = cfg.to_string();

        // SAFETY: same single-threaded build-script contract as below.
        unsafe {
            env::set_var("TAURI_CONFIG", &patched);
        }
        println!("cargo:rustc-env=TAURI_CONFIG={patched}");
    } else {
        // Desktop distribution path: emit final_config_str unchanged
        // (CSP-merged, createUpdaterArtifacts still true per tauri.conf.json
        // — needed for updater manifest on GitHub Releases).
        // SAFETY: build scripts are single-threaded by Cargo's contract —
        // exactly one `main()` runs per build script invocation, with no
        // spawned threads. `tauri_build::build()` is a synchronous API
        // (no async, no `std::thread::spawn`) that reads `TAURI_CONFIG`
        // via `env::var()` on the same thread as this `main()`. `set_var`
        // is marked `unsafe` since Rust edition 2024 due to potential data
        // races in multi-threaded contexts, which do not apply here.
        unsafe {
            env::set_var("TAURI_CONFIG", &final_config_str);
        }
        println!("cargo:rustc-env=TAURI_CONFIG={final_config_str}");
    }

    println!("cargo:rerun-if-env-changed=ORIGA_APP_STORE");
    println!("cargo:rerun-if-env-changed=ORIGA_CDN_BASE_URL");
    println!("cargo:rerun-if-env-changed=TRAILBASE_URL");
    println!("cargo:rerun-if-env-changed=ORIGA_LANDING_BASE_URL");
    println!("cargo:rerun-if-changed=build_config.rs");
    println!("cargo:rerun-if-changed=../build_defaults.rs");
    // `TAURI_CONFIG` may be set externally by `cargo tauri build/dev --config
    // <merge>` (Tauri CLI); changes to it must re-run this build script so the
    // CSP patch is re-merged into the latest external value.
    println!("cargo:rerun-if-env-changed=TAURI_CONFIG");
}
