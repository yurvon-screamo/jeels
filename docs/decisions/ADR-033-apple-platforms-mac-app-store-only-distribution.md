# ADR-033: Apple Platforms — Mac App Store Only Distribution

## Status

Accepted (supersedes the original "Two Distribution Channels + Dynamic Fallback"
decision from 2026-07-28; the legacy cargo-zigbuild desktop-macOS path was
removed in PR #308 on 2026-07-29 after 5 failed Apple CI runs and
re-evaluation against Tauri's official App Store distribution guide).

## Date

2026-07-29

## Context

Origa ships on five platforms (Windows, Linux, macOS, iOS, Android).
Windows and Linux use direct download via GitHub Releases (`Origa_x64-setup.exe`,
`Origa_amd64.AppImage`, `Origa_amd64.deb`) per ADR-025. iOS and macOS target
the Apple App Store (signed `.ipa` and `.pkg`, sandboxed, TestFlight +
App Store review). Android uses Google Play + RuStore.

The first version of this ADR (2026-07-28, PR #299) decided to keep
**two parallel macOS distribution channels**:

1. Legacy desktop-distribution `Origa_macos-arm64.zip` via cargo-zigbuild
   cross-compile on a Linux runner (fast, free, unsigned). Used by the
   landing page's direct-download link `/releases/latest/download/
   Origa_macos-arm64.zip`.
2. Mac App Store signed `.pkg` via the new `_build-tauri-apple.yml`
   pipeline on a `macos-15` runner. Uploaded to App Store Connect via
   `xcrun altool`.

A dynamic cleanup gate in the release job kept the legacy zip as
fallback when Apple upload failed, dropped it when upload succeeded.

After five iterations of the Apple CI pipeline (PRs 299, 302, 304, 305,
306, 307) and a careful re-read of Tauri's official documentation

(<https://v2.tauri.app/distribute/sign/macos/> and
<https://v2.tauri.app/distribute/app-store/>), the dual-channel approach
was re-evaluated and abandoned:

- The legacy zip was **unsigned** — any macOS user clicking the landing
  download link got a binary macOS Gatekeeper warns about. For a paid
  Apple Developer Program member ($99/year), shipping unsigned binaries
  to macOS users is a worse experience than asking them to wait for the
  App Store listing.
- The two pipelines had **divergent outputs** with different capabilities
  (no sandbox, no entitlements, no notarization on the legacy side).
  Documenting and supporting both confused both users and CI.
- The dynamic cleanup gate (drop legacy zip when Apple upload succeeds)
  was **complexity without lasting value** — once the App Store pipeline
  is green on tag pushes, the fallback is dead code.
- The legacy `cargo-zigbuild` desktop path also functioned as a
  **PR-level macOS compilation smoke test** (catches Rust code that
  breaks macOS but not Linux/Windows). This is a real loss — see
  Consequences.

## Decision

### 1. macOS distribution is Mac App Store only

- iOS App Store: signed `.ipa` via `cargo tauri ios build
  --export-method app-store-connect` on `macos-15` runner.
- Mac App Store: signed Universal Binary `.pkg` via `cargo tauri build
  --target universal-apple-darwin --bundles app` + `xcrun productbuild
  --sign "<installer identity>"` on `macos-15` runner.
- Both uploaded to App Store Connect via `xcrun altool --upload-app`.

No direct-download macOS binary is produced or shipped. The landing
page macOS card uses the same "coming soon" treatment as iOS until the
Mac App Store listing goes live.

### 2. Apple builds do not gate the GitHub Release

`tauri.yml` release job `if:` keeps `always() && needs.build-tauri.result
== 'success'`. Apple build failure does not block GitHub Release for
Windows/Linux/Android. `needs: build-apple` is included in `needs:` list
solely to make `needs.build-apple.outputs.ios-upload-status` /
`macos-upload-status` available for the release body markdown (which
conditionally advertises TestFlight vs "build pending").

This isolation is deliberate: Apple pipeline runs on slow `macos-15`
runner (~30 min/target), requires external secrets that may be absent
on forks, and targets a different distribution channel. Coupling release
publishing to Apple success would block every Windows/Linux/Android
release whenever Apple has an issue.

### 3. App Store compliance via `ORIGA_APP_STORE` env var + `--config` flag

Two complementary mechanisms disable self-updater and devtools for App
Store builds (Mac App Store 2.4.5(vii), 2.5.1):

- **`tauri/build.rs`** checks for `ORIGA_APP_STORE=1` env var and emits
  `cargo:rustc-cfg=app_store`. `tauri/src/lib.rs` uses `#[cfg(not(app_store))]`
  to skip `Builder::plugin()` registration for `tauri-plugin-updater`,
  `tauri-plugin-single-instance`, `release-devtools`. Runtime
  self-update behavior is fully disabled.
- **`cargo tauri ios build` / `cargo tauri build`** invocations in CI
  pass `--config '{"bundle":{"createUpdaterArtifacts":false},
  "plugins":{"updater":null}}'` as RFC 7396 JSON Merge Patch. Tauri-cli
  natively merges this onto `tauri.conf.json`, removing the updater
  section entirely so tauri-bundler does not expect
  `TAURI_SIGNING_PRIVATE_KEY` for macOS bundle signing.

Why both: tauri-cli does not propagate `--features` / `--no-default-features`
to cargo (so cargo feature alone does not work for `cargo tauri build`).
TAURI_CONFIG env var set by build.rs is visible to `tauri::generate_context!()`
macro but **NOT** to tauri-bundler (which re-reads tauri.conf.json
directly). The `--config` CLI flag is the only path that reaches both
the codegen macro and tauri-bundler.

The crates (`tauri-plugin-updater`, `tauri-plugin-single-instance`)
remain in the Cargo dependency tree (non-optional) because Tauri's
capability schema validation in `tauri::generate_context!()` requires
the `updater:default` permission entry to resolve even when the plugin
is unregistered at runtime. Symbols stay in the binary; runtime
behavior is fully disabled. Apple reviewers test runtime behavior, not
static symbol presence — this complies with the guidelines.

### 4. Privacy manifest (`PrivacyInfo.xcprivacy`)

Required since spring 2024 for both iOS and macOS App Store submission
(ITMS-91053 rejection without it). Declares 3 accessed API categories
with reason codes per Apple TN3183:

- `UserDefaults CA92.1` — tauri-plugin-store account settings
- `FileTimestamp C617.1` — FSRS database in `app_data_dir`
- `SystemBootTime 35F9.1` — FSRS scheduling intervals

File location: `tauri/gen/apple/PrivacyInfo.xcprivacy`. Wired into iOS
target sources via `project.yml`; copied to macOS bundle via
`tauri.conf.json` `bundle.resources` map form.

### 5. iOS Info.plist source-of-truth = `project.yml info.properties`

iOS Info.plist usage descriptions (`NSCameraUsageDescription`,
`NSMicrophoneUsageDescription`, `NSPhotoLibraryUsageDescription`,
`CFBundleDisplayName`, `ITSAppUsesNonExemptEncryption`,
`LSApplicationCategoryType`) are declared in
`tauri/gen/apple/project.yml` `info.properties`, NOT edited directly
into `Info.plist`. XcodeGen applies `info.properties` as an overlay
over the file during project generation, so keys declared in
`project.yml` survive `cargo tauri ios build` regeneration.

### 6. macOS signing requires both API Key + Mac Distribution certs

- **iOS signing**: `cargo tauri ios build` → `xcodebuild
  -allowProvisioningUpdates -authenticationKeyPath/ID/IssuerID` →
  API key auto-creates iOS Distribution cert + Provisioning Profile,
  identity installed in keychain. 4 API Key secrets sufficient.
- **macOS signing**: `cargo tauri build --target universal-apple-darwin
  --bundles app` → tauri-bundler calls `codesign --sign "..."`
  directly (NOT via xcodebuild). API key not involved. Identity
  (cert + private key) must be in keychain before build starts.
  3 additional Mac cert secrets required.

Total: **7 secrets** for full Apple pipeline:

- `APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_CONTENT`
  (base64 of `.p8`), `APPLE_TEAM_ID` — iOS signing + upload (both
  platforms)
- `APPLE_MAC_APP_CERT_P12` (base64 of "Mac App Distribution" `.p12`),
  `APPLE_MAC_INSTALLER_CERT_P12` (base64 of "Mac Installer
  Distribution" `.p12`), `APPLE_MAC_CERT_PASSWORD` — macOS signing

### 7. No `--requirements` codesign workaround

Earlier iterations of this ADR documented a manual `codesign --force
--options runtime --requirements "designated => anchor apple generic
and identifier \"net.uwuwu.origa\""` step after `cargo tauri build`
to work around Tauri issue #15230 (codesign Requirement check failure
for Mac App Store).

The workaround was removed in PR #308 because:

- Bash/YAML quoting of inner double-quotes broke the requirement
  specification even when passed via env var →
  `invalid requirement specification` failure.
- Tauri's official App Store distribution guide
  (<https://v2.tauri.app/distribute/app-store/>) does not mention any
  `--requirements` workaround.
- Modern tauri-cli (`tauri-macos-sign v2.3.4` per CI logs, July 2026)
  signs correctly during `cargo tauri build`, and `productbuild --sign`
  accepts the signed `.app` directly.

The correct flow per Tauri docs is:

```bash
tauri build --bundles app --target universal-apple-darwin
xcrun productbuild --sign "<installer identity>" \
  --component <app-path> /Applications <output.pkg>
xcrun altool --upload-app --type mac --file <output.pkg> \
  --apiKey $APPLE_API_KEY_ID --apiIssuer $APPLE_API_ISSUER
```

## Consequences

### Positive

- macOS users get a sandboxed, signed, notarized Mac App Store binary
  — preferred UX for non-technical users, no Gatekeeper warnings.
- iOS users get TestFlight → App Store distribution.
- Single source of truth for macOS binaries (the App Store pipeline).
- No dynamic cleanup gate complexity in release job.
- Unsigned-binary direct-download (security concern) eliminated.
- `_build-tauri.yml` simplified: 3 desktop jobs (Win/Linux/Android)
  instead of 4.

### Negative

- **CI coverage regression**: PRs no longer run macOS compilation smoke
  (the legacy `cargo-zigbuild` cross-compile on Linux runner is gone).
  macOS-incompatible Rust regressions (e.g. using Linux-only API)
  surface only at tag push on `macos-15` runner (~30 min, blocking
  release). Accepted trade-off — to mitigate, contributors should run
  `cargo check --target aarch64-apple-darwin` locally before merging
  macOS-affecting changes, or a lightweight smoke job may be re-added
  in a future PR if regressions become frequent.
- 7 Apple secrets to manage.
- `tauri-plugin-updater` symbols present in App Store binary despite
  runtime gating (Tauri capability validation constraint).
- macOS runner minutes cost (~10x Linux). Mitigated by Apple builds
  running only on tag pushes and manual dispatch, not on PRs.

### Risks

- **`productbuild` without `--requirements`** is not yet verified in
  CI (this ADR documents the post-#308 architecture, but the first
  post-#308 tag-triggered release is the actual verifier). If
  `productbuild` rejects the signed `.app` on Requirement check, the
  `--requirements` workaround must be re-introduced with a different
  bash-quoting strategy (e.g. write the requirement to a temp file
  and pass `--requirements <(cat tmpfile)` via process substitution).
- **macOS runner image** with iOS 26 SDK required (`deploymentTarget.iOS:
  26.0`). GitHub Actions `macos-15` runner ships Xcode 26 by July 2026;
  if absent, `xcodebuild -showsdks | grep iphoneos26` check fails fast
  with a clear error.
- **Mac cert creation** requires Mac access (Keychain Access → CSR →
  Portal → export `.p12`). CI cannot generate these — user must
  pre-create and store as GitHub secrets. Blocks first macOS run until
  user does the manual setup.
- **APPLE_API_KEY_CONTENT secret corruption** (observed during bring-up):
  if the base64-encoded `.p8` is pasted via GitHub web UI with folded
  newlines or truncation, `xcodebuild` fails ~10 min later with
  `CryptoKitASN1Error.invalidPEMDocument`. CI now fail-fasts via
  `openssl pkey -in KEY_PATH -noout` validation right after decode.
- **`xcrun altool` deprecated** by Apple (mid-2026). Works for now, but
  may be removed in a future Xcode version. Migration path: App Store
  Connect API direct upload, or `Transporter` CLI. Not blocking — altool
  expected to be supported through at least 2027.

## Out of scope for this ADR

- **`bundle.createUpdaterArtifacts: true` unconditional in
  `tauri.conf.json`**: kept for desktop distribution (Windows/Linux
  updater). App Store builds override to `false` via `--config` flag
  per Decision §3. No need to make it conditional in the source file.
