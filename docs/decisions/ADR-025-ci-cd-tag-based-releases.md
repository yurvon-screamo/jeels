# ADR-025: Tag-based CI/CD — stable / prerelease / alpha channels

## Status

Accepted

## Date

2026-07-03

## Context

Before this ADR, the CI/CD topology conflated two concerns: **verifying a
commit** (CI) and **shipping it** (CD). Three workflows ran CD on every
successful master CI run via the `workflow_run` trigger:

- `docker.yml` — built and pushed both Docker images to GHCR, then redeployed
  Railway (production landing + UI).
- `tauri.yml` — built all four Tauri clients, regenerated `latest.json`, and
  published/updated a **perpetual release tagged `latest`** on every master CI
  success.

This had several problems:

1. **Production deployed on every master push.** `workflow_run` with
   `conclusion == 'success'` redeployed Railway as soon as CI was green — no
   human gate, no tag, no version bump. A master push was effectively a
   release.
2. **The `latest` pseudo-release accumulated garbage.** `tauri.yml:185`
   created/updated a GitHub release tagged `latest` on every master CI success.
   Because `softprops/action-gh-release` keeps existing assets by default and
   the asset filter used `cp -n` (no-clobber), the `latest` release accumulated
   **130+ assets** across historical runs — WASM bundles (`origa_ui-*.wasm`),
   dictionary binaries (`char_def.bin`, `dict.*`), landing images (`anki.svg`),
   CSS, and `index.html`. It was marked `prerelease: true`, so GitHub excluded
   it from `/releases/latest` resolution — meaning the pseudo-release was
   simultaneously noisy (130 junk assets) and functionally invisible.
3. **No prerelease channel.** The `version` action distinguished only `stable`
   (numeric tag) and `alpha` (everything else). There was no way to cut an `rc`
   build for validation without it being treated as either a full release or an
   alpha.
4. **Inconsistent artifact names.** `_build-tauri.yml` renamed artifacts to
   `origa_latest_*` (lowercase) only when `version_type != 'stable'`. Stable
   builds kept Tauri's default `Origa_<version>_*` (capital). The landing page
   linked to `/releases/latest` (the GitHub release UI), not to specific files,
   so users landed on a web page rather than getting a direct download.
5. **CI verified Tauri/Docker builds only on PRs.** `ci.yml` gated
   `build-tauri`, `docker-build-landing`, and `docker-build-ui` behind
   `if: github.event_name == 'pull_request'`. On master push these were skipped
   — so a master commit could merge, deploy to production via `workflow_run`,
   and never have had its Tauri or Docker build verified by CI (only by the CD
   pipeline itself, where a failure means a failed production deploy).

### Current state at the time of this ADR (verified)

- Stable releases **exist**: 0.2.0 → 0.4.0. The `0.4.0` release is
  `make_latest` (marked "Latest" by GitHub), `prerelease: false`, with assets
  `Origa_0.4.0_x64-setup.exe`, `Origa_0.4.0_amd64.AppImage/.deb`,
  `Origa-macos-arm64.zip`, `app-universal-release.apk`, and a working
  `latest.json`.
- The Tauri updater endpoint (`tauri/tauri.conf.json:56`) is
  `/releases/latest/download/latest.json`. It currently resolves to 0.4.0's
  `latest.json` → **the updater is functional for the 0.4.0 install base.**
- The `latest`-tagged pseudo-release is `prerelease: true`, so it is **excluded**
  from `/releases/latest` resolution. Deleting it is **cosmetic cleanup**
  (removes 130 junk assets), not a functional fix — the updater and
  `/releases/latest` are unaffected by its presence or absence.
- Historical bare tags `0.2.0`–`0.4.0` (without the `v` prefix) exist because
  the old `release` job used `tag_name: ${{ needs.version.outputs.version }}`
  (the version with `v` stripped), creating a second tag alongside the git tag
  `v0.4.0`.

## Decision

### 1. Master push = full CI only; tags = CD

- `ci.yml` triggers on `pull_request`, `push` (master), and `workflow_dispatch`.
  It now runs the **full** CI matrix on every event: lint, test, e2e,
  `build-tauri` (compile verification, no publish), and both `docker-build-*`
  jobs (build verification, `push: false`). The `version`, `build-tauri`,
  `docker-build-landing`, and `docker-build-ui` jobs previously gated on
  `pull_request` now run unconditionally — the `check` (CI Gate) job requires
  `success` for all of them (no longer tolerates `skipped`).
- `docker.yml` and `tauri.yml` trigger **only** on `push: tags: [v*.*.*]` and
  `workflow_dispatch`. The `workflow_run` trigger is removed entirely. There is
  no CD on master push.

### 2. `require-ci-green` restores the "CI green before CD" invariant

Removing `workflow_run` also removed its implicit gate — that CD only ran after
CI concluded `success`. To restore this invariant for tag-triggered CD, both
`docker.yml` and `tauri.yml` gain a first job `require-ci-green` that queries
the GitHub API (via the pre-installed `gh` CLI) for a successful `ci.yml` run
matching the tagged commit's full SHA:

```bash
gh run list --workflow ci.yml --commit <sha> --status success --limit 1
```

Every downstream job lists `require-ci-green` in its `needs`. On
`workflow_dispatch`, the job auto-succeeds (operator escape-hatch — the operator
is the gate). On `push` (tag), it fails closed if no green CI run exists for the
commit. This was empirically verified: `gh run list --commit <full-sha>
--status success` returns the run databaseId, and an empty result prints an
empty string (not `null`), so the `[ -z "$RUN_ID" ]` guard is correct.
`--commit` matches only the full 40-character SHA; `github.sha` in CI is always
full.

### 3. Three version channels via the `version` action

`version/action.yml` now emits `version_type ∈ {stable, prerelease, alpha}`:

- **stable** — tag matches `^v[0-9]+\.[0-9]+\.[0-9]+$` (e.g. `v1.0.0`).
  `version` = `1.0.0`, `version_base` = `1.0.0`.
- **prerelease** — tag matches `^v[0-9]+\.[0-9]+\.[0-9]+-rc[0-9.]*[0-9]+$`,
  covering both `v1.0.0-rc1` and `v1.0.0-rc.1`. A bare `rc` (no trailing digit)
  falls through to alpha. `version` = `1.0.0-rc1`, `version_base` = `1.0.0`.
- **alpha** — everything else (master push, PR, non-conforming tag). Unchanged:
  derives `MAJOR.MINOR.PATCH-alpha.<commit-count>` via `git describe`.

All existing consumers use `== 'stable'` or `!= 'stable'` comparisons, so
adding `prerelease` is backward-compatible — no consumer breaks.

### 4. Release semantics (`tauri.yml` `release` job)

- `if:` requires a `v` tag **and** `version_type ∈ {stable, prerelease}`. An
  alpha-type tag (e.g. a hypothetical `v1.0.0-beta1` → alpha) does **not**
  create a release; build jobs still run (minor waste, acceptable).
- `tag_name: ${{ github.ref_name }}` — the actual git tag (`v1.0.0` or
  `v1.0.0-rc1`). This attaches the release to the tag that triggered the
  workflow and **stops creating duplicate bare tags** (the old
  `tag_name: version` created `0.4.0` alongside `v0.4.0`).
- `make_latest: ${{ version_type == 'stable' }}` — only stable releases become
  GitHub's "Latest". Prereleases are `make_latest: false`, so
  `/releases/latest` keeps pointing at the prior stable until a new stable tag
  supersedes it.
- `prerelease: ${{ version_type == 'prerelease' }}` — rc releases are marked
  prerelease (GitHub excludes them from `/releases/latest`).
- `name` — `Release <version>` for stable, `Pre-release <version>` for
  prerelease.
- `cp -f` (not `-n`) in `Prepare release assets`: every tag is a fresh release,
  no cross-run asset accumulation.

### 5. `latest.json` is stable-only

`generate-latest-json` runs `if: startsWith(github.ref, 'refs/tags/v') &&
version_type == 'stable'`. Prereleases deliberately do **not** refresh the
updater manifest — users on the stable channel never get auto-updated to an rc.
The manifest's URL tag segment is the **git tag** (`github.ref_name`, e.g.
`v1.0.0`) — it must match the release's `tag_name` (§4); using the stripped
version (`1.0.0`) would 404, because the new `tag_name: github.ref_name` no
longer creates a duplicate bare tag the way the old `tag_name: version` did.
The asset filename segment keeps the stripped version (`Origa_1.0.0_*`) because
Tauri stamps bundle names from `tauri.conf.json:version` (no `v`). The
manifest's `version` field is the bare version (`1.0.0`) the updater compares
against. (Research finding: the official `tauri-action` generates `latest.json`
with GitHub API asset URLs — so the manifest is filename-agnostic; only the
landing page needs fixed names.)

The `release` job lists `generate-latest-json` in `needs` (for artifact
ordering — `latest.json` must be uploaded before `release` downloads it), with
`if: always() && ... && (version_type != 'stable' ||
generate-latest-json.result == 'success')`: for stable, a failed
`generate-latest-json` blocks the release (no broken updater manifest); for
prerelease, the skipped `generate-latest-json` is tolerated.

### 6. Dual artifact naming (versioned + fixed alias)

`_build-tauri.yml` produces **two** files per platform after every build:

| Platform | Versioned (release history + updater target) | Fixed alias (landing direct download) |
| --- | --- | --- |
| Windows NSIS | `Origa_<version>_x64-setup.exe` | `Origa_x64-setup.exe` |
| Linux AppImage | `Origa_<version>_amd64.AppImage` | `Origa_amd64.AppImage` |
| Linux DEB | `Origa_<version>_amd64.deb` | `Origa_amd64.deb` |
| ~~macOS zip~~ | ~~`Origa_<version>_macos-arm64.zip`~~ | ~~`Origa_macos-arm64.zip`~~ — **removed in PR #308 (2026-07-29)**: macOS distribution moved to Mac App Store only, see ADR-033. |
| Android APK | `origa_<version>.apk` | `origa.apk` |

Desktop uses `Origa_` (capital) to match Tauri's `productName: "Origa"` default
output (verified against the 0.4.0 release assets); Android uses lowercase
`origa` because Gradle's output (`app-universal-release.apk`) carries no
productName and the historical `origa-latest.apk` convention was lowercase. The
versioned file is Tauri's/Gradle's default output (not renamed); the alias is
created via `cp` (not `mv`) so both exist. The previous `Rename artifacts for
latest release` steps (`if: version_type != 'stable'`) are deleted — naming is
now unconditional across all channels.

The release job's `find` glob was updated: the macOS pattern changed from
`-name 'Origa-macos-arm64.zip'` (hyphen) to `-name 'Origa_*macos-arm64.zip'`
(underscore, matches both versioned and alias). Windows/Linux/Android globs
(`*-setup.exe`, `*.AppImage`, `*.deb`, `*.apk`) already matched both names.

### 7. `update-version` receives the full version

All `update-version` call sites (`tauri.yml` build-frontend, `_build-tauri.yml`
four build jobs) now pass `version` (full) instead of `version_base`. Rationale:
a prerelease binary must report its rc suffix so the Tauri updater can order it
correctly. For `v1.0.0-rc1`, `tauri.conf.json` is stamped `1.0.0-rc1`; when
stable `1.0.0` ships, `latest.json.version = 1.0.0` and the updater compares
`1.0.0-rc1 < 1.0.0` (semver pre-release ordering) → offers the stable update to
rc users. For stable, `version == version_base` (no change). For alpha, the
binary now reports `0.4.1-alpha.N` instead of the bare base (more informative;
harmless). `version_base` is retained as a `version` action output for
reference but no longer feeds `update-version`.

### 8. Railway deploy is stable-only

`docker.yml` `deploy-to-railway`: `if: success() && startsWith(github.ref,
'refs/tags/v') && version_type == 'stable'`. `build-and-push` (GHCR image push)
runs on any `v` tag (rc images land in GHCR, no production effect until Railway
redeploys), but Railway production redeploy happens only on stable tags. This
makes the rc-first validation strategy (see Migration) safe for the landing
page: an rc tag does not redeploy production with new fixed-alias URLs while
`/releases/latest` still points at the prior stable (which lacks alias assets).
The `:latest` Docker image tag is likewise gated on stable (`type=raw,value=latest,
enable=${{ version_type == 'stable' }}`); the dead `:edge` tag (master-only,
now that docker.yml no longer runs on master) is removed.

### 9. Landing direct download links

`origa_landing/src/pages/download.rs` replaces the single `GITHUB_RELEASES_URL`
(which linked to the release UI page) with per-platform constants pointing at
`/releases/latest/download/<fixed-alias>`. Clicking starts the file download
directly (GitHub serves these with `Content-Disposition: attachment`); no
`target="_blank"` (direct-download URLs keep the current tab and trigger the
file save dialog). Research finding: `/releases/latest/download/<name>` requires
a **fixed** filename — versioned names break on every release — which is why
the fixed aliases in §6 exist.

## Migration (post-merge, manual — documented in the PR)

The refactor ships behind the normal tag-based pipeline. Because a live 0.4.0
install base depends on the updater, the first post-merge tag is validated with
**rc-first** rather than directly on stable:

1. **(Merge this PR.)** Master now runs full CI; `docker.yml`/`tauri.yml` are
   dormant until a tag is pushed.
2. **Push `v0.4.1-rc1`.** This triggers CD with `version_type=prerelease`:
   - `require-ci-green` verifies the commit's CI is green.
   - Tauri builds produce dual-named artifacts.
   - A prerelease GitHub release is created (`make_latest: false`, marked
     prerelease). `/releases/latest` still points at 0.4.0 (unaffected).
   - `latest.json` is **not** regenerated (prerelease).
   - Railway is **not** redeployed (stable-only). Production landing keeps its
     pre-refactor download links.
   - Validate: release assets include both versioned and alias names for all
     five platforms; the macOS glob matched; no asset accumulation.
3. **Push `v0.4.1` (stable).** This triggers the full stable path:
   - `make_latest: true` → `/releases/latest` now points at 0.4.1.
   - `latest.json` regenerated with 0.4.1 asset URLs → updater offers 0.4.1 to
     0.4.0 users.
   - Railway redeploys landing + UI. The landing's fixed-alias download links
     now resolve (0.4.1 has the alias assets).
4. **Delete the `latest` pseudo-release (cosmetic).** Anytime after step 3:

   ```bash
   gh release delete latest --yes
   git push --delete origin latest
   ```

   This removes the 130 junk assets. It is purely cosmetic — the pseudo-release
   was already excluded from `/releases/latest` resolution by virtue of being
   `prerelease: true`.

### Known window: landing fixed-alias 404 on stable push

On a stable tag push, `docker.yml` (Railway landing redeploy) and `tauri.yml`
(asset upload + release) run in parallel. Sequence: Docker image build (~5 min)
→ Railway redeploy (~2 min) = landing live in ~7 min; Tauri build (~15–20 min)
→ release. The new stable release becomes `make_latest` only after the
`release` job completes. During this window (~10–15 min), the redeployed
landing's fixed-alias URLs may 404 if `/releases/latest` has not yet flipped to
the new release (the prior stable lacks the alias assets only if it predates
this ADR; after the first post-refactor stable, all subsequent stables have
aliases, so the window is only the gap between Railway live and release
`make_latest`). Accepted trade-off: releases are infrequent (weekly+), the
window is short, and users retry. Rejected alternative: cross-workflow ordering
(`docker.yml` waits on `tauri.yml`'s release) — adds coupling and a single
point of failure for two otherwise-independent pipelines.

### Operations: tagging discipline

- **Tag only after CI is green on master.** If a tag is pushed while CI is still
  running, `require-ci-green` fails (fail-closed — no deploy). Recovery: wait
  for CI to complete, then either delete and re-push the tag (`git tag -d vX.Y.Z
  && git push --delete origin vX.Y.Z && git tag vX.Y.Z && git push origin
  vX.Y.Z`) or trigger the CD workflow via `workflow_dispatch` from the Actions
  UI (which bypasses the CI gate as an operator escape-hatch).
- **rc tags must be numbered** (`v1.0.0-rc1`, not `v1.0.0-rc`). A bare `rc`
  falls through to alpha and produces no release.
- **Only `rc` is a recognized prerelease suffix.** `-beta1`, `-alpha2`, etc. →
  alpha → no release (build jobs run, release skips).

## Alternatives Considered

### A1: Keep `workflow_run`, gate CD on a manual promotion label/commit

- **Pros:** No new CI-status-check job; reuses the existing trigger.
- **Cons:** `workflow_run` fires on every master CI success — to make CD opt-in,
  you'd need a label/commit-message convention parsed inside the workflow, which
  is fragile and re-introduces the "CD triggered by CI" coupling this ADR
  removes. Does not address the `latest` pseudo-release or the missing
  prerelease channel.
- **Rejected.**

### A2: Single release per stable tag; no prerelease channel

- **Pros:** Simplest — stable tags release, nothing else does.
- **Cons:** No safe way to validate the release pipeline (artifacts, globs,
  `latest.json`) without risking the live updater install base. The rc-first
  migration strategy depends on a prerelease channel that does not disturb
  `/releases/latest`.
- **Rejected.** Prerelease support is cheap (one regex branch + two conditional
  flags) and pays for itself in the first migration.

### A3: Versioned-only artifact names; landing links to `/releases/latest` (UI page)

- **Pros:** No dual naming; `_build-tauri.yml` stays simpler.
- **Cons:** The landing's download buttons would open the GitHub release UI page
  (extra click, version noise, no direct download). The explicit goal (§9) is
  direct file downloads. Fixed aliases are required for
  `/releases/latest/download/<name>` to resolve.
- **Rejected.**

### A4: Cross-workflow ordering to eliminate the §Migration landing 404 window

- **Pros:** Zero 404 window on stable push.
- **Cons:** Couples `docker.yml` and `tauri.yml` (one must trigger/wait on the
  other). Adds a single point of failure: if the Tauri release job fails, the
  landing never redeploys (or vice versa). Two independent pipelines become one
  fragile chain.
- **Rejected.** The ~10–15 min window on an infrequent event is a better
  trade-off than coupling.

### A5: Remove `version_base` and `version_type` from `_build-tauri.yml` inputs

- **Pros:** Eliminates now-vestigial inputs cleanly.
- **Cons:** Contract-breaking change to the `workflow_call` signature requires
  updating both callers (`ci.yml`, `tauri.yml`) in the same PR, expanding scope
  and risk. The inputs are workflow inputs (not Rust code), so they do not
  trigger the `#[allow(dead_code)]` rule; they are retained with a comment
  explaining they are kept for signature stability.
- **Rejected** for this PR. A follow-up may clean them up once the pipeline is
  proven.

## Consequences

### Positive

- **Production deploys are a deliberate human act** (tag push), not an automatic
  consequence of CI passing on master.
- **CI verifies the full build matrix on master** (Tauri compile, Docker build)
  — a regression on master is caught before it could be tagged, not after.
- **The `latest` pseudo-release stops accumulating** and can be deleted.
- **Prerelease channel** (`v*.*.*-rc*`) enables safe pipeline validation without
  disturbing the stable install base or `/releases/latest`.
- **Direct download links** on the landing page (one click → file download,
  no GitHub UI interstitial).
- **Consistent artifact naming** across all channels (always versioned + alias;
  no more `origa_latest_*` lowercase divergence from `Origa_<version>_*`).
- **No duplicate bare tags** (`0.4.0` alongside `v0.4.0`) — `tag_name:
  github.ref_name` attaches the release to the triggering git tag.
- **CI-gate invariant restored** via `require-ci-green`, replacing the implicit
  `workflow_run.conclusion == 'success'` gate that the trigger removal deleted.

### Negative

- **Master push no longer deploys anything.** Operators must push a tag to ship.
  This is the intended behavior but is a workflow change that requires team
  awareness (documented in the PR and this ADR).
- **CI cost on master increases**: `build-tauri` (4-platform cross-compile) and
  both `docker-build-*` now run on every master push, not just PRs. Accepted —
  this is the explicit goal (§1, problem 5) and the alternative (shipping
  unverified master commits to production) is worse.
- **`version_base` and `version_type` inputs in `_build-tauri.yml` are
  vestigial** (retained for signature stability; see A5). Documented in a
  comment.
- **macOS `CFBundleShortVersionString` for rc** = `1.0.0-rc1` (non-numeric).
  Apple prefers `n.n.n`, but this is a manual `.app` bundle (not a notarized
  `.dmg`), and alpha already carried non-numeric versions (`0.4.1-alpha.N`).
  No regression.
- **Landing fixed-alias 404 window** (~10–15 min) on each stable push while
  Railway redeploy races the Tauri release. Accepted (§Migration).

## Verification

CI/CD workflows cannot be exercised end-to-end locally (they run in GitHub
Actions). Verification is therefore layered:

| Check | Command | Scope |
| --- | --- | --- |
| Workflow syntax/semantics | `actionlint` on all 4 workflows + version action | Local; all pass |
| Landing compiles + lints | `cargo clippy -p origa_landing --all-targets --features ssr -- -D warnings` | Local; 0 warnings |
| Landing format | `cargo fmt --check -p origa_landing` | Local; clean |
| Landing tests | `cargo test -p origa_landing --features ssr` | Local; 30+ tests pass (security_headers, seo_meta, sitemap, build_config) |
| `require-ci-green` query | `gh run list --workflow ci.yml --commit <full-sha> --status success` | Empirically verified: returns run ID for green commits, empty string otherwise |
| **End-to-end (rc-first)** | Push `v0.4.1-rc1` → inspect prerelease release assets | Post-merge; the migration itself is the final E2E test |
| **Manual latest.json check** | Before pushing `v0.4.1`, dry-run the `generate-latest-json` jq locally with `VERSION=0.4.1 GIT_TAG=v0.4.1` and confirm the URL `releases/download/v0.4.1/Origa_0.4.1_x64-setup.exe` resolves against the asset name the release will publish | rc-first does NOT exercise this (generate-latest-json is stable-only), so the URL tag/asset-segment consistency must be verified by hand |
| **End-to-end (stable)** | Push `v0.4.1` → verify `/releases/latest` flips, `latest.json` updated, updater offers to 0.4.0 | Post-merge |

## References

- ADR-009: Tauri config parameterization (`tauri.conf.json` `version` field consumed by bundlers)
- ADR-024: Build-script env var empty-handling pattern (version-stamping conventions)
- Tauri updater `latest.json` format: <https://v2.tauri.app/plugin/updater/>
- GitHub `/releases/latest/download/<asset>` behavior: requires a fixed filename; versioned names break the URL on each release
- `softprops/action-gh-release` v2 (current `@v2` resolves to v2.6.2; v2.5.0 had a prerelease-detection bug fixed in v2.5.1+)
- `tauri/tauri.conf.json:56` — updater endpoint `/releases/latest/download/latest.json` (unchanged by this ADR)
- Existing stable release (0.4.0) assets verified via `gh api repos/yurvon-screamo/origa/releases/latest`

## Addendum (2026-07-12): Single artifact naming (alias-only)

### What changed

§6 "Dual artifact naming" is replaced with **single artifact naming** for
desktop platforms (Windows NSIS, Linux AppImage/DEB, macOS zip). Each release
now ships only the fixed-name alias — the versioned copy produced by Tauri's
default output is renamed (not duplicated) via `mv` in `_build-tauri.yml`.

| Platform | Sole asset in release |
| --- | --- |
| Windows NSIS | `Origa_x64-setup.exe` |
| Linux AppImage | `Origa_amd64.AppImage` |
| Linux DEB | `Origa_amd64.deb` |
| ~~macOS zip~~ | ~~`Origa_macos-arm64.zip`~~ — removed in PR #308, see ADR-033 |
| Android APK | `origa.apk` (already single-naming) |

### Rationale

The versioned filename (`Origa_0.4.1_x64-setup.exe`) was redundant with the
release tag (`v0.4.1`) — both identify the version. The dual naming forced
users to scan a release asset list that contained two near-identical entries
per platform, with no guidance on which to pick. Switching to alias-only
resolves the user complaint ("asset list cluttered with duplicates") while
preserving every functional contract that dual naming served:

- §9 (landing direct download) — `/releases/latest/download/Origa_x64-setup.exe`
  still resolves because the alias is still published.
- `latest.json` URLs (`tauri.yml` `generate-latest-json`) now point at the
  alias asset inside the git tag, e.g.
  `releases/download/v0.4.2/Origa_x64-setup.exe`.
- Tauri updater signature validity is preserved — the `.sig` file is a
  Minisign signature over the **content** of the bundle, not its filename, so
  the versioned → alias rename in `_build-tauri.yml` does not invalidate it.
  The `.sig` is renamed alongside the bundle.

### What did NOT change

- §9 (landing direct download links) — alias filenames are still the contract
  with `origa_landing/src/pages/download.rs`.
- §5 (`latest.json` is stable-only) — prereleases still do not refresh the
  updater manifest.
- The macOS bundle format (manual `.app` zip, not `.app.tar.gz`) — updater
  support for macOS remains a known limitation; the signature output is empty
  for macOS and `latest.json` does not include a `darwin-*` platform entry.

### Migration

Historical releases (`v0.4.0`, `v0.4.1`) keep their dual assets — only
`v0.4.2+` uses single naming. There is no backward-compat break: the alias
filenames are unchanged from before, only the versioned duplicates are no
longer produced. The first post-addendum tag (rc-first per §Migration)
validates that the rename produces a single asset per platform and that
`latest.json` URLs resolve against it.
