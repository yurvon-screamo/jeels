# ADR-033: Apple Platforms — Separate Distribution Channels + Dynamic Fallback Cleanup

## Status

Accepted

## Date

2026-07-28

## Context

Origa ships on five platforms (Windows, Linux, macOS, iOS, Android) via
two fundamentally different distribution models:

1. **Direct download** (Windows, Linux, macOS desktop) — GitHub Releases
   host fixed-name aliases (`Origa_x64-setup.exe`,
   `Origa_amd64.AppImage`, `Origa_macos-arm64.zip`) that the landing
   page links via `/releases/latest/download/<alias>` (ADR-025). Builds
   are unsigned or self-signed; users install at their own risk.
2. **Curated stores** (iOS App Store, Mac App Store, Google Play,
   RuStore) — Apple and Google require signed builds, review process,
   and per-store metadata. Builds are uploaded to App Store Connect /
   Play Console via `xcrun altool` / RuStore API; users install from
   the store app.

macOS is unique: it appears in **both** channels simultaneously.
Desktop-distribution macOS users (Apple Silicon direct-download via
`Origa_macos-arm64.zip`) and Mac App Store users (sandboxed, signed
.pkg) get different binaries. The legacy desktop macOS pipeline
(`cargo-zigbuild` on Linux runner, unsigned) is **incompatible** with
Mac App Store requirements (sandbox entitlements, Mac Distribution
certificate signing, Universal Binary).

Apple CI/CD pipeline was added in PR #299 (`feat/apple-ci-cd-app-store`)
as a new reusable workflow `_build-tauri-apple.yml` running on
`macos-15` GitHub runner. It produces:

- iOS `.ipa` — App Store export, uploaded via `xcrun altool --type ios`
- macOS `.pkg` — Universal Binary (arm64 + x86_64), App Sandbox
  entitlements, signed with Mac Distribution certs, uploaded via
  `xcrun altool --type mac`

The legacy `_build-tauri.yml` `build-macos` job (cargo-zigbuild on
Linux runner) continues to produce the unsigned desktop-distribution
`Origa_macos-arm64.zip`. The two pipelines are **completely
independent** — different runners, different signing, different
distribution channels.

## Decision

### 1. Two macOS distribution channels coexist

- `_build-tauri.yml` `build-macos` job — desktop-distribution
  `Origa_macos-arm64.zip` for direct download (landing page link,
  ADR-025). Unsigned, fast, free Linux runner.
- `_build-tauri-apple.yml` `build-macos` job — Mac App Store signed
  `.pkg` via TestFlight. Signed, sandboxed, slow (`macos-15` runner,
  ~30 min).

### 2. Dynamic fallback cleanup in release job

`tauri.yml` release job copies ALL platform assets including the legacy
`Origa_macos-arm64.zip`. After copy, a conditional step removes it
based on Apple pipeline success:

```bash
if [ "$MACOS_UPLOAD_STATUS" = "success" ]; then
  rm -f release_assets/Origa_macos-arm64.zip
fi
```

`MACOS_UPLOAD_STATUS` is propagated from `_build-tauri-apple.yml`
outputs (`macos-upload-status`).

**Self-healing semantics:**

- Apple upload succeeded → legacy zip dropped, users install via App
  Store only (cleaner release UI, single source of truth).
- Apple upload failed/skipped (secrets missing, Dependabot, signing
  error) → legacy zip retained as fallback, landing page link still
  resolves.
- No manual `confirm_apple_working` flag — the pipeline decides
  automatically on every release.

### 3. `build-apple` does NOT gate `release` job

`tauri.yml` release job `if:` keeps `always() && needs.build-tauri.result
== 'success'`. Apple build failure does not block GitHub Release for
other platforms. `needs: build-apple` is included solely to make
`needs.build-apple.outputs.*` available for the dynamic cleanup
conditional.

This isolation is deliberate: Apple pipeline is new (introduced
2026-07-28), runs on slow `macos-15` runner, requires external secrets
that may be absent on forks. Coupling release publishing to Apple
success would block every Windows/Linux/Android release whenever Apple
has an issue.

### 4. App Store `app-store` cargo feature

To comply with Mac App Store Guidelines:

- **2.4.5(vii)**: "They must use the Mac App Store to distribute
  updates; other update mechanisms are not allowed." Self-update plugin
  (`tauri-plugin-updater`) must be disabled.
- **2.5.1**: "Apps may not access or claim to access any private user
  data without permission." Debug code (release-devtools) is grounds
  for rejection.

The `app-store` cargo feature gates out these via `#[cfg(not(feature =
"app-store"))]` on plugin registration in `tauri/src/lib.rs`. App Store
builds use `--no-default-features --features app-store`.

**Compromise:** `tauri-plugin-updater` and `tauri-plugin-single-instance`
remain in the Cargo dependency tree (non-optional) because Tauri's
capability schema validation in `tauri::generate_context!()` requires
the `updater:default` permission entry to resolve even when the plugin
is unregistered at runtime. Symbols stay in the binary; runtime
self-update behavior is fully disabled. Apple's review process tests
runtime behavior, not static symbol presence — so this complies with
the guidelines.

### 5. Privacy manifest (`PrivacyInfo.xcprivacy`)

Required since spring 2024 for both iOS and macOS App Store submission
(ITMS-91053 rejection without it). Declares 3 accessed API categories
with reason codes per Apple TN3183:

- `UserDefaults CA92.1` — tauri-plugin-store account settings
- `FileTimestamp C617.1` — FSRS database in `app_data_dir`
- `SystemBootTime 35F9.1` — FSRS scheduling intervals

The file lives at `tauri/gen/apple/PrivacyInfo.xcprivacy` (canonical
location for iOS Xcode project sources). macOS bundle picks it up via
`tauri.conf.json` `bundle.resources` map form
(`{"gen/apple/PrivacyInfo.xcprivacy": "PrivacyInfo.xcprivacy"}`).

### 6. iOS Info.plist source-of-truth = `project.yml info.properties`

iOS Info.plist usage descriptions (`NSCameraUsageDescription`,
`NSMicrophoneUsageDescription`, `NSPhotoLibraryUsageDescription`,
`CFBundleDisplayName`, `ITSAppUsesNonExemptEncryption`,
`LSApplicationCategoryType`) are declared in
`tauri/gen/apple/project.yml` `info.properties`, NOT edited directly
into `Info.plist`. XcodeGen applies `info.properties` as an overlay
over the file during project generation, so keys declared in
`project.yml` survive `cargo tauri ios build` regeneration.

### 7. macOS signing requires both API Key + Mac Distribution certs

iOS signing path: `cargo tauri ios build` → `xcodebuild
-allowProvisioningUpdates -authenticationKeyPath/ID/IssuerID` → API
key auto-creates iOS Distribution cert + Provisioning Profile, identity
installed in keychain. **4 API Key secrets sufficient.**

macOS signing path: `cargo tauri build --target universal-apple-darwin
--bundles app` → `tauri-bundler` calls `codesign --sign "..."` directly
(**NOT** via xcodebuild). API key not involved. Identity (cert +
private key) must be in keychain before build starts. **3 additional
Mac cert secrets required.**

Total: **7 secrets** for full Apple pipeline:

- `APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_CONTENT`
  (base64 of `.p8`), `APPLE_TEAM_ID` — iOS + upload (both platforms)
- `APPLE_MAC_APP_CERT_P12` (base64 of "Mac App Distribution" `.p12`),
  `APPLE_MAC_INSTALLER_CERT_P12` (base64 of "Mac Installer
  Distribution" `.p12`), `APPLE_MAC_CERT_PASSWORD` — macOS signing

## Consequences

### Positive

- macOS users can install via Mac App Store (sandboxed, signed,
  auto-updated) — preferred UX for non-technical users.
- iOS users get TestFlight → App Store distribution.
- Landing page direct-download link remains valid (no broken UX) even
  when Apple pipeline has issues.
- PR-level macOS smoke test preserved (cargo-zigbuild cross-compile on
  Linux, free) — catches macOS-incompatible Rust regressions before
  they reach a release tag.
- No coupling between Apple pipeline success and Windows/Linux/Android
  release publishing.
- Self-healing dynamic cleanup: no manual `confirm_apple_working` flag
  to remember.

### Negative

- Two macOS pipelines to maintain (legacy cargo-zigbuild + new
  macos-15 App Store). Cleanup of legacy deferred indefinitely — the
  fallback semantics are useful.
- 7 Apple secrets to manage (vs 4 for iOS-only or 0 for non-Apple).
- `tauri-plugin-updater` symbols present in App Store binary despite
  runtime gating (Tauri capability validation constraint).
- `bundle.createUpdaterArtifacts: true` remains unconditional in
  `tauri.conf.json` — App Store bundle contains `.sig` files that
  nothing references. Future work: gate via `build.rs` TAURI_CONFIG
  merge patch when feature `app-store` is active.
- macOS runner minutes cost (~10x Linux). Mitigated by Apple
  builds running only on tag pushes and manual dispatch, not on PRs.

### Risks

- **Tauri #15230** (codesign Requirement check failure for Mac App
  Store) workaround via manual `codesign --requirements` may break if
  Tauri upstream fixes the bug differently. Mitigation: workaround
  documented in `_build-tauri-apple.yml` step comments with link to
  the issue.
- **macOS runner image** with iOS 26 SDK required (`deploymentTarget.iOS:
  26.0`). GitHub Actions `macos-15` runner ships Xcode 26 by July 2026;
  if absent, `xcodebuild -showsdks | grep iphoneos26` check fails fast
  with a clear error.
- **Mac cert creation** requires Mac access (Keychain Access → CSR →
  Portal → export `.p12`). CI cannot generate these — user must
  pre-create and store as GitHub secrets. Blocks first macOS run until
  user does the manual setup.
