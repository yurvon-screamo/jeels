# ADR-044: Apple OAuth callback via direct server-side scheme redirect

## Status

Accepted

## Date

2026-09-02

## Context

Mac App Store review of v0.7.3 rejected Sign in with Apple (Guideline
2.1(a): "the app showed error message while logging in via Sign in with
Apple"). Production HTTP logs (Railway, 2026-09-02 09:51–09:53) show the
reviewer attempted SIWA four times; every attempt completed server-side —

```
GET  /api/auth/v1/oauth/apple/login            303
POST /api/auth/v1/oauth/apple/callback         303   (code exchanged)
GET  /public/auth/desktop-callback.html        200
—— no follow-up code-exchange request from the app ——
```

— and stalled at the interstitial page. Root cause:
`desktop-callback.html` hands control back to the app with a JavaScript
hop (`window.location.href = "origa://…"`), but
`ASWebAuthenticationSession` on macOS only intercepts
**redirect-initiated** navigations to the callback scheme. A JS hop from
an intermediate page is silently dropped, the auth sheet hangs forever,
and the app never receives the callback.

Why the interstitial page exists at all: on opener platforms
(Android/Windows/Linux) the default browser cannot be trusted to follow
custom-scheme redirects on its own — Chromium blocks scheme navigations
without a fresh user gesture (the no-consent-screen re-login case), so
`desktop-callback.html` carries a manual fallback button that is the only
guaranteed delivery path there (see wiki
`oauth-desktop-callback-chromium`, ADR-008).

## Decision

Split the `redirect_uri` per platform class (`native_oauth_redirect_uri`
in `origa_ui/src/pages/login/oauth_buttons.rs`):

- **Apple platforms (macOS/iOS), inside Tauri**: `origa://auth/callback`.
  TrailBase answers the provider callback with a server-side 303 straight
  to the scheme (private-use URI scheme, RFC 8252), which the auth session
  intercepts reliably. The frontend already receives the intercepted URL
  via the `tauri-plugin-aswebauth` Promise and feeds it into the shared
  `process_oauth_url` contract — no other code changes.
- **Other native platforms**: keep `desktop-callback.html` byte-identical
  to the legacy value (released builds depend on this exact URI being
  accepted server-side; Chromium needs the interstitial fallback button).
- **Web**: same-origin `/login`, unchanged.

Server-side prerequisite: `auth.custom_uri_schemes: "origa"` in the
production TrailBase `config.textproto` (Railway volume
`/app/traildepot/config.textproto`). `validate_redirect_impl` in the
TrailBase fork already supports custom schemes; the config line is
additive and inert for existing flows (same-host and relative redirects
pass separate validation branches). Deployed 2026-09-02, verified with
positive (`origa://` → 303), negative (`evil://` → 400) and legacy
(`desktop-callback.html` → 303) probes.

## Threat model

`custom_uri_schemes` allow-lists the **scheme only**: any
`origa://<any-host>/<any-path>` redirect_uri is accepted. The callback's
integrity control is PKCE — the verifier lives on the client and the
authorization code is bound to the challenge server-side — which is
unchanged. An attacker who can register the `origa` scheme on the same
machine could complete a login for their own account in our app; the
`ASWebAuthenticationSession` guarantee (only the session that started the
flow receives the callback) mitigates this on Apple platforms, and this
is the standard trade-off of the RFC 8252 private-use scheme pattern.

## Rollback coupling

Before release 0.7.4: revert = remove the config line + restart (the app
change alone is inert for non-Apple platforms). **After** 0.7.4 ships:
reverting the config line breaks login on Apple builds (`400 invalid
redirect`) until a revert release goes out — config rollback and app
rollback must travel together.

## Verification

- Sandbox boot of the exact production image
  (`ghcr.io/yurvon-screamo/trailbase:apple-formpost`, `v0.33.5-4-gbff9bd50`) with the edited
  config: parse + vault merge + full `validate_config` green.
- Unit tests: `redirect_uri_tests` (Apple → scheme; non-Apple →
  byte-identical legacy value, which is what legalizes skipping
  Win/Android regression smoke for this change).
- Manual (gates the 0.7.4 resubmission): complete all three provider
  logins on macOS — in `cargo tauri dev` **and** in the signed
  store-like CI artifact (dev ≠ signed sandbox for
  ASWebAuthenticationSession presentation).

## Consequences

- iOS gets the same fix for free (it shared the broken JS-hop pattern).
- `desktop-callback.html` stays deployed forever-ish: released
  Android/Windows/Linux builds keep pointing at it. New builds of those
  platforms also keep using it (interstitial is load-bearing there).
- ADR-008's redirect_uri contract (desktop-callback for all native
  platforms) is superseded for Apple platforms by this ADR.

## References

- App Review rejection b67a5273-2dd2-47f0-8920-dea001eafe75 (Guideline 4,
  2.1(a), 2.4.5(i))
- RFC 8252 (OAuth 2.0 for Native Apps, private-use URI scheme)
- Chromium gesture requirement for external-protocol redirects
  (crbug.com/41328386, AppAuth-Android #448)
- wiki: `oauth-desktop-callback-chromium`, `trailbase-apple-oauth-form-post`
