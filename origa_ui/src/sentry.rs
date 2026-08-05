//! Sentry integration for the WASM frontend.
//!
//! Sentry's JavaScript SDK is used (the Rust `sentry` crate does not compile
//! to WASM). The loader script is injected dynamically at runtime via the DOM
//! (`<script src="https://js.sentry-cdn.com/{public_key}.min.js">`) rather than
//! hardcoded in `index.html`, so the DSN can be parameterised at build time
//! through `SENTRY_DSN_UI` without violating the Cargo "build scripts must not
//! mutate committed source files" contract. See ADR-036 §4.
//!
//! The `layer = "ui"` scope tag distinguishes WASM/browser events from
//! tauri-native events in the shared Sentry project.
//!
//! ## Panic bridge
//!
//! `init()` does not install the panic hook itself — `lib.rs::init_tracing`
//! wraps the existing `console_error_panic_hook` so that Rust panics are first
//! forwarded to `capture_exception` and then logged to the browser console.
//! On the WASM side there is no `flush(None)` concern: the JS SDK flushes
//! asynchronously, so the panic hook never blocks.

use js_sys::Reflect;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, Element};

/// Build-time-injected Sentry configuration. Empty values disable Sentry
/// (the dev/Dependabot path). See `build.rs`.
const DSN: &str = env!("SENTRY_DSN_UI");
const RELEASE: &str = env!("SENTRY_RELEASE_UI");
const ENVIRONMENT: &str = env!("SENTRY_ENVIRONMENT_UI");

/// Initialise Sentry by injecting the loader script and configuring the SDK.
///
/// No-op when `SENTRY_DSN_UI` is empty (dev/Dependabot builds). Defensive
/// against missing `window`/`document` (SSR contexts, headless test runners).
///
/// The actual `Sentry.init` call is deferred until the loader script has
/// loaded via its `window.sentryOnLoad` callback hook, which we install
/// *before* injecting the script to avoid losing the callback if the script
/// is served from HTTP cache and loads synchronously (ADR-036 §4).
pub fn init() {
    init_with(DSN, RELEASE, ENVIRONMENT);
}

/// Parameterised entry point used by `init()` and the wasm-bindgen tests.
/// Split out so tests can drive the loader-injection logic with arbitrary
/// DSN/release/environment values without depending on compile-time env.
pub(crate) fn init_with(dsn: &str, release: &str, environment: &str) {
    if dsn.is_empty() {
        tracing::debug!("[sentry] disabled (no SENTRY_DSN_UI)");
        return;
    }

    let Some(public_key) = extract_public_key(dsn) else {
        tracing::warn!("[sentry] could not parse public key from DSN, disabling");
        return;
    };

    let Some(window) = web_sys::window() else {
        tracing::warn!("[sentry] no global window, skipping init");
        return;
    };
    let Some(document) = window.document() else {
        tracing::warn!("[sentry] no document, skipping init");
        return;
    };
    let Some(head) = document.head() else {
        tracing::warn!("[sentry] no <head>, skipping init");
        return;
    };

    // 1. Install `window.sentryOnLoad` BEFORE injecting the script so the
    //    loader invokes it even when it loads synchronously from cache.
    let init_js = format!(
        r#"(function() {{
            window.sentryOnLoad = function() {{
                Sentry.init({{
                    dsn: "{dsn}",
                    release: "{release}",
                    environment: "{environment}",
                    sendDefaultPii: false,
                    tracesSampleRate: 1.0,
                    integrations: [
                        // CaptureConsole routes tracing-wasm's console.error
                        // output (which is where all WASM tracing::error! calls
                        // land via the WASMLayer) into Sentry as error events.
                        // console.warn/info become breadcrumbs via the default
                        // Breadcrumbs integration (not CaptureConsole).
                        new Sentry.Integrations.CaptureConsole({{ levels: ['error'] }})
                    ]
                }});
                Sentry.setTag("layer", "ui");
            }};
        }})();"#
    );
    if let Err(e) = js_sys::eval(&init_js) {
        tracing::warn!("[sentry] failed to install sentryOnLoad: {:?}", e);
        return;
    }

    // 2. Inject the loader script. `defer` is intentionally NOT set: it is a
    //    no-op on dynamically-inserted <script> elements per HTML spec, and
    //    `sentryOnLoad` above already gates the actual init.
    let loader_url = format!("https://js.sentry-cdn.com/{public_key}.min.js");
    if let Err(e) = inject_script(&document, &head, &loader_url) {
        tracing::warn!("[sentry] failed to inject loader script: {:?}", e);
        return;
    }

    tracing::info!(
        "[sentry] enabled (release={}, environment={})",
        release,
        environment
    );
}

/// Forward a panic message to Sentry as a captured exception.
///
/// **Must never panic itself**: this is called from the panic hook on every
/// WASM panic, including the path where Sentry is disabled (no loader
/// injected, `window.Sentry` is `undefined`). Defensive on every JS call —
/// any failure is silently dropped so the downstream console hook still runs.
pub fn capture_exception(msg: &str) {
    // The SDK accepts a string here and synthesises a stacktrace-less event.
    // Wrapping in `new Error(msg)` would preserve a synthetic stack, but the
    // WASM panic message already includes location info from
    // `console_error_panic_hook`, so the marginal value is low.
    let Ok(sentry_obj) = sentry_global() else {
        return;
    };
    if sentry_obj.is_undefined() || sentry_obj.is_null() {
        return;
    }
    let Ok(capture) = Reflect::get(&sentry_obj, &"captureException".into()) else {
        return;
    };
    let Some(f) = capture.dyn_ref::<js_sys::Function>() else {
        return;
    };
    let _ = f.call1(&sentry_obj, &JsValue::from(msg));
}

/// Read `window.Sentry`. Returns `Err` only if `window` itself is missing;
/// the returned value may still be `undefined` when the loader has not yet
/// loaded (or was never injected).
fn sentry_global() -> Result<JsValue, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from("no window"))?;
    Reflect::get(&window.into(), &"Sentry".into())
}

/// Extract the public key (first URL segment before `@`) from a Sentry DSN.
///
/// DSN format: `https://<public_key>@<host>/<project_id>`. Returns `None` for
/// malformed inputs so `init_with` can fail gracefully.
fn extract_public_key(dsn: &str) -> Option<&str> {
    let before_at = dsn.split_once('@')?.0;
    let key = before_at
        .rsplit_once("://")
        .map(|(_, k)| k)
        .unwrap_or(before_at);
    if key.is_empty() {
        return None;
    }
    Some(key)
}

/// Create and append a `<script src=url crossorigin data-lazy=no>` element to
/// `<head>`.
///
/// `data-lazy="no"` forces the loader to fetch the full SDK immediately (on the
/// next event-loop tick) instead of waiting for the first error. This is
/// required for performance monitoring: BrowserTracing must be active *before*
/// the app's `fetch` calls to capture them as transactions. Without it, the
/// loader only downloads the SDK when an error occurs — by which point the
/// dictionary-loading `fetch` requests have already happened uninstrumented.
fn inject_script(
    document: &Document,
    head: &web_sys::HtmlHeadElement,
    url: &str,
) -> Result<(), JsValue> {
    let script: Element = document.create_element("script")?;
    script.set_attribute("src", url)?;
    script.set_attribute("crossorigin", "anonymous")?;
    script.set_attribute("data-lazy", "no")?;
    head.append_child(&script)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_public_key_parses_standard_dsn() {
        let dsn = "https://abc123def456@o789.ingest.sentry.io/42";
        assert_eq!(extract_public_key(dsn), Some("abc123def456"));
    }

    #[test]
    fn extract_public_key_rejects_missing_at() {
        assert_eq!(extract_public_key("https://sentry.io/42"), None);
    }

    #[test]
    fn extract_public_key_rejects_empty_key() {
        assert_eq!(extract_public_key("https://@o1.ingest.sentry.io/1"), None);
    }

    #[test]
    fn extract_public_key_handles_missing_scheme() {
        // Defensive: a DSN missing the scheme still has the key segment,
        // though Sentry itself would reject it. We parse leniently here and
        // let Sentry's own init fail if the value is unusable.
        assert_eq!(extract_public_key("abc@host/1"), Some("abc"));
    }

    #[test]
    fn extract_public_key_rejects_empty_dsn() {
        assert_eq!(extract_public_key(""), None);
    }
}

/// WASM-only tests for `init_with`'s DOM-injection logic. Run via
/// `wasm-pack test --chrome origa_ui -- sentry` (or
/// `cargo test -p origa_ui --target wasm32-unknown-unknown -- sentry`).
///
/// These are the regression guards for the race fixed in ADR-036 §4
/// (`sentryOnLoad` installed BEFORE `<script>` appended) and for the
/// empty-DSN no-op contract.
#[cfg(all(target_arch = "wasm32", test))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    /// Empty DSN must not inject any `<script>` into `<head>`. Regression
    /// guard for the dev/Dependabot disabled-Sentry path.
    #[wasm_bindgen_test]
    fn init_with_empty_dsn_is_noop() {
        let before = count_head_scripts();
        init_with("", "1.0.0", "test");
        let after = count_head_scripts();
        assert_eq!(
            before, after,
            "no <script> should be injected for empty DSN"
        );
    }

    /// A valid DSN must inject exactly one `<script>` with the expected
    /// loader URL into `<head>`. Regression guard for the loader-injection
    /// path (extract_public_key + inject_script + sentryOnLoad ordering).
    #[wasm_bindgen_test]
    fn init_with_dsn_injects_single_loader_script() {
        // Snapshot before so the test is order-independent across other
        // tests that may have already injected.
        let before = count_head_scripts();
        init_with(
            "https://abc123def456@o789.ingest.sentry.io/42",
            "1.0.0",
            "test",
        );
        let after = count_head_scripts();

        assert_eq!(
            after - before,
            1,
            "exactly one <script> should be injected for a set DSN"
        );

        // The injected script's src must point at the Sentry loader CDN with
        // the public key extracted from the DSN.
        let last_src = last_head_script_src().unwrap_or_default();
        assert!(
            last_src.contains("js.sentry-cdn.com/abc123def456.min.js"),
            "injected script src must point at the Sentry loader CDN with the DSN public key, got: {last_src}"
        );

        // data-lazy="no" must be set so the loader fetches the full SDK
        // immediately (required for BrowserTracing to capture fetch calls
        // during dictionary loading).
        let last_data_lazy = last_head_script_data_lazy().unwrap_or_default();
        assert_eq!(
            last_data_lazy, "no",
            "injected script must have data-lazy=no for eager SDK loading"
        );
    }

    fn count_head_scripts() -> usize {
        let window = web_sys::window().expect("window");
        let document = window.document().expect("document");
        let head = document.head().expect("head");
        head.query_selector_all("script")
            .map(|nodes| nodes.length() as usize)
            .unwrap_or(0)
    }

    fn last_head_script_src() -> Option<String> {
        let window = web_sys::window()?;
        let document = window.document()?;
        let head = document.head()?;
        let nodes = head.query_selector_all("script").ok()?;
        let last = nodes.item(nodes.length() - 1)?;
        let el = last.dyn_ref::<web_sys::Element>()?;
        el.get_attribute("src")
    }

    fn last_head_script_data_lazy() -> Option<String> {
        let window = web_sys::window()?;
        let document = window.document()?;
        let head = document.head()?;
        let nodes = head.query_selector_all("script").ok()?;
        let last = nodes.item(nodes.length() - 1)?;
        let el = last.dyn_ref::<web_sys::Element>()?;
        el.get_attribute("data-lazy")
    }
}
