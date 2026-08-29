# ADR-041: Direct-link Windows installer on the CDN for Microsoft Store

## Status

**Superseded by [ADR-042](ADR-042-msix-microsoft-store.md)** (2026-08-25) —
the Store submission moved from a linked EXE to an MSIX package; the CDN
mirror, its workflow and its tests were removed.

Originally **Accepted** (2026-08-22).

## Date

2026-08-22.

## Context

Microsoft Store rejected the Origa submission: the store requires a
**direct** installer URL that answers `HTTP 200`, but GitHub Releases asset
URLs answer with a `302` redirect to `objects.githubusercontent.com`. The
store's validation follows redirects nowhere.

Origa already operates an S3-backed public CDN:
`https://s3.origa.uwuwu.net` (Railway `s3-proxy` → Tigris bucket
`adaptable-foodbox-ucep7wx`, DNS on Cloudflare after PR #372 reverted the
ADR-037 user-Tigris migration). Verified live: the CDN answers object GETs
with `200` directly — no redirects, Range supported, CORS `GET, HEAD`.

The release pipeline (`tauri.yml`) already ships per-release artifacts to
GitHub Releases (`release` job), RuStore (`upload-rustore`, isolated), and
the Apple stores (`build-apple`). Adding one isolated upload job fits the
existing pattern.

## Decision

Mirror the **stable-only** Windows NSIS installer — the fixed-name ADR-025
alias `Origa_x64-setup.exe` — into the existing CDN bucket under a new
`releases/` prefix on every stable tag push:

| Key | Purpose | Cache-Control |
| --- | --- | --- |
| `releases/<X.Y.Z>/Origa_x64-setup.exe` | permanent versioned archive — **the URL submitted to the store** | `public, max-age=300, must-revalidate` (release-updated) |
| `releases/latest/Origa_x64-setup.exe` | convenience alias (landing page, manual testing) | `no-cache` |

**Submission contract learned from the store's validation (2026-08-23):**
the store REQUIRES the submitted URL to be versioned — it parses a version
out of the path (its example is `.../downloads/1.1/setup.exe`) and rejects
anything it cannot parse as `X.Y.Z`. Two consequences: (1) the version
segment carries NO `v` prefix — `releases/v0.6.4/` fails validation with
the generic "must point to a versioned package" error while
`releases/0.6.4/` passes; (2) the submitted URL is the VERSIONED key per
release (each app update = new URL + resubmission), so the `latest` alias
is NOT what goes into the store form — it exists for humans, not for the
store bot.

`https://s3.origa.uwuwu.net/releases/<X.Y.Z>/Origa_x64-setup.exe` is the
per-release store URL. RC/prerelease tags are gated out (stable-only
contract at three layers: `tauri.yml` if-clause, reusable-workflow
self-gate, strict `^\d+\.\d+\.\d+$` version validation in the script).

### Why the existing Railway bucket

The orphaned user-owned Tigris bucket `origa-cdn` (ADR-037, still ~4 GB,
pending cleanup) could have been revived with zero egress fees. Rejected:
it is scheduled for decommission, would re-introduce a second
storage vendor mid-cleanup, and MS Store downloads are rare enough that
Railway egress exposure is negligible (monitor via Railway metrics; the
ADR-037 investigation shows how). A new clean bucket was rejected for the
same one-more-vendor reason. Zero new infrastructure beats marginally
cheaper rare downloads.

### Why versioned keys are NOT immutable

NSIS builds are not byte-reproducible: re-running a tag produces different
bytes. If the versioned key were `immutable` and a re-run overwrote it, the
CDN edge would serve the year-long cached copy of the OLD bytes — exactly
the PR #182 poisoning pattern. `must-revalidate` bounds staleness to ~5 min
and self-heals overwrites. The MS Store link (`releases/latest/`) is
`no-cache`, so a submission always validates against fresh bytes.

### Multipart scale

The installer is ~58 MB. With the deploy scripts' 16 KB multipart chunk
(that threshold exists to force multipart on *small* files against T3's
~24 KB single-PUT limit) this would be ~3.5k sequential 16 KB PUTs per key.
The release upload uses **8 MB parts**: 7 parts per key
(58 372 366 ÷ 8 388 608 = 6.96), ~14 part-PUTs per release. 8 MB parts are
empirically proven on T3 — the aws CLI historically auto-multiparted above
its own 8 MB threshold with such parts (the pre-#223 deploy path for the
118 MB whisper models).

### Upload order and verification

Versioned key uploads FIRST, `latest` second (pairing invariant: the store
alias must never point at a release whose archive failed to land).
After both uploads:

1. **Authenticated HEAD** per key — `Cache-Control` and `Content-Length`
   are mandatory checks. The stored SHA256 is compared only when the store
   returns it in simple format; multipart uploads produce a composite
   `<base64>-<parts>` checksum that never equals the plain file hash, so
   composite or absent checksums degrade to an informational note.
2. **Full public GET** of both URLs with sha256 comparison — the only
   check that exercises the proxy edge (railway-hikari) on the real ~58 MB
   body; HEAD cannot catch proxy timeouts/limits on large bodies.

Uploads request `ChecksumAlgorithm=SHA256`; if the store rejects the
checksum extension on multipart creation, the upload retries once without
a checksum (wide catch is deliberate — a non-checksum failure simply fails
the retry identically) and integrity rests on the GET check.

### Credential contract

CI passes scoped data-plane S3 keys (the existing `[origa]` deploy
principal — no new grants) via environment variables. Contract (applies to
every script on `scripts/_cdn_s3.py`): env `AWS_ACCESS_KEY_ID`, when set,
takes precedence over the local `[origa]` profile. GitHub secrets:
`ORIGA_CDN_S3_ACCESS_KEY_ID`, `ORIGA_CDN_S3_SECRET_ACCESS_KEY`.
Dependencies are installed with `uv sync --frozen` from `scripts/uv.lock`
— single source of truth, no manual version pins.

### CI isolation

The `upload-release-cdn` job (tauri.yml → reusable
`_upload-release-cdn.yml`) is intentionally **not** a dependency of the
`release` job, mirroring `upload-rustore`: a transient CDN/S3 failure must
not block the GitHub Release. GitHub remains the canonical release surface;
the CDN copy serves the store link.

## Manual bootstrap (first submission)

Do not wait for the next stable tag — upload the current stable installer
by hand and resubmit to MS Store:

```powershell
# Download Origa_x64-setup.exe from the latest GitHub Release into <dir>, then:
cd scripts
uv run python upload_release_artifacts.py --version <X.Y.Z> --artifact-dir <dir>
# The script verifies both keys (HEAD + full public GET) and prints the
# MS Store link: https://s3.origa.uwuwu.net/releases/latest/Origa_x64-setup.exe
```

## Consequences

### Positive

- MS Store submissions validate: direct `200` link, stable across releases.
- Zero new infrastructure, one new isolated CI job, one new script.
- Full verification chain on every upload (metadata policy + byte
  integrity through the actual public path).

### Negative

- ~58 MB × 2 keys per stable release stored indefinitely. Lifecycle rules
  are deliberately NOT introduced: the archive is small, the cost is
  trivial, and old installers may be needed for store re-validation.
  Revisit if release cadence × size grows.
- Railway egress if the link leaks publicly (MS Store fetches are rare;
  monitored via Railway metrics — see ADR-037 for the investigation
  workflow).
- `releases/<X.Y.Z>*` re-upload on tag re-run briefly serves mixed cache states
  (≤5 min, bounded by must-revalidate).

## Alternatives considered

- **Revive the orphaned user-Tigris bucket** — zero egress, but
  reintroduces a vendor mid-cleanup; rejected (see above).
- **Host on the landing service** — requires serving large binaries from
  the SSR app on Railway; new runtime concern for zero benefit.
- **GitHub Pages** — 100 MB per-file limit headroom is thin for a growing
  installer, and Pages is not an artifact-distribution surface.
- **Presigned S3 URLs** — expire; the store needs a durable public URL.

## References

- ADR-025 — fixed-name installer alias (`Origa_x64-setup.exe`).
- ADR-037 + PR #372 — user-Tigris migration and its revert (current CDN
  topology: Railway s3-proxy → `adaptable-foodbox-ucep7wx`).
- PR #182 / ADR-016 — edge-cache poisoning precedent behind the
  non-immutable versioned keys.
- `scripts/upload_release_artifacts.py` — the uploader + verifier.
- `scripts/_cdn_s3.py` — S3 transport (`upload_file` chunk/checksum
  params, `stat_object`, credential contract).
- `.github/workflows/_upload-release-cdn.yml`, `tauri.yml`
  (`upload-release-cdn` job).
