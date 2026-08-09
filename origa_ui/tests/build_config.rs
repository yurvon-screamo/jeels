//! Drift-detection and Prove-It tests for `origa_ui/build_config.rs`.
//!
//! Strategy: `#[path]`-include the pure `build_config` module and exercise
//! `resolve_trailbase` / `resolve_cdn` across the three input shapes (unset /
//! empty / set). The "unset" cases also serve as drift guards for
//! `DEFAULT_TRAILBASE` / `DEFAULT_CDN`: the fallback branch returns exactly
//! that constant, so any change to its value breaks the test.

#[path = "../build_config.rs"]
mod build_config;

use build_config::{resolve_cdn, resolve_trailbase};

#[test]
fn trailbase_uses_production_default_when_unset() {
    assert_eq!(resolve_trailbase(None), "https://app.origa.uwuwu.net");
}

#[test]
fn trailbase_uses_production_default_when_empty() {
    assert_eq!(resolve_trailbase(Some("")), "https://app.origa.uwuwu.net");
}

#[test]
fn trailbase_uses_explicit_value_when_set() {
    assert_eq!(
        resolve_trailbase(Some("https://staging.example.com")),
        "https://staging.example.com"
    );
}

#[test]
fn cdn_uses_production_default_when_unset() {
    assert_eq!(resolve_cdn(None), "https://s3.origa.uwuwu.net");
}

#[test]
fn cdn_uses_production_default_when_empty() {
    assert_eq!(resolve_cdn(Some("")), "https://s3.origa.uwuwu.net");
}

#[test]
fn cdn_uses_explicit_value_when_set() {
    assert_eq!(
        resolve_cdn(Some("https://cdn.staging.example.com")),
        "https://cdn.staging.example.com"
    );
}
