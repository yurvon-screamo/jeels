# ADR-036: Sentry error monitoring integration

## Status

Accepted

## Date

2026-08-01

## Context

Origa is shipped to end users on five targets (Windows, Linux, macOS, iOS,
Android) plus a WASM frontend running inside the Tauri WebView. Production
crashes — native Rust panics in the tauri layer, JS/WASM exceptions in the UI
layer — were invisible: no stack traces, no reproduction steps, no aggregate
view of "which crash affects the most users". The repo has no error-reporting
infrastructure at all.

This ADR records the design decisions for the initial Sentry (SaaS) rollout:
which Sentry SDKs are wired into which layer, how the DSN is delivered to each
layer, which feature flags are enabled, how the CSP is updated, and which
plumbing is intentionally deferred to a follow-up PR.

The rollout targets a **single Sentry project** with a `layer` scope tag
(`tauri` vs `ui`) distinguishing the two layers. This avoids the operational
overhead of two projects (separate quota, separate dashboards, separate
release-health views) while preserving the ability to filter by layer.

## Decision

### 1. Two Sentry SDKs, one project

| Layer | SDK | Mechanism |
| --- | --- | --- |
| `tauri/` (native Rust) | [`sentry` Rust crate](https://crates.io/crates/sentry) 0.49 | `sentry::init` in `tauri/src/lib.rs::run()`; `sentry-panic` integration auto-installs the panic hook. |
| `origa_ui/` (WASM / JS) | [Sentry JavaScript loader](https://docs.sentry.io/platforms/javascript/install/loader/) | Loader script (`js.sentry-cdn.com`) injected dynamically from WASM via `web_sys` at `init_tracing()`; `window.sentryOnLoad` configures the SDK. |

The Rust `sentry` crate does not compile to WASM, and there is no
pure-Rust Sentry client that targets WASM, so the WASM layer uses the official
JS SDK.

### 2. Feature set for the Rust crate

```toml
sentry = { version = "0.49", default-features = false, features = [
    "rustls-no-provider", "panic", "backtrace", "contexts",
    "release-health", "reqwest", "debug-images",
] }
```

`default-features = false` is required because the default `transport` feature
pulls in `native-tls` → `openssl-sys`, which (a) violates the rustls-only
policy of the workspace and (b) breaks the Android cross-compile pipeline
(no OpenSSL sysroot in the NDK setup). The `reqwest` feature is sufficient:
`apply_defaults` in `sentry::init` registers `DefaultTransportFactory`
unconditionally, and that factory selects the reqwest backend when the
`reqwest` feature is on. `transport` is a convenience alias for `reqwest +
native-tls` and is therefore not used.

`rustls-no-provider` (not `rustls`) reuses the ring crypto provider that
`tauri/src/lib.rs::run()` installs as its first statement. Using `rustls`
would bring a second crypto provider (`aws-lc-rs`) into the dependency tree
and risk a runtime "no process-level CryptoProvider" panic if the order ever
changes. `rustls-no-provider` makes the load-bearing dependency on
`tauri/Cargo.toml`'s `rustls = { features = ["ring"] }` explicit: removing
that line silently breaks Sentry transport.

`metrics`, `logs`, `debug-images` (default-on but `metrics`/`logs` disabled
here) are out of scope for the initial rollout: this PR captures errors only.
`debug-images` IS enabled: it attaches the loaded-module list to native
events, which Sentry uses for crash grouping/correlation even before symbol
upload is configured (see §6).

### 3. DSN delivery

A single `SENTRY_DSN` env var is exposed by CI (`vars.SENTRY_DSN`, NOT a
secret — the public DSN key is shipped in the client binary by design). Each
build script reads it and re-emits under a crate-scoped name:

| Build script | Emits | Read by |
| --- | --- | --- |
| `tauri/build.rs` | `SENTRY_DSN_TAURI`, `SENTRY_ENVIRONMENT`, `SENTRY_RELEASE` | `tauri/src/lib.rs::init_sentry` via `env!()` |
| `origa_ui/build.rs` | `SENTRY_DSN_UI`, `SENTRY_ENVIRONMENT_UI`, `SENTRY_RELEASE_UI` | `origa_ui/src/sentry.rs` via `env!()` |

`SENTRY_RELEASE` is derived from `ORIGA_VERSION` in both build scripts (not a
separate CI var), so native and WASM layers cannot drift on release name.

Env-var reads use `env::var()` + `cargo:rustc-env` + `cargo:rerun-if-env-changed`
(NOT `option_env!`): `option_env!` captures at build-script compile time and
its value is not invalidated by `rerun-if-env-changed`, so cached build-script
binaries (CI uses `Swatinem/rust-cache`) would silently retain stale values.
This is the same pattern ADR-024 established for `ORIGA_CDN_BASE_URL` /
`TRAILBASE_URL`. Empty/unset DSN = Sentry disabled at runtime (`init_sentry`
returns `None`; `sentry::init` does likewise in WASM).

### 4. WASM loader injection (not hardcoded `index.html`)

`origa_ui/index.html` is **not modified**: build scripts must not mutate
committed source files (Cargo contract, ADR-009). Instead, the loader script
is injected dynamically from WASM at `init_tracing()`:

1. `window.sentryOnLoad = function() { Sentry.init({dsn, release, environment, ...}); }`
   is installed via `js_sys::eval` BEFORE the loader script is appended, so
   the loader invokes it even when it loads synchronously from HTTP cache.
2. `<script src="https://js.sentry-cdn.com/{public_key}.min.js" crossorigin="anonymous">`
   is appended to `<head>` via `web_sys::Document::create_element`.

The public key is extracted from the DSN by `extract_public_key` (string
split, no Sentry crate dependency in WASM).

**Trade-off:** JS errors that occur before WASM instantiates are not
captured. These are rare (they would mostly be "WASM failed to instantiate"
errors, which cannot be reported by client-side code anyway) and the
alternative — hardcoding the DSN in `index.html` — was rejected because it
loses dev/Dependabot gating and reintroduces the Cargo-contract question of
how to parameterise a committed file.

### 5. WASM panic bridge

`origa_ui/src/lib.rs::init_tracing` wraps `console_error_panic_hook`:

```rust
console_error_panic_hook::set_once();   // install console hook
let console_hook = std::panic::take_hook(); // take it
std::panic::set_hook(Box::new(move |info| {
    sentry::capture_exception(&info.to_string()); // forward to Sentry
    console_hook(info);                            // then console fallback
}));
sentry::init(); // inject loader
```

`sentry::capture_exception` is **defensive** — it is called on every WASM
panic, including the disabled path (no loader, `window.Sentry` is
`undefined`). Every JS call is wrapped; any failure is silently dropped so
the console fallback hook still runs.

### 6. `default_integrations = true` (no custom panic hook)

The native Rust layer uses the default integrations (backtrace, contexts,
debug-images, panic, release-health). The `sentry-panic` integration
installs a panic hook that calls `client.flush(None)` synchronously inside
the hook before returning.

**Trade-off accepted:** `flush(None)` is bounded by the OS TCP connect/read
timeout (~20-75s), not infinite. With `panic = "abort"` in the release
profile, abort fires only after the panic hook returns. This means a panic
on a slow/offline network may visually freeze the app for tens of seconds
before exit. Alternative considered: `default_integrations = false` + manual
registration of the four non-panic integrations + a custom panic hook with
bounded `flush(Some(Duration::from_secs(2)))`. **Rejected** for the initial
rollout because (a) it adds complexity (manual integration registration with
version-specific type paths that may regress on a `sentry` crate bump), (b)
the panic situation already means the app is dying — the marginal UX
difference between "freezes 30s then exits" and "exits immediately" is small
for the user, who must restart either way, (c) if it proves painful in
production, a follow-up PR can introduce the custom hook as a targeted
mitigation with a regression test.

### 7. CSP update

`tauri/build_config.rs::build_csp` and `tauri/tauri.conf.json` add:

- `script-src`: `https://js.sentry-cdn.com` (loader script host)
- `connect-src`: `https://<ingest_host>` (envelope submission, pinned)

The ingest host is **extracted from the DSN at build time** by
`build_config::extract_sentry_ingest_host` and parameterised into
`build_csp`. The default (used when `SENTRY_DSN` is unset, e.g. local dev)
is `DEFAULT_SENTRY_INGEST_HOST`, pinned to the production project's exact
host.

**Why pin and not `*.sentry.io` wildcard?** `sentry.io` is a multi-tenant
SaaS domain: anyone can create a free Sentry account and receive an
`<orgid>.ingest.sentry.io` subdomain. A wildcard `connect-src` would allow
the WebView to POST to **any** Sentry project, including attacker-controlled
ones — a data exfiltration vector post-XSS. CSP3 §8.6 explicitly warns about
this class. The cost of pinning (a CSP edit on the rare Sentry project /
region migration) is negligible vs a permanent exfil surface.

`build_csp_with_production_defaults_matches_committed_tauri_conf` and
`build_csp_substitutes_staging_hosts` in `tauri/tests/build_config.rs`
enforce byte-equality between the template and the committed `tauri.conf.json`
and assert the staging build carries the staging ingest host (not a
wildcard, not the production host). `extract_sentry_ingest_host` is covered
by five unit tests (US DSN, EU DSN, malformed variants, empty input).

### 8. CI/CD wiring

A single `vars.SENTRY_DSN` is exposed to all build jobs. `SENTRY_ENVIRONMENT`
is derived from `version_type`:

| `version_type` | `SENTRY_ENVIRONMENT` |
| --- | --- |
| `stable` | `production` |
| `prerelease` | `staging` |
| (other: master, dev, branch) | `development` |

The mapping is inlined as a GitHub Actions expression in each job's `env:`
block, applied in: `tauri.yml::build-frontend`, `_build-tauri.yml::build-{windows,linux,android}`,
`_build-tauri-apple.yml::build-{ios,macos}`.

## Verification coverage

- **Pure helpers** (`extract_public_key`) are covered by host-side `#[test]`
  unit tests in `origa_ui/src/sentry.rs`.
- **CSP wiring** is covered by drift-guard tests in
  `tauri/tests/build_config.rs` (byte-equality + staging Sentry hosts).
- **Build-script wiring** (env → `cargo:rustc-env`) is NOT covered by
  `cargo test` — build scripts are not unit-tested (architectural limit,
  same gap as for all other build-script env vars in the project). Drift
  guards on the build_config constants protect the values.
- **`init_sentry` / `init_with` runtime behaviour** (DSN parse, scope tag,
  loader injection, panic hook) is **manual-check only** — requires a live
  Sentry DSN and a running app. The Slice 4 root-store smoke-test (below)
  covers the "does an event actually reach Sentry UI" path.

## Slice 4 verification

- `cargo tree -e features -i tokio` — confirms the sentry transport's tokio
  runtime doesn't unexpectedly enable `rt-multi-thread` in the workspace.
- `cargo tree -i reqwest` — confirms two reqwest versions are present
  (`0.12.x` workspace + `0.13.2` sentry) and quantifies the binary-size
  overhead.
- Binary size delta (before/after) measured on a release build for each
  mobile target where size matters for store review.
- **Root-store smoke-test**: build with a real DSN, run the binary, trigger a
  test panic, verify the event appears in the Sentry UI within ~30s. This is
  the only way to confirm that the `rustls-no-provider` feature, the ring
  provider installed by tauri, and reqwest's root-store chain combine into a
  working TLS handshake to `*.ingest.sentry.io`. UNVERIFIED offline — must
  be done manually before merging the release tag that ships this.

## Consequences

### Positive

- Native Rust panics (tauri layer) and JS/WASM exceptions (UI layer) are
  reported to Sentry with stack traces, environment, release, and layer tag.
- Single Sentry project with `layer` tag keeps the operational surface small.
- CI gating via `vars.SENTRY_DSN` lets dev/Dependabot builds skip Sentry
  automatically (empty DSN = disabled), no per-actor conditional needed.
- rustls-only policy preserved — no native-tls/openssl-sys in the dependency
  tree, Android cross-compile intact.
- Drift-guard tests catch any future CSP / DSN-wiring regression.

### Negative

- **Native stack traces will not be symbolicated** until a follow-up PR
  uploads debug symbols (PDB on Windows, dSYM on macOS, ELF debug-info on
  Linux, .sym on Android). Without symbols, native events arrive with raw
  addresses — useful for grouping/correlation (via `debug-images` module
  list) but useless for human triage. WASM events similarly need source-map
  upload (trunk sourcemap generation under `opt-level=z` is UNVERIFIED and
  may produce incomplete maps). The follow-up is tracked as the obvious
  next slice; this PR ships the plumbing (init/capture/transport/CSP/env)
  so the symbol upload is a pure CI addition.
- **`flush(None)` UX risk** on offline panic (see §6). Acceptable for the
  initial rollout; revisit if user reports come in.
- **`reqwest 0.13` duplication**: sentry 0.49 depends on `reqwest ^0.13`,
  the workspace pins `reqwest 0.12`. Both versions end up in the binary.
  This is determinist bloat (~150KB per arch), not a runtime risk. A
  workspace-wide reqwest bump is out of scope.
- **`sentry 0.49` future-incompat warning**: the `sentry` 0.49 crate uses
  patterns flagged by Rust's future-incompat lints. Not a current build
  failure; will need a `sentry` bump when a future Rust version rejects it.
- **Loader injection race**: WASM errors that occur before
  `init_tracing()` runs are not captured. This is accepted (see §4).

## NOTICED BUT NOT TOUCHING

- **Source maps upload** (WASM) — follow-up, requires trunk sourcemap spike
  under `opt-level=z` + `sentry-cli sourcemaps upload` CI step.
- **Native debug symbol upload** — follow-up, requires
  `sentry-cli debug-files upload` per platform + xwin PDB cross-compile spike
  for Windows.
- **Performance tracing / Session Replay** — follow-up (payload overhead,
  opt-in).
- **User Feedback widget** — follow-up.
- **Landing SSR Sentry** — out of scope, separate deployment, not mentioned
  in the original task.
- **Sentry crate bump past 0.49** — out of scope; revisit when future-incompat
  becomes a hard error.

## References

- Sentry Rust SDK: <https://docs.sentry.io/platforms/rust/>
- Sentry JS loader script: <https://docs.sentry.io/platforms/javascript/install/loader/>
- `sentry-panic` source (`flush(None)` contract):
  <https://docs.rs/sentry-panic/latest/src/sentry_panic/lib.rs.html>
- ADR-009: Tauri config parameterization (Cargo contract for build scripts)
- ADR-024: Build-script env var reads must handle the empty-string case
  (`resolve_env` principle; `env::var` + `cargo:rustc-env` +
  `rerun-if-env-changed` pattern)
- ADR-028: Self-hosted fonts on the CDN (CSP `font-src` carries the CDN host)
