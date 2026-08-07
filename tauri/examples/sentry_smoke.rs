//! Standalone Sentry smoke test — verifies that the `sentry` crate can
//! initialise, capture a message, and capture a panic with this project's
//! feature set + the user-provided DSN. Run with:
//!
//! ```powershell
//! $env:SENTRY_DSN = "https://<key>@<host>/<project>"
//! $env:ORIGA_CDN_BASE_URL = "https://origa-cdn.yurvon.workers.dev"  # required by build.rs
//! cargo run --example sentry_smoke --release
//! ```
//!
//! After the panic, check the Sentry UI — both the `capture_message` Info
//! event and the `panic` Fatal event should appear tagged with
//! `layer:tauri`, `smoke_test:true`.
//!
//! This is a manual debug tool, not a regression test. Delete if it rots.

use std::env;
use std::time::Duration;

fn main() {
    // Install the ring crypto provider — exactly what `tauri/src/lib.rs::run()`
    // does as its first statement. Without this, reqwest's rustls (used by
    // the sentry transport with `rustls-no-provider`) panics with
    // "No provider set". See ADR-036 §2.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let dsn = env::var("SENTRY_DSN").expect("SENTRY_DSN must be set");
    let release = env::var("ORIGA_VERSION").unwrap_or_else(|_| "smoke-test".into());
    let environment = env::var("SENTRY_ENVIRONMENT").unwrap_or_else(|_| "smoke".into());

    println!(
        "[smoke] initialising sentry with DSN ending in {}",
        dsn.rsplit_once('@').map(|(_, h)| h).unwrap_or("???")
    );

    let _guard = sentry::init(
        sentry::ClientOptions::new()
            .dsn(&dsn)
            .release(release)
            .environment(environment)
            .send_default_pii(false)
            .debug(true)
            .in_app_include(["origa", "origa_ui", "origa_app"]),
    );

    sentry::configure_scope(|scope| {
        scope.set_tag("layer", "tauri");
        scope.set_tag("smoke_test", "true");
    });

    println!("[smoke] sending capture_message (Info)...");
    sentry::capture_message(
        "smoke test: capture_message works — if you see this in Sentry UI, transport is OK",
        sentry::Level::Info,
    );

    println!("[smoke] waiting 3s for async flush...");
    std::thread::sleep(Duration::from_secs(3));

    println!("[smoke] NOW panicking — sentry-panic integration should capture this...");
    println!("[smoke] (panic=abort in release profile; panic hook fires before abort)");

    panic!("sentry smoke test panic — Everything is on fire!");
}
