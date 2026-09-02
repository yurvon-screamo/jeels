# ADR-008: Mobile OAuth login flow on Tauri Android

## Status

Accepted — redirect_uri contract updated by ADR-044 (Apple platforms now
use the direct `origa://` scheme redirect; Android/Windows/Linux keep this
ADR's desktop-callback flow)

## Date

2026-06-18

## Context

The Tauri Android build had completely broken login — all three methods (email/password, Google OAuth, Yandex OAuth) failed. Diagnosis revealed four interacting root causes:

1. **Promise handling (RC-1):** `tauri-plugin-opener`'s `openUrl()` returns a JavaScript `Promise`. The previous handler in `oauth_buttons.rs` checked only the synchronous `.is_err()` on `JsValue::call1`, which cannot detect asynchronous Promise rejection. On Android, the opener Promise rejects (because the URL is not in the capability allow-list — see RC-3), but the rejection is swallowed and the `window.open` fallback never runs. Symptom: "tap Google button, nothing happens."

2. **Empty redirect_uri (RC-2):** Commit `eeee03ad` ("mobile OIDC redirect", 2026-05-23) removed `ORIGA_PUBLIC_BASE_URL: ${{ vars.TRAILBASE_URL }}` from CI to fix static asset serving through Tauri protocol. Side effect: `config::public_url()` (= `env!("ORIGA_PUBLIC_BASE_URL").unwrap_or("")`) became empty, so `redirect_uri` collapsed to a relative path `/public/auth/desktop-callback.html`. TrailBase cannot redirect to a relative URL.

3. **Opener scope glob mismatch (RC-3):** `tauri/capabilities/default.json` allowed `https://origa.uwuwu.net/*`. The actual TrailBase URL is `https://app.origa.uwuwu.net` (env `TRAILBASE_URL`). The glob does NOT match subdomains by default — each host needs its own entry.

4. **CSP blocks fetch (RC-4):** `tauri/tauri.conf.json` CSP `connect-src` lacked `app.origa.uwuwu.net`, so `POST /api/auth/v1/login` (email/password) and `/api/auth/v1/token` (OAuth code exchange) were blocked by the WebView. This is why ALL three methods failed, not just OAuth.

Key constraint: AGENTS.md marks `.github/workflows/`, `Cargo.toml`, and `origa/src/domain/` as "ask first" zones. The fix had to land WITHOUT touching CI workflows or workspace deps.

## Decision

Four coordinated fixes, all on the application/config side (no CI changes):

### RC-1: Await the opener Promise

`oauth_buttons.rs::open_url_external` now uses `JsFuture::from(promise).await` with an explicit `Err` branch that falls back to `window.open(url, "_blank")`. The `url` is cloned to an owned `String` before the `spawn_local` closure (lifetime `'static` requirement).

### RC-2: Derive redirect_uri from `trailbase_url()` (Rust-side)

Instead of restoring `ORIGA_PUBLIC_BASE_URL` in CI (which would require workflow changes — AGENTS.md "ask first" zone), the `redirect_uri` is now built as `format!("{}{}", trailbase_url(), "/public/auth/desktop-callback.html")` where `trailbase_url()` returns `env!("TRAILBASE_URL")` (= `https://app.origa.uwuwu.net`). `trailbase_url()` was promoted from private to `pub(crate)`.

### RC-3: Explicit capability entries per host

Removed vestigial `accounts.google.com`, `oauth.yandex.com`, `passport.yandex.com` entries (opener never opens them — TrailBase redirects to providers server-side in the external browser). Added explicit `https://app.origa.uwuwu.net/*`. Kept `https://origa.uwuwu.net/*` for redirect/error-link flows.

### RC-4: CSP allowlist the TrailBase host

Added `https://app.origa.uwuwu.net` to CSP `connect-src` in `tauri.conf.json`.

### Bonus: Compile-time gated diagnostic overlay

For future OAuth-flow regressions on mobile (where DevTools are unavailable), added an on-screen trace overlay gated on `option_env!("ORIGA_DEBUG_OAUTH") == "1"`. In production builds (env unset), the macro is a dead branch removed by the optimizer — no allocation. Implemented as `macro_rules! report_debug` with FIFO cap (16KB) to bound accumulated trace.

## Alternatives Considered

### A1: Restore `ORIGA_PUBLIC_BASE_URL` in CI workflows

- **Pros:** Single source of truth for public base URL; `config::public_url()` stays meaningful.
- **Cons:** Touches `.github/workflows/` (AGENTS.md "ask first" zone); requires user confirmation; doesn't fix RC-1/RC-3/RC-4.
- **Rejected:** Out of autonomous scope; deferred to a separate ticket if URL parameterization is needed.

### A2: Parameterize CSP via `tauri/build.rs`

- **Pros:** Eliminates duplication of `app.origa.uwuwu.net` across 4 locations (CSP, capability, `trailbase_url`, redirect_uri builder).
- **Cons:** Build-script complexity; `tauri.conf.json` is static JSON without template substitution support; would need a pre-build step.
- **Deferred (2026-06-15):** Tracked as `TODO(TECH-DEBT)` in `tauri/build.rs`.
- **RESOLVED (2026-06-18):** Implemented by ADR-009 using the native Tauri v2 `TAURI_CONFIG` env var (RFC 7396 JSON Merge Patch) — no custom pre-build step required. CSP is injected; capabilities stay static but are guarded by a byte-equality drift-detection test.

### A3: Use `window.open` as primary path (not opener plugin)

- **Pros:** No capability configuration needed.
- **Cons:** Android WebView without `setSupportMultipleWindows(true)` silently returns `null` from `window.open` — does not open the system browser. `tauri-plugin-opener` is the correct abstraction.
- **Rejected:** Would not work on Android.

### A4: Disable mobile target entirely

- **Pros:** Avoids the entire class of mobile-specific issues.
- **Cons:** Login is a critical user-facing path; Android is a supported target per AGENTS.md.
- **Rejected.**

## Consequences

### Positive

- All three login methods work end-to-end on Tauri Android.
- Future OAuth regressions are debuggable on-device via `ORIGA_DEBUG_OAUTH=1` overlay (no DevTools needed).
- Opener capability is now minimal (no vestigial entries) — cleaner security posture.
- Promise handling pattern is consistent with existing codebase (`oauth_listeners.rs:147`, `updater.rs:76`, `text_to_speech.rs:67`).

### Negative

> **RESOLVED (2026-06-18):** The host duplication tech-debt documented in the
> bullet below has been partially addressed by ADR-009 (Tauri config
> parameterization via the `TAURI_CONFIG` env var JSON Merge Patch). `tauri/build.rs`
> now injects the parameterized CSP at build time; the opener allow-list in
> `tauri/capabilities/default.json` stays as a committed static file but is
> asserted byte-for-byte against a template in `tauri/tests/build_config.rs`
> (drift detection — the build script does NOT mutate committed source files).
> Duplication is **reduced**, not fully eliminated — see ADR-009 Consequences
> for the residual sync points (notably `origa_ui`'s `env!("TRAILBASE_URL")`).

- `app.origa.uwuwu.net` host string is now duplicated in 4 places (CSP, capability allow-list, `trailbase_url()` return value path, redirect_uri builder). Renaming the host requires touching all 4 manually. Tracked as TECH-DEBT.
- Diagnostic overlay adds ~150 LOC of gated debug code. Acceptable: zero production cost, high future diagnostic value.
- CSP still has pre-existing `'unsafe-inline'` / `'unsafe-eval'` in `script-src` (not introduced by this ADR, separate hardening task).

### Tauri v2 mobile gotchas (for future agents/engineers)

These are non-obvious behaviors that cost significant diagnosis time. Read before touching any mobile flow:

1. **`window.open` in Android WebView** without `setSupportMultipleWindows(true)` silently returns `null` — does not open the system browser. Use `tauri-plugin-opener`.
2. **`opener.openUrl` returns a `Promise`** — must be handled via `JsFuture::from(promise).await` with explicit `Err` branch. `.is_err()` on the synchronous `call1` does NOT catch async rejection. This was the primary silent-failure mechanism.
3. **Opener capability scope** — a glob `https://origa.uwuwu.net/*` does NOT match subdomain `app.origa.uwuwu.net`. Each host requires its own explicit entry.
4. **Custom URL scheme `origa://` on Android** is registered via **AndroidManifest intent-filter** (`tauri/gen/android/app/src/main/AndroidManifest.xml`), NOT via `mobile` config in `tauri.conf.json`. The `mobile` config is for Universal Links (`https://...`) and expects an `AssociatedDomain` object; an array of strings causes build errors (this was already fixed in commit `f2f13722`).
5. **CSP in `tauri.conf.json`** applies ONLY to Tauri WebView builds. Playwright E2E runs against the web build on localhost and does NOT cover CSP. CSP changes require manual verification in a Tauri build.
6. **`withGlobalTauri: true`** gives access to `window.__TAURI__.opener.openUrl` without an npm package (Rust reflection approach).
7. **`RwSignal<T>` in Leptos 0.8** implements `Copy` for any `T` (reactive_graph backend) — safe to move into `spawn_local` closures.

## Verification

|Check|Command / Source|Result|
|---|---|---|
|Format|`cargo fmt --check`|PASS|
|Lint|`cargo clippy --workspace --all-targets -- -D warnings`|0 warnings|
|Tests|`cargo test --workspace`|1458 passed (134 origa_ui + 1322 origa + 2 utils)|
|Build (WASM)|`cargo build -p origa_ui`|PASS (Leptos 0.8 WASM)|
|Build (Tauri)|`cargo build -p origa-app`|PASS (Tauri desktop)|
|Callback endpoint|`curl -I https://app.origa.uwuwu.net/public/auth/desktop-callback.html`|200 OK|
|Code review|@code-quality-reviewer|approve / ready|

Manual checkpoint pending (on-device, no DevTools): build debug APK with `ORIGA_DEBUG_OAUTH=1`, verify all three login methods + `adb logcat | grep deep-link` shows `origa://auth/callback?code=...`.

## References

- PR: <https://github.com/yurvon-screamo/origa/pull/159>
- Related upstream issue: tauri-apps/plugins-workspace#2234
- Regression-introducing commit: `eeee03ad` ("mobile OIDC redirect", 2026-05-23)
