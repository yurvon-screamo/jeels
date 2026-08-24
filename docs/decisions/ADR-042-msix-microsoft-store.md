# ADR-042: Microsoft Store distribution via MSIX (winapp/MakeAppx packaging)

## Status

**Accepted** (2026-08-25). Supersedes
[ADR-041](ADR-041-installer-direct-link-cdn.md).

## Date

2026-08-25.

## Context

The Microsoft Store rejected the linked-EXE submission of 0.6.4 under policy
10.2.9: the submitted NSIS installer must be Authenticode-signed with a
certificate chaining to the Microsoft Trusted Root Program (SHA256 or
better, RFC3161 timestamp). Self-signed certificates do not qualify.

Every path to such a certificate was evaluated and is unavailable to this
project's maintainer:

| Path | Verdict |
| --- | --- |
| Azure Artifact Signing (ex-Trusted Signing — recommended in the rejection letter) | Closed to new subscribers since 2025-04-02: US/Canada organizations with ≥3 years of verifiable history only; individual onboarding paused until GA. |
| Public CA OV/EV certificate | Certum explicitly refuses applicants from Russia/Belarus/etc.; SSL.com/Sectigo/DigiCert stopped serving RF. Only realistic access is foreign residency/entity, which this project does not have. |
| MSIX package | **Free**: the Store re-signs MSIX packages with a Microsoft certificate at publication ("code signing, hosting" benefits quoted in the rejection letter itself). |

Tauri's bundler cannot emit MSIX (EXE/MSI only), but packaging a finished
Win32 exe into MSIX is mechanical, and Microsoft ships first-class tooling:
the `winapp` CLI documents a dedicated Tauri flow, and open-source precedent
exists (GeoLibre ships a Tauri app to the Store as MSIX with published
scripts).

## Decision

Distribute Origa on the Microsoft Store as an **MSIX packaged app**, built by
a dedicated CI pipeline that is fully isolated from the existing release
train. Direct downloads (NSIS from GitHub Releases / landing page) are
unchanged and remain unsigned — SmartScreen reputation for that channel is a
separate, unsolved problem.

### Packaging

`tauri/scripts/build-msix.ps1` is the single source of truth for staging +
packaging; CI calls it verbatim:

1. Version mapping: `^\d+\.\d+\.\d+$` → `X.Y.Z.0`; anything else →
   `0.0.0.<run_number>` (smoke-only; never submit — the Store requires
   strictly increasing versions per product).
2. Store-flavored build: `ORIGA_APP_STORE=1 npx tauri build --no-bundle`.
3. Stage `Origa.exe` (+ `WebView2Loader.dll` when emitted) with the committed
   `tauri/msix/Package.appxmanifest` + `Assets/`. The Apple-only
   `PrivacyInfo.xcprivacy` resource mapping is deliberately not staged.
4. Inject Partner Center identity placeholders + version into the staged
   manifest.
5. Pack via Windows SDK `MakeAppx.exe`, sign via `signtool.exe` with an
   ephemeral self-signed cert whose CN equals the manifest Publisher. The
   `.pfx` + password travel WITH the CI artifact — without them nobody can
   install the package locally. The self-signature exists purely for local
   smoke testing; the Store replaces any signature at publication.

Deviation from the original plan note: winapp CLI was demoted from primary
to documented alternative. Its `pack`/cert flags around pfx passwords were
not verifiably non-interactive, while MakeAppx/signtool are guaranteed on
GitHub windows runners and on any MSVC-Rust machine. The manifest/staging
layout is winapp-compatible either way.

### Manifest contract (`tauri/msix/Package.appxmanifest`)

- Identity Name/Publisher/PublisherDisplayName: `__PARTNER_CENTER_*__`
  placeholders, filled after the "MSIX or PWA app" product is created in
  Partner Center (values live in Product management → Product Identity;
  certification compares them byte-for-byte).
- Language `en-US` (mandatory field; UI-localized EN/RU lives inside the
  app, not in MSIX resources).
- TargetDeviceFamily Windows.Desktop MinVersion 10.0.19041.0 /
  MaxVersionTested 10.0.26100.0.
- `runFullTrust` capability + `Windows.FullTrustApplication` entry point:
  mandatory for packaged Win32 apps; certification warns but grants.
- `origa://` protocol extension: OAuth deep-link callbacks arrive through
  the manifest instead of NSIS registry entries.

### Store-build code gate (policy 10.2.5)

Store policy forbids self-update outside the Store. The pre-existing
macOS App Store mechanism (`ORIGA_APP_STORE=1` → rustc-cfg `app_store`)
now also gates the Windows updater path: `mod updater_commands`, its
imports, the plugin/state registration block, and both IPC commands are
`#[cfg(all(any(windows, target_os = "linux"), not(app_store)))]`. A new
unconditional `is_store_build()` command lets the WASM side skip the
startup update check (single-instance stays ungated — MSIX protocol
activation depends on it). Regression-tested in
`tauri/tests/build_config.rs::lib_rs_compiles_update_machinery_out_of_store_builds`.

### CI isolation

`build-windows-store` lives in its own reusable workflow
(`_build-windows-store.yml`) and is called from `tauri.yml` as a SIBLING of
`build-tauri`, deliberately absent from `release.needs` — same doctrine as
`upload-rustore`: a transient store-package failure must never block the
GitHub Release. Inside one reusable workflow this cannot be expressed via
needs-tricks (any failed job fails the workflow conclusion), hence the
separate file. Trigger: stable tags only, plus a `force_store_msix`
workflow_dispatch escape hatch for smoke builds.

### CDN mirror removal (supersedes ADR-041)

ADR-041's `releases/` CDN mirror existed solely to give the store validator
a direct versioned URL for the linked EXE. With MSIX, the package is
uploaded directly to Partner Center and hosted by Microsoft — the mirror,
its workflow (`_upload-release-cdn.yml`), its script
(`upload_release_artifacts.py`), its tests, and the `releases/*` cache rules
in `_cdn_cache.py` are removed. Runtime code has zero `releases/`
references (verified by grep).

## Consequences

- **Two Windows channels with two update mechanisms.** Direct downloads
  keep the Tauri updater (`latest.json`); Store installs update through the
  Store (updater compiled out). Channels do not interfere.
- **Dual installation is possible and asymmetric.** An NSIS user installing
  the Store version gets a second Start-menu entry. MSIX filesystem
  virtualization is read-through: reads fall back to real AppData when no
  private copy exists (existing user data partially visible), writes go to
  the private package layer, and the WebView2 profile is fully isolated.
  No data migration is offered — accepted, since the Store channel targets
  new users.
- **WebView2 runtime is not bootstrapped.** `webviewInstallMode` applies to
  installers only; an MSIX machine needs the evergreen runtime already
  present (preinstalled on Win11 and updated Win10; missing on clean
  un-updated Win10 — accepted channel limitation, Win10 is EOL/ESU-era).
- **First Windows×app_store compile happens at the first stable tag** (PR
  CI checks only the Linux flavor of the gate). Accepted residual: if it
  breaks, the release itself is unaffected — the Store package is simply
  absent that tag.
- **Re-submission requires a version bump.** The Store rejects duplicate
  versions per product even though ADR-041 tolerated re-tag overwrites.
- **Partner Center product must be recreated** (one-time manual step): the
  old "EXE or MSI app" product blocks the name; delete it, create
  "MSIX or PWA app", copy Product Identity into the manifest placeholders,
  dispatch-rebuild, upload the `.msix` in the submission's Packages page.
