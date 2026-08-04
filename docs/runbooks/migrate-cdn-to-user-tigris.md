# Migrate CDN bucket to user-owned Tigris, deprecate Railway s3-proxy

**Status:** CUTOVER IN PROGRESS — DNS record updated in Aeza DB (2026-08-03 14:51 UTC), **waiting on Aeza authoritative NS propagation** (up to 2 days, normal for Aeza). Production unaffected: `s3.origa.uwuwu.net` still routes through Railway s3-proxy while NS propagates.
**Date:** 2026-08-02 (Slice 0–1), 2026-08-03 (Slice 2 cutover initiated).
**ADR:** [ADR-037](../decisions/ADR-037-migrate-cdn-to-user-tigris-deprecate-s3-proxy.md).

## ⚠️ Aeza NS propagation: 24-48 h window

Aeza authoritative NS (`ns1-4.aeza-dns.net`) sync from database slowly — **up to 2 days** (confirmed by operator). Web UI shows the new record immediately; the NS infrastructure does not. Any cutover depending on DNS must budget 24-48 h propagation window — `t3 buckets set --custom-domain` will fail with `Failed to verify CNAME for bucket domain` until NS propagates. Monitor (bypass Windows DnsClient cache):

```powershell
Clear-DnsClientCache
nslookup -type=CNAME s3.origa.uwuwu.net ns1.aeza-dns.net
```

`ns1.aeza-dns.net` is authoritative — its answer is the current zone state. When it returns `origa-cdn.t3.tigrisbucket.io`, NS propagation is complete; Tigris verification will succeed.

## Prerequisites

- `t3` CLI installed (`npm i -g @tigrisdata/cli`) and logged in (`t3 login`) under the user-owned Tigris account (`yurvon-screamo` org).
- `~/.aws/credentials [origa-cdn]` — scoped IAM key for `origa-cdn` (Editor role, no `PutBucket*`).
- Operator has Aeza API v2 Bearer token (settings → API keys).

## Slice 0 — Provision destination bucket (DONE 2026-08-02)

```bash
t3 buckets create origa-cdn --public -l global -t STANDARD
t3 access-keys create origa-cdn-deploy   # note returned id+secret → ~/.aws/credentials [origa-cdn]
t3 access-keys assign <key-id> --bucket origa-cdn --role Editor
t3 buckets set-cors origa-cdn \
  -o '*' -m GET,HEAD \
  --headers 'Range,If-None-Match,If-Match,Content-Length,Content-Type,ETag,Last-Modified,Cache-Control' \
  --expose-headers 'Content-Length,Content-Range,ETag,Accept-Ranges,Last-Modified' \
  --max-age 3600 --override
```

## Slice 1 — Migrate data (DONE)

Tigris server-side migration copies everything without client bandwidth:

```bash
# Configure shadow migration (source = Railway-scoped creds, read-only)
t3 buckets set-migration origa-cdn \
  -b adaptable-foodbox-ucep7wx -e https://t3.storageapi.dev -r auto \
  --key tid_qYczVK... --secret tsec_95XVPG...

# Active bulk copy (Tigris-internal, schedules server-side copy for unmigrated objects)
t3 buckets migrate origa-cdn -y
```

Both stay configured: shadow migration is a safety net for cache-miss lazy pull; `t3 buckets migrate` actively backfills. They can run concurrently — server-side de-duplicates.

Verification: `t3 buckets get origa-cdn` reports `All Versions Count` approaching 600 683. Spot-check via direct HEAD on the public URL:

```bash
curl -sI https://origa-cdn.t3.storageapi.dev/manifest.json         # 200, Cache-Control: no-cache
curl -sI https://origa-cdn.t3.storageapi.dev/whisper/onnx/decoder_model.onnx  # 200, immutable
curl -s -o /dev/null -w "%{http_code}" "https://origa-cdn.t3.storageapi.dev/?list-type=2"  # 403 (listing denied)
```

## Slice 2 — Cutover

Order matters: DNS must move first, then Tigris `set --custom-domain` (Tigris verifies CNAME → issues TLS).

### 2.1 Pre-switch smoke (against direct Tigris URL, no DNS change yet)

```powershell
$env:ORIGA_CDN_BASE_URL = "https://origa-cdn.t3.storageapi.dev"
# Plus required compile-time vars (ORIGA_VERSION, etc.) — see AGENTS.md "Среда разработки"
# Run dev build (tauri / trunk) and exercise:
#   - ML model load (progress bar visible — content-length exposed via CORS ExposeHeaders)
#   - Audio playback (opus, through blob: URL — Bug A workaround inert, see ADR-037 §6)
#   - Font preload (woff2)
#   - Dictionary / grammar / phrases JSON fetches
#   - 2nd conditional GET on a must-revalidate JSON → expect 304 (new ETags after first 200, see ADR-037 §7)
```

### 2.2 Switch DNS (Aeza API v2)

Aeza API v1 `/api/services/<id>` PUT is read-only for DNS records despite the old wiki. Use v2:

```bash
# Find domain ID (not service ID) — visible in my.aeza.net → services/<service_id>/ns → network tab
DOMAIN_ID=2776   # for uwuwu.net (service ID 1095525)

# Find s3.origa CNAME record ID
curl -s "https://my.aeza.net/api/v2/domains/$DOMAIN_ID/records?limit=200" \
  -H "authorization: Bearer $AEZA_TOKEN" | jq '.items[] | select(.name=="s3.origa")'

# PATCH the record
curl -s -X PATCH "https://my.aeza.net/api/v2/domains/$DOMAIN_ID/records/<record_id>" \
  -H "authorization: Bearer $AEZA_TOKEN" -H "content-type: application/json" \
  --data-raw '{"type":"CNAME","name":"s3.origa","content":"origa-cdn.t3.tigrisbucket.io","ttl":300,"priority":null,"weight":null,"port":null,"isEnabled":true,"note":null}'
```

| Name | Type | Was | Now | TTL |
| --- | --- | --- | --- | --- |
| `s3.origa` | CNAME | `sltxm1ip.up.railway.app` | `origa-cdn.t3.tigrisbucket.io` | 300 |

**Do NOT toggle Cloudflare proxy on.** Tigris must terminate TLS itself to issue the certificate. Leave `_railway-verify.s3.origa` TXT in place until Slice 3 — rollback anchor.

### 2.3 Register custom domain in Tigris (after NS propagation)

```bash
t3 buckets set origa-cdn --custom-domain s3.origa.uwuwu.net
# Fails with "Failed to verify CNAME for bucket domain" if NS not propagated — wait and retry
```

### 2.4 Post-switch smoke (against production hostname)

```powershell
$env:ORIGA_CDN_BASE_URL = "https://s3.origa.uwuwu.net"
# Same smoke checklist as 2.1
```

DNS-level verification:

```bash
curl -s "https://dns.google/resolve?name=s3.origa.uwuwu.net&type=A"
# Expected: Answer with origa-cdn.t3.tigrisbucket.io / Tigris anycast IPs
curl -s "https://dns.google/resolve?name=s3.origa.uwuwu.net&type=AAAA"
# Either clean NOERROR (Tigris serves AAAA — ADR-021 non-regression closed) or SERVFAIL (persists)
curl -sI https://s3.origa.uwuwu.net/manifest.json | findstr /i server
# Expected: server: Tigris OS (was: Railway Hikari)
```

## Slice 3 — Cleanup (24-48 h post-cutover)

Only after stable production traffic through Tigris. Monitor:

- **Tigris dashboard** (`console.tigris.com`) — request volume, error rate, egress (non-zero, free).
- **Sentry** — `OrigaError::NetworkError` / CDN-fetch-failure spike (per ADR-036). Any spike > baseline means a CDN regression.
- **Railway metrics** — `s3-proxy` egress should drop to ~0 (only scanner probe traffic).

Cleanup actions:

```bash
railway service delete -s a3f10cf6-2d4f-42cf-a176-8ebb4253d734 -e 2670e319-c511-49cf-8bc8-0195e579ddbf -y
# The service is dashboard-only — no railway.json / railway.toml in repo, CI cannot resurrect it.
```

Remove Railway verification TXT at Aeza (`_railway-verify.s3.origa`). Optional: decommission old Railway bucket `adaptable-foodbox-ucep7wx` (Railway name `origa-content`) — `railway bucket delete -b origa-content -y` — or keep as read-only backup (~$0.02/GB/month).

Finalize: append post-cutover numbers (Railway egress 30-day, Tigris request count) to ADR-037 §Verification.

## Known regressions (post-cutover, accepted)

1. **First-load cache miss on every client for `must-revalidate` JSON** — one-shot `200` instead of `304` because ETags changed format under multipart re-upload (ADR-037 §7). Subsequent conditional GETs resume `304`.
2. **Per-request Tigris billing in steady state** — every anonymous GET on a cache-missing object is billed at Tigris request rates. Egress free; requests are not.
3. **Operator-owned Tigris account is now in the trust boundary.**

## Rollback

### DNS rollback

At Aeza, revert the `s3.origa` CNAME back to `sltxm1ip.up.railway.app`. Within DNS TTL (5 min) traffic returns to Railway s3-proxy. **Caveat**: Railway may have released the custom-domain binding for `s3-proxy` during the cutover window. If `https://s3.origa.uwuwu.net` returns Railway's "domain not bound" page after the CNAME revert, re-create the binding in Railway Dashboard → `origa-appuru` → `s3-proxy` → Settings → Networking → Custom Domain, and re-add the `_railway-verify.s3.origa` TXT at Aeza.

### s3-proxy already deleted (Slice 3)

Re-create from `pottava/s3-proxy` image on Railway with the env vars snapshot from before deletion (`AWS_ACCESS_KEY_ID`, `AWS_S3_BUCKET=adaptable-foodbox-ucep7wx`, `AWS_API_ENDPOINT=https://t3.storageapi.dev`, `AWS_SECRET_ACCESS_KEY` from `~/.aws/credentials [origa]` or Railway Dashboard, plus the three `CORS_ALLOW_*` headers). Re-bind the custom domain per the caveat above.

### Tigris bucket warm fallback

`origa-cdn` can be left in place read-only as a fast fallback — the next cutover attempt is just a DNS flip. To decommission fully: `t3 buckets delete origa-cdn`.

## Plan B — if bucket-level ACL `public-read` proves insufficient in production

The migration's empirical verification (2026-08-02) showed Tigris bucket-level `public-read` grants anonymous `GetObject` while denying `ListBucket`. This is observed behaviour, not a documented Tigris contract. If a future Tigris change closes the S3-conformance gap the other way, apply per-object ACL via boto3:

```python
import boto3
from botocore.client import Config

c = boto3.Session(profile_name="origa-cdn").client(
    "s3", endpoint_url="https://t3.storageapi.dev",
    config=Config(signature_version="s3v4", s3={"addressing_style": "virtual"}),
    region_name="auto",
)
paginator = c.get_paginator("list_objects_v2")
for page in paginator.paginate(Bucket="origa-cdn"):
    for obj in page.get("Contents", []):
        c.put_object_acl(Bucket="origa-cdn", Key=obj["Key"], ACL="public-read")
```

## Periodic regression smoke

```bash
# Anonymous LIST must remain denied (defence-in-depth, ADR-037 §4)
curl -s -o /dev/null -w "%{http_code}\n" "https://origa-cdn.t3.storageapi.dev/?list-type=2"
# Expected: 403
```

If this ever returns 200, treat as a security regression: Tigris has changed ACL semantics and the full 600k-object inventory is exposed. Mitigation: switch to Plan B (per-object ACL) or migrate to a bucket policy equivalent (when / if Tigris implements `PutBucketPolicy`).
