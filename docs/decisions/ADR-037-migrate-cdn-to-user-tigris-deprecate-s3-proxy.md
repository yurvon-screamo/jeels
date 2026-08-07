# ADR-037: Migrate CDN bucket to user-owned Tigris, deprecate Railway s3-proxy

## Status

**Cutover in progress** (2026-08-02 — infrastructure built, 2026-08-03 —
DNS record updated in Aeza DB, waiting on Aeza NS propagation up to 2 days).
Production unaffected: `s3.origa.uwuwu.net` continues routing through Railway
s3-proxy until NS propagates; Tigris bucket serves shadow-migration traffic
on direct `origa-cdn.t3.storageapi.dev` URL in parallel.

## Date

2026-08-02 (Slice 0–1), 2026-08-03 (Slice 2 cutover initiated).

## Context

Origa's static content (ML models, dictionaries, audio, fonts, kanji SVG, JSON
catalogs) is served from a single hostname `s3.origa.uwuwu.net`. Until this ADR
that hostname was a CNAME to Railway (`sltxm1ip.up.railway.app`) terminating at
an `s3-proxy` service (Docker image `pottava/s3-proxy`) that proxied every
request to a private Tigris bucket `adaptable-foodbox-ucep7wx` (Railway name
`origa-content`) using Railway-scoped credentials.

The bill for Railway spiked. Root cause analysis on 2026-08-02 produced the
following 30-day network metrics (Railway CLI `metrics --network --since 30d`):

| Service | egress max | ingress max |
| --- | --- | --- |
| `s3-proxy` | **10.5 GB** | 13.5 GB |
| `landing` | 0.1 MB | 0.01 MB |
| `origa-trailbase` | 0.08 MB | 0.05 MB |

`s3-proxy` was the only meaningful egress source. HTTP log inspection showed
the heavy hitters: `ndlocr/{parseq-30,parseq-50,parseq-100,deim}.onnx` and
`whisper/onnx/{decoder,encoder}_model.onnx` (multi-second responses, 30–120 MB
each), plus the release-updated JSON catalogs under `dictionary/`, `grammar/`,
`phrases/`, `pitch/`.

Tigris (the S3-compatible backend Railway uses under the brand "T3 Storage")
advertises **Zero Egress Fees** and a global anycast CDN. The `s3-proxy` hop on
Railway added no value: it forwarded bytes from one Tigris account (Railway's
internal) to end users while billing every byte as Railway egress, even though
the same bytes would be free if served directly from a user-owned Tigris bucket.

### Hypothesis: open the existing Railway-managed bucket publicly

Before deciding to migrate, we verified empirically whether the existing
`adaptable-foodbox-ucep7wx` bucket could simply be made public and served
directly, avoiding any data migration:

- **Via S3 API with Railway-scoped credentials** (`tid_qYczVK...`, profile
  `origa`): every `PutBucketAcl`, `PutBucketCors`, `PutPublicAccessBlock`,
  `DeletePublicAccessBlock` returns `SignatureDoesNotMatch` or `AccessDenied`.
  The credentials are scoped to data-plane operations only.
- **Via S3 API with Railway workspace credentials** (`tid_SAtTG...`, profile
  `default` — railway-issued legacy key with broader scope):
  `head_bucket` on `adaptable-foodbox-ucep7wx` returns `403 Forbidden` even
  though `list_buckets` sees the bucket. `PutBucketAcl` etc. all return
  `AccessDenied` or a misleading `BucketAlreadyExists`.
- **Via Tigris-native `t3` CLI** (`tigris` v3.6.1) with the same credentials:
  `t3 buckets get adaptable-foodbox-ucep7wx` works (read), but
  `t3 buckets set ... --access public`, `set-cors`, `set --custom-domain` all
  return `Forbidden`. `t3 whoami` confirms `organizationId:
  railway_$nsID$_e97da63b-...` — Railway's Tigris organization, on which
  Railway grants only Observer / data-plane roles to user-issued keys.

**Conclusion:** Railway intentionally blocks configuration-plane access
(access level, CORS, custom domain, IAM) on its managed Tigris buckets.
Repurposing the existing bucket without migration is impossible.

### A new user-owned Tigris account is required

A fresh Tigris account (sign-up on tigrisdata.com via Fly.io SSO, free tier,
organization `yurvon-screamo`) gives owner-level credentials. On a throwaway
test bucket `origa-public-test-uwuwu` (created and deleted on 2026-08-02) we
verified all required primitives work end-to-end:

| Operation | Result |
| --- | --- |
| `create_bucket` | OK |
| `put_bucket_acl ACL=public-read` | OK |
| `put_bucket_cors` with `ExposeHeaders` | OK |
| anon GET **without** Origin header | HTTP 200 |
| anon HEAD **without** Origin header | HTTP 200 |
| anon LIST (`?list-type=2`) | **HTTP 403** — inventory not exposed |
| Range request (`Range: bytes=0-4`) | HTTP 206, `accept-ranges: bytes` |
| `PutBucketPolicy` | `NotImplemented` (Tigris does not implement S3 bucket policies — see §4) |

## Decision

Migrate all 600 683 objects (3.7 GB) from `adaptable-foodbox-ucep7wx`
(Railway-managed) to a new user-owned bucket `origa-cdn` under organization
`yurvon-screamo`. After migration, deprecate and delete the `s3-proxy` Railway
service. The hostname `s3.origa.uwuwu.net` is preserved across the cutover —
only its DNS target and TLS-terminating edge change.

### 1. New topology

| Layer | Before | After |
| --- | --- | --- |
| Client-facing URL | `https://s3.origa.uwuwu.net` | `https://s3.origa.uwuwu.net` (unchanged) |
| CNAME target | `sltxm1ip.up.railway.app` | `origa-cdn.t3.tigrisbucket.io` (DNS-only) |
| Edge / TLS termination | Railway Hikari (behind Cloudflare front-end Railway controls) | Tigris global anycast + automatic TLS issuance on CNAME validation |
| Bucket | `adaptable-foodbox-ucep7wx` (Railway-managed Tigris) | `origa-cdn` (user-owned Tigris) |
| Deploy credentials | Railway-scoped `tid_qYczVK...` (profile `origa`) | User-org scoped `tid_eHRHguc...` (profile `origa-cdn`, Editor role on `origa-cdn` only) |
| Public access control | impossible (Railway blocks) | `t3 buckets set --access public --disable-directory-listing=true` |
| CORS injection | `s3-proxy` response headers (env vars) | `t3 buckets set-cors ... --override` (bucket-native) |

### 2. The `s3.origa.uwuwu.net` hostname is invariant

No shipped Origa artifact references the bucket name or the Railway hostname
directly. Verified consumers of the literal `s3.origa.uwuwu.net`:

- `build_defaults.rs:28` — `DEFAULT_CDN` constant.
- `tauri/tauri.conf.json:25` — CSP `connect-src`/`img-src`/`media-src`/`font-src`.
- `origa_ui/index.html:17` — `<link rel="preload" href="https://s3.origa.uwuwu.net/fonts/...">`.
- `origa_landing/src/content/{ru,en}.rs:292,319` — privacy/description copy.
- CI variables `ORIGA_CDN_URI_PREFIX` / `ORIGA_BASE_URI` (resolved to
  `s3.origa.uwuwu.net` at deploy time).
- `cdn_url()` in `origa_ui/src/core/config.rs:56-83` — concatenation only,
  no URL signing.

Code, CSP, shipped desktop binaries, and CI pipelines are untouched. The
cutover is a pure DNS operation from the client's perspective.

### 3. Migration tool: `scripts/migrate_cdn_bucket.py` + Tigris server-side shadow migration

The migration uses a **hybrid approach**: a one-time client-side bulk copy
script for the bulk of the data, plus Tigris-native server-side shadow
migration as both a fallback for any cache-misses and a verification
backstop. Empirically the Tigris user-account endpoint throttles
`list_objects_v2` erratically on buckets >100k objects (60 s+ for a single
page), and `t3 objects list` hangs on buckets >100k; per-key HEAD became
the resume strategy of choice.

**`scripts/migrate_cdn_bucket.py`** — client-side bulk copy with the following design choices:

- **Pure boto3**, no shell-out to `aws` CLI. The source bucket uses CJK kanji
  as object keys (`一.svg`, `丁.svg`, …) — the existing `run_aws_raw` /
  `pwsh -Command aws` pattern in `scripts/_cdn_s3.py:62` carries a shell-
  injection denylist (`_UNSAFE_KEY_CHARS:49`) precisely because of this; the
  migration routes every key through boto3 and inherits the safety for free.
- **Resume by key + size**, not mtime. mtime is unreliable cross-account
  (clock skew, timezone, destination `LastModified` = upload time). The
  destination bucket is listed once at start; any source key whose size
  matches an existing destination key is skipped.
- **Migration-specific `TransferConfig`**: 16 KB multipart threshold (T3
  single-PUT limit is "~24 KB", per `scripts/_cdn_s3.py:263`), per-object
  inner concurrency 4. This intentionally diverges from the steady-state
  `max_concurrency=1` in `_cdn_s3.py:316` (which exists for T3 rate-limit
  headroom during routine deploys); a one-time bulk migration trades
  rate-limit headroom for wall-clock time.
- **Outer concurrency 16** via `ThreadPoolExecutor` for the many small
  objects (kanji SVG, audio, JSON); large multipart uploads (whisper decoder
  ~118 MB) proceed in parallel.
- **Metadata passthrough**: `head_object` source → `ExtraArgs` destination
  for `CacheControl`, `ContentType`, `ContentEncoding`, `ContentLanguage`,
  `ContentDisposition`. Most objects have no `ContentEncoding` (verified:
  `scripts/_cdn_s3.py:355` only sets `CacheControl` + `ContentType`); any
  object that did have it is copied as-is.
- **Verification at the end**: count match, byte-sum match, SHA256 spot-check
  on a sample covering each content type (model, dictionary chunk, grammar,
  pitch, phrase index, opus audio, woff2 font), and byte-for-byte SHA256
  equality of `manifest.json` (content-addressed, must match).

Source/dest listings are cached to
`scripts/.cdn_migration_cache/{source,dest}_index.json` (gitignored) so a
partial-failure rerun does not re-paginate the 600k-object source (which
takes ~6 min by itself).

**Verify-after-upload**: every uploaded object is immediately followed by
`head_object` on destination (with retry/backoff) to confirm it actually
materialised. `upload_fileobj` returning without exception does not guarantee
persistence — during an early migration attempt that was killed mid-flight,
the script's `up=` counter advanced by ~345k but the destination listing
showed only ~95k current objects. The verify-after-upload step ensures the
counter is honest: any silent failure lands the key in `failed[]` rather
than incrementing `up`.

**Tigris shadow migration** (`t3 buckets set-migration` + `t3 buckets migrate`)
is layered on top: the destination bucket is configured to lazily pull any
missing object from the Railway-managed source on first access, and an active
background `t3 buckets migrate` schedules server-side copies for all
unmigrated objects without client bandwidth. This means DNS cutover is
**safe to perform before client-side migration completes** — cache misses
on the new endpoint pull from Railway through Tigris, then cache forever.

### 3a. Aeza NS propagation caveat

Aeza authoritative NS (`ns1-4.aeza-dns.net`) updates from the database
slowly — observed 30+ minutes with no propagation after a record update
in API v2; the operator confirms **2-day windows are normal**. The DNS
record is updated in the Aeza database immediately, but the NS infrastructure
replicates on its own schedule.

This affects the cutover sequence:

- The Aeza DB record shows the new CNAME within seconds.
- Tigris `set --custom-domain` will keep failing with `Failed to verify
  CNAME for bucket domain` until NS propagates and the public DNS resolves
  the new target.
- During the propagation window, production continues serving through the
  old Railway s3-proxy chain — no impact.

See the runbook's watcher snippet for monitoring propagation.

### 4. Public access without `ListBucket` exposure (Tigris deviation from S3)

On AWS S3, bucket-level ACL `public-read` grants both `GetObject` and
`ListBucket` to anonymous principals — the latter exposing the full key
inventory. The original review of this migration plan recommended a bucket
policy with `s3:GetObject` only (no `ListBucket`) as defence in depth.

**Tigris does not implement S3 bucket policies** — `PutBucketPolicy` returns
`NotImplemented`. However, Tigris's bucket-level ACL `public-read` empirically
grants anonymous `GetObject` while leaving anonymous `ListBucket` denied
(verified 2026-08-02 on the throwaway test bucket: anon
`GET /test.txt` → 200, anon `GET /?list-type=2` → 403). This is a deviation
from S3 semantics in the safer direction.

This deviation is **observed behaviour, not a documented contract**. If
Tigris ever "fixes" S3 conformance here, anonymous `ListBucket` would open
and the full 600k-object inventory would leak. Mitigation:

- ADR records the verification date and the deviation.
- The runbook includes a periodic regression smoke
  (`curl -s -o /dev/null -w "%{http_code}" "https://origa-cdn.t3.storageapi.dev/?list-type=2"`
  expected `403`).

Bucket creation in this account used
`t3 buckets create origa-cdn --public` (default directory-listing off),
equivalent to `--access public --disable-directory-listing`.

### 5. CORS configuration

`scripts/_cdn_s3.py` per-object `Cache-Control` is preserved through the
migration (`immutable` for ML models / kanji / dictionaries, `must-revalidate`
for release-updated JSON, `no-cache` for `manifest.json`). The
`must-revalidate` policy in particular protects against the CDN edge-cache
poisoning regression documented in PR #182 and ADR-016.

Bucket-level CORS (set via `t3 buckets set-cors origa-cdn --override`):

| Field | Value |
| --- | --- |
| `AllowedOrigins` | `*` |
| `AllowedMethods` | `GET`, `HEAD` |
| `AllowedHeaders` | `Range`, `If-None-Match`, `If-Match`, `Content-Length`, `Content-Type`, `ETag`, `Last-Modified`, `Cache-Control` |
| `ExposeHeaders` | `Content-Length`, `Content-Range`, `ETag`, `Accept-Ranges`, `Last-Modified` |
| `MaxAgeSeconds` | `3600` |

`ExposeHeaders` is critical for the model-load progress bar in
`origa_ui/src/loaders/model_cache.rs:167,279-286`, which reads the
cross-origin `content-length` response header to drive the progress callback.
Without `ExposeHeaders`, the browser hides `content-length` from JavaScript
and the bar regresses to indeterminate for the 30–120 MB model downloads.

### 6. Bug A: Railway Hikari on-the-fly opus gzip — workaround stays inert

`origa_ui/src/repository/cdn_provider.rs:386-391,477-502` documents "Bug A":
the Railway Hikari reverse proxy applied `Content-Encoding: gzip` to `.opus`
responses on the fly, in some cases producing a *larger* body (3145 > 3139
bytes — negative compression). The entire blob-URL architecture
(`prefetch_blob_url`, `resolve_audio_url -> Option<String>`, and the
regression test `resolve_audio_url_contract_returns_option`) exists as a
workaround for this behaviour.

Objects in the bucket do **not** carry `Content-Encoding`
(`scripts/_cdn_s3.py:355` ExtraArgs = only `CacheControl` + `ContentType`).
The migration copies metadata as-is and therefore produces no
`Content-Encoding` on the destination either. Served through Tigris, opus
flows as-is; `fetch()` decompresses nothing because there is nothing to
decompress; the blob-URL path continues to work, but the workaround becomes
**inert**.

The workaround code is **intentionally retained**. Removing it would create a
latent regression trap: any future transport-compressing CDN edge or proxy
re-introduces the same `ERR_CONTENT_DECODING_FAILED` failure mode that Bug A
documented. The defence-in-depth value of keeping the indirection is higher
than the readability cost of dead-looking code, given that Bug A's
provenance is now in this ADR.

### 7. ETag change implication

AWS S3 ETags for multipart-uploaded objects are computed from the multipart
upload structure, not the object content. The migration re-uploads every
object via multipart (16 KB threshold), so ETags change format on
destination even for byte-identical content. The `must-revalidate` policy
(`Cache-Control: public, max-age=300, must-revalidate`) means the first
conditional `GET` with `If-None-Match` against a stale client-held ETag
returns `200` instead of `304` — a one-shot re-download of those JSON
catalogs. Subsequent conditional requests against the new ETags resume `304`
behaviour. This is a one-time effect at cutover, not a steady-state cost.

## Alternatives considered

### A1. Cloudflare proxy in front of Railway s3-proxy

Toggle `s3.origa.uwuwu.net` to Cloudflare proxied (orange cloud), let
Cloudflare cache edge responses. Rejected: ADR-021 removed the `uwuwu.net`
zone from Cloudflare authoritative NS due to Yandex indexing failure and
bot-management opacity. Re-introducing Cloudflare for `s3.origa` would
either require re-delegating the zone (reversing ADR-021) or a Cloudflare
CNAME-setup partial (Business plan or third-party). Either path reintroduces
the complexity this ADR removes.

### A2. Cloudflare Worker → Railway bucket

A Worker signs SigV4 requests with Railway-scoped credentials and proxies to
the private bucket; Cloudflare caches edge responses. Rejected: it
re-introduces a proxy layer (one was removed, one was added), exposes scoped
credentials in Worker env, and adds Worker code as a failure surface. The
net stack simplification is zero.

### A3. Cloudflare R2

Migrate to R2 instead of Tigris. Rejected: R2 has free egress too, but
requires re-targeting `scripts/_cdn_s3.py` at the R2 endpoint, learning a
new dashboard, and adds a second object-storage vendor. Tigris already backs
the existing data and was verified end-to-end on test buckets; switching
backends adds cost without benefit.

### A4. Presigned URLs

Generate per-object presigned URLs from a backend. Rejected: incompatible
with immutable ML models that clients fetch directly without backend
mediation, and with the WASM frontend that constructs URLs client-side
without signing.

### A5. Request owner-level Tigris credentials from Railway support

Theoretically possible. Rejected: depends on a support response, no
documented policy that Railway grants this, and even if granted, the
underlying coupling to Railway's internal Tigris account would remain (any
Railway-side change could revoke access). A user-owned account is durable.

## Consequences

### Positive

- **Railway egress drops to near zero.** Tigris Zero Egress means the only
  Railway egress left is `landing` (~0.1 MB/30d) and `origa-trailbase`
  (~0.08 MB/30d) — together ~0.2 MB, against the previous `s3-proxy` peak of
  10.5 GB over the same window.
- **Full bucket control via `t3` CLI**: access level, CORS, custom domain,
  IAM, lifecycle, snapshots — all first-class operations on the user account.
- **Simpler stack**: one less service (`s3-proxy`), one less proxy hop, one
  fewer place where response transforms (Bug A's gzip) can silently intrude.
- **Global anycast**: Tigris serves reads from the nearest region
  automatically (verified: `X-Tigris-Served-From: fra` from a European
  client). No CDN cache rules to misconfigure.
- **IPv6 may improve**: ADR-021 noted a known non-regression where
  `s3.origa.uwuwu.net` as a CNAME to Railway returned AAAA SERVFAIL through
  Google Public DNS (Railway NS protocol bug). Tigris may serve AAAA
  correctly — the post-cutover runbook verifies this.

### Negative

- **One-time migration cost**: ~600k GET requests on source + ~615k PUT
  requests (small files single-PUT, large models multipart) on destination.
  Tigris per-request pricing applies even with zero egress — the migration
  itself is not free, just cheap.
- **Per-request pricing in steady state**: every anonymous GET on a
  cache-missing object is billed by Tigris at the request rate, in addition
  to the storage cost. Egress itself remains free.
- **New vendor relationship**: a Tigris account (`yurvon-screamo`,
  Fly.io SSO) now holds production CDN data. Credentials, billing alerts,
  and access reviews extend to this account.
- **`manifest.json` cache invariant unchanged**: clients detect CDN updates
  via the manifest's SHA hashes; the migration preserves manifest bytes
  exactly (verified by SHA256 equality in §3), so no client-side cache
  invalidation logic is affected.

## Security

- **Anonymous read surface**: bucket ACL `public-read` exposes `GetObject`
  only, not `ListBucket` (§4). All served objects are public catalog content
  (dictionaries, ML models, audio, fonts) — no user data, no PII, no secrets.
  The landing copy already describes these as "public catalog content"
  (`origa_landing/src/content/{ru,en}.rs:292`).
- **Deploy credentials**: scoped IAM access key `tid_eHRHguc...` with Editor
  role on `origa-cdn` only (no `PutBucket*`, no admin on the organization).
  Stored in `~/.aws/credentials [origa-cdn]` on the operator's machine.
  If `scripts/deploy_cdn.py` ever moves to CI, these scoped credentials
  become a GitHub secret — never the root account key.
- **Backup of pre-cutover state**: `docs/backups/railway-s3-proxy-envs.2026-08-02.txt`
  records the `s3-proxy` env vars (secret redacted; pointer to
  `~/.aws/credentials [origa]` profile and Railway Dashboard). The
  `s3.origa.uwuwu.net` rollback path is to re-create the service on Railway
  and flip the CNAME back (see runbook).

## Verification

Post-migration verification is performed by `scripts/migrate_cdn_bucket.py`
automatically; post-cutover verification is in
`docs/runbooks/migrate-cdn-to-user-tigris.md`. The empirical findings that
this ADR depends on (Tigris ACL behaviour, CORS, custom-domain flow) were
verified on 2026-08-02 against throwaway test buckets and recorded above.

Final migration numbers (object count, byte sum, wall-clock time, request
count, post-cutover Railway egress over 30 days) will be appended to this
section once the cutover is complete and the post-cutover observation window
has elapsed.

## References

- ADR-007 — landing DNS indexing fix; original A-record workaround.
- ADR-016 — cache-control for `browserconfig.xml` and `llms.txt`; PR #182
  CDN edge-poisoning precedent that the `must-revalidate` policy mitigates.
- ADR-021 — revert Cloudflare proxy; records `s3.origa.uwuwu.net` AAAA
  SERVFAIL as a "known non-regression" (this ADR may close it).
- ADR-036 — Sentry integration; provides the post-cutover early-warning
  surface for `OrigaError::NetworkError` spikes.
- `scripts/migrate_cdn_bucket.py` — the one-time migration tool.
- `scripts/_cdn_s3.py` — steady-state CDN deploy transport (unchanged except
  for the eventual profile/bucket name swap in `S3_BUCKET` / `S3_PROFILE`).
- `docs/runbooks/migrate-cdn-to-user-tigris.md` — execution runbook + rollback.
- `docs/backups/railway-s3-proxy-envs.2026-08-02.txt` — pre-cutover backup.
