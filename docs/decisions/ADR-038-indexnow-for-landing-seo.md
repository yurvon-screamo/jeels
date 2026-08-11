# ADR-038: IndexNow for landing SEO

## Status

Accepted (2026-08-11).

## Context

The Origa landing site (`origa.uwuwu.net`) is already well-optimised for organic
discovery: a complete `sitemap.xml` (78 URLs across 4 locales), `robots.txt`
that explicitly allows AI crawlers (ADR-017), `llms.txt` for generative engine
optimisation, hreflang alternates, correct `Cache-Control` tiers, and
canonical-URL enforcement. What is missing is a way to **push** notifications to
search engines when content changes, rather than waiting for crawlers to
re-discover the sitemap on their own schedule.

IndexNow is an open protocol supported by **Yandex**, **Bing**, **Naver**,
**Seznam**, **Yep**, and **Amazon** (but not Google). A site owner POSTs a list
of changed URLs to a single global endpoint (`api.indexnow.org/IndexNow`), which
fans out to all participating engines. The engines then prioritise crawling
those URLs ahead of their normal schedule.

For Origa, Yandex support is the primary motivation: the target audience
includes Russian-speaking learners, and Yandex is the dominant search engine in
that market. Bing covers English-language Bing/Edge/Copilot traffic. Google
does not participate in IndexNow, but already discovers the sitemap reliably.

Content changes happen at release cadence (stable tag → `docker.yml` → Railway
redeploy). This aligns with IndexNow best practices: submit on meaningful
content changes, not cosmetic updates; debounce at scale; never submit
retroactively.

## Decision

Implement IndexNow with three components:

### 1. Key file (`/{key}.txt`)

A UUID key (`e7825074-6888-4e03-a9ad-91459e4c9940`) is committed as
`origa_landing/public/{key}.txt`. The file content is the key itself. Search
engines fetch this file on first submission to verify domain ownership.

The key is served via an explicit Axum `ServeFile` route in `server.rs`, with
`Cache-Control: no-cache` — matching the policy on `robots.txt`, `sitemap.xml`,
and `llms.txt`. This ensures that if the key is ever rotated, search engines see
the new key immediately rather than a CDN-cached stale copy.

### 2. Notification script (`scripts/notify_indexnow.py`)

A zero-dependency Python script (standard library only) that:

1. Parses `origa_landing/public/sitemap.xml`, extracting every `<loc>` URL.
2. POSTs them as a JSON batch to `https://api.indexnow.org/IndexNow` with the
   key, key location, and host fields.
3. Handles batch splitting if the URL list ever exceeds the 10 000-URL limit.
4. Accepts `--dry-run` for local testing and `--sitemap` for custom paths.
5. Treats HTTP 200 and 202 as success (202 = first submission, pending key
   verification); any other status code exits non-zero so CI surfaces the
   failure.

The key is hard-coded in both the key file and the script constant. Rotating
the key requires updating both in the same commit.

### 3. CI step (`docker.yml` → `notify-indexnow` job)

A new job in the Docker workflow runs after `deploy-to-railway` on stable
releases only (`version_type == 'stable'`). It checks out the repo, waits 30
seconds for Railway to finish rolling the deployment, then runs the notification
script. The job is non-blocking: if IndexNow is down or returns an error, the
release pipeline has already succeeded (images built, Railway redeployed); the
failure is visible in the Actions log but does not roll anything back.

### Why the global endpoint, not per-engine

The IndexNow documentation states that submitting to **any one** participating
engine's endpoint propagates to all others. The global endpoint
(`api.indexnow.org/IndexNow`) is the simplest target and avoids maintaining a
list of engine-specific URLs that could change.

## Alternatives considered

### A1. Cloudflare IndexNow integration

Cloudflare offers native IndexNow submission on cache purge. Rejected: ADR-021
removed the `uwuwu.net` zone from Cloudflare authoritative NS. Re-introducing
Cloudflare proxy just for IndexNow would reverse that decision and reintroduce
the bot-management opacity and Yandex indexing issues documented there.

### A2. Per-engine submission (Yandex + Bing separately)

Submitting to `yandex.com/indexnow` and `bing.com/indexnow` individually.
Rejected: IndexNow's cross-engine propagation makes this redundant. The global
endpoint is simpler and equally effective.

### A3. Application-level trigger (Axum handler POSTs on startup)

The landing server itself POSTs to IndexNow on boot. Rejected: couples
SEO infrastructure to the application runtime, fires on every container restart
(not just content changes), and adds outbound HTTP dependencies to the SSR
server. CI-step is the documented best practice ("integrate into deployment
pipelines").

### A4. Yandex.Webmaster manual submit

Yandex.Webmaster provides a "Reindex page" form. Rejected: manual, does not
scale to 78 URLs across 4 locales, and cannot be automated in CI.

## Consequences

### Positive

- **Faster Yandex/Bing indexing** of new and updated landing pages, blog
  articles, and `/docs/*` content after each stable release.
- **Zero runtime cost**: no new dependencies in the Rust application, no
  background tasks, no outbound HTTP from the server.
- **Zero ongoing maintenance**: the script reads the sitemap dynamically, so
  new pages are picked up automatically without code changes.
- **Non-blocking**: CI failure in the IndexNow step does not affect the release.

### Negative

- **No Google benefit**: Google ignores IndexNow. Google indexing continues to
  rely on sitemap discovery and organic crawling. This is acceptable: Google's
  crawl of the sitemap is already reliable.
- **Key management**: the key is committed to the repository. This is by design
  — IndexNow keys are not secrets (they are publicly served at `/{key}.txt`),
  and a leaked key alone cannot submit URLs without also hosting the matching
  key file on the target domain.
- **First submission returns 202**: until search engines verify the key file,
  submissions return HTTP 202 instead of 200. This is expected and not an error.

## Verification

- `scripts/notify_indexnow.py --dry-run` confirms 78 URLs are extracted from the
  generated sitemap.
- `tests/cache_headers.rs::indexnow_key_file_has_no_cache` verifies the key file
  route serves with `no-cache`.
- The first real stable release after this ADR will produce a `notify-indexnow`
  job in the GitHub Actions run, observable in the workflow logs.

## References

- [IndexNow documentation](https://www.indexnow.org/documentation)
- [IndexNow FAQ](https://www.indexnow.org/faq)
- ADR-016 — cache-control policy precedent (`no-cache` for crawl-control files).
- ADR-017 — AI crawler allowlist in `robots.txt`.
- ADR-021 — Cloudflare NS removal (reason A1 is rejected).
- `origa_landing/src/server.rs` — key-file route registration.
- `scripts/notify_indexnow.py` — notification script.
- `.github/workflows/docker.yml` — CI integration (`notify-indexnow` job).
