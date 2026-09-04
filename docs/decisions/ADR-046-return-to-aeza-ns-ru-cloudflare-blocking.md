# ADR-046: Return DNS Delegation to Aeza NS — RU Blocking of Cloudflare Authoritative NS

## Status

Accepted

## Date

2026-09-03

## Context

Since 2026-08-26, Russian TSPU (DPI) began intercepting plain DNS queries to Cloudflare/Google
public resolvers, on top of the existing registry-level blocking of significant parts of
Cloudflare's IP space (in effect since June 2025, tightened in 2026). The `uwuwu.net` zone was
delegated to Cloudflare authoritative NS (`ali`/`clay.ns.cloudflare.com`) — an **undocumented**
change made at the registrar (OnlineNIC RDAP `last changed: 2026-08-23`). Result: Russian
recursive resolvers could not reach the authoritative NS → SERVFAIL for the entire zone →
`origa.uwuwu.net` (landing), `app.origa.uwuwu.net` (UI + TrailBase) and `s3.origa.uwuwu.net`
(CDN) all became unreachable from Russia. `*.up.railway.app` kept working (Railway-operated NS;
same edge IPs `69.46.46.46/55`), which isolated the failure to zone resolution, not to hosting.

This is the **second** time Cloudflare routing hurt RU availability: PR #372 (2026-08-10) reverted
the user-Tigris CDN migration (ADR-037) because of TSPU DPI-throttling of Cloudflare-fronted
routes from Russia.

### NS migration history of `uwuwu.net`

| Date | Change | Documented |
| --- | --- | --- |
| 2026-06-13 | Aeza NS + plain A-record (SERVFAIL fix) | ADR-007 |
| 2026-06-24 | → Cloudflare NS (proxied) | ADR-007 update |
| 2026-06-28 | → back to Aeza NS | ADR-021 |
| 2026-08-23 | → Cloudflare NS (DNS-only) at OnlineNIC | **none** (root-caused during this incident) |
| 2026-09-03 | → back to Aeza NS (`a`/`b.aeza-dns.net`, new Aeza scheme) | this ADR |

## Decision

Return the zone delegation to Aeza authoritative NS (`a`/`b.aeza-dns.net`). Cloudflare is removed
from the DNS path entirely; the CF zone object remains in the account as a rollback reference
(see rollback in the runbook).

Execution (2026-09-03):

1. User switched registrar delegation to `a`/`b.aeza-dns.net` (RDAP last-changed 15:03:35 UTC).
2. Zone state audited via direct queries to authoritative NS **and** Aeza API v2 (X-API-Key auth):
   Railway records present (A + CNAME + `_railway-verify` TXT), but **mail records were missing**
   — MX ×2 (iCloud), SPF TXT and `apple-domain` TXT existed only in the CF zone (added after
   2026-06-28, never mirrored into the Aeza DB).
   Note on the audit discrepancy: during planning, the OLD Aeza NS fleet (`ns1–4.aeza-dns.net`)
   showed only `origa A` — that fleet serves stale data; the live DB (served by the new
   `a`/`b.aeza-dns.net` fleet) already contained all Railway records (old record ids). Only the
   4 mail records were genuinely absent and had to be created.
3. Missing records restored via Aeza API v2 from DoH-cached values of the former CF zone
   (CF NS stopped answering authoritatively right after delegation switch, so the live zone was
   no longer queryable — values captured from resolver cache).
4. Zone dumped to `docs/backups/uwuwu.net.aeza.2026-09-03.zone`.
5. Same day (user request): `origa` switched from the hardcoded A-record `69.46.46.46`
   (ADR-007 workaround) to a native CNAME → `c2qj368z.up.railway.app` (the landing service's
   current CNAME target per the Railway panel). Rationale: the A-record was a single-IP point of
   failure, and its promised SEO benefit never materialized — note this is an **assumption, not
   a proven verdict**: ADR-021 left the Yandex-indexing question explicitly open (Open Question
   #1), and user judged the trade-off not worth the IP fragility. User explicitly accepted the
   AAAA SERVFAIL regression (see Follow-up 5 for the monitoring trigger).

## Consequences

**Positive:**

- Zone resolvable from Russia again (Aeza NS are RU-reachable). All three public hosts verified
  200 OK externally after cutover.
- `origa`/`app`/`s3`/`pass` are all served by Aeza as CNAME → Railway (A answers visible in
  the chain) — no more hardcoded edge IP that silently breaks when Railway changes it.
  **Known non-regression:** `AAAA` for these subdomains still returns SERVFAIL
  (verified live post-cutover, Status 2) — the long-standing Railway DNS bug documented in
  ADR-021:83. CNAME-flattening is NOT performed by Aeza; the bug is NOT hidden. Tauri desktop
  clients connect over IPv4 and are unaffected; strict AAAA-first resolvers may still struggle
  with these hosts. For the landing this risk is consciously accepted as an assumption-based
  trade-off: the ADR-007 A-record did not observably improve Yandex indexing (ADR-021 left the
  question open, but no benefit was ever observed), so the single-IP fragility of the A-record
  outweighed the speculative AAAA benefit. Detection of any real-world regression is tracked
  as Follow-up 5.
- No Cloudflare in the DNS path — one fewer variable in RU availability incidents.

**Negative / risk:**

- No DDoS protection / edge caching in front of DNS or HTTP (same as post-ADR-021 state).
- Aeza DB → NS propagation is slow (minutes to 48 h, confirmed by operator during ADR-037):
  the restored MX/SPF/TXT records lag behind the API state; incoming mail on `uwuwu.net`
  degrades for resolver caches that already saw the new delegation until propagation completes.
- Single vendor for DNS + slow propagation makes emergency DNS changes painful.
- **NS redundancy reduced:** Aeza's new scheme is 2 nameservers (`a`/`b.aeza-dns.net`,
  `178.236.249.103`/`178.236.250.104` — adjacent ranges of the same provider) instead of the
  previous 4 (`ns1–4`), shrinking failure-domain diversity for zone availability.
- If the real RU blocker is an IP-range block on Railway edge (alternative hypothesis), this
  migration does not fix it — but the `*.up.railway.app`-works symptom argues against that.

**Alternative hypothesis (recorded, not resolved):** TSPU may block by IP rather than by NS
reachability. Falsification criterion: if after full NS propagation the zone resolves from RU
(`nslookup origa.uwuwu.net 77.88.8.8` returns A) but HTTPS still fails → IP/SNI blocking, reopen
investigation. Verification from RU must use Yandex DNS (77.88.8.8) or the ISP resolver —
Google/Cloudflare DoH themselves degrade from RU and are not valid tools there.

**During the delegation-cache window (~up to 48 h):** any urgent DNS change must be applied in
BOTH zones (Aeza DB and the dormant CF zone) while resolvers still hold cached old NS.

## Follow-ups

1. Verify MX/SPF/apple-domain on `a`/`b.aeza-dns.net` after propagation; re-check mail flow.
2. Renew the domain at OnlineNIC **before 2026-11-23** (expires; flagged 2026-09-03).
3. `cdn.origa` CNAME → `origa-cdn.t3.tigrisbucket.io` is a legacy ADR-037 leftover pointing at
   the orphaned user-Tigris bucket awaiting cleanup (see AGENTS.md) — decide remove/keep.
4. Update wiki `hosting/aeza-api_`: v2 auth is `X-API-Key`, not `Authorization: Bearer`
   (Bearer returns 401 `not_auth` — live-checked 2026-09-03).
5. **Watch for AAAA-regression fallout on the landing** (the only LLM-facing surface): after the
   `origa` CNAME lands, monitor Yandex Webmaster / GSC for "DNS error / not indexed" signals for
   2–4 weeks (ADR-007's failure mode was silent, week-long, via Webmaster reports). Trigger: if
   indexed-page count drops or DNS errors reappear → swap `origa` back to the A-record
   (DELETE CNAME id=58715, POST A 69.46.46.46 ttl 300; content IP verified via
   `dig c2qj368z.up.railway.app A +short`).

## References

- `docs/runbooks/return-to-aeza-ns-2026-09.md` — execution runbook + rollback
- `docs/backups/uwuwu.net.aeza.2026-09-03.zone` — zone dump (post-restore)
- ADR-021 (previous revert), ADR-007 (original SERVFAIL history), ADR-037 + PR #372 (CF DPI-throttle precedent)
- TSPU DNS interception reporting: https://vc.ru/services/3102791 ,
  https://habr.com/ru/articles/1075272 (2026-08-26+)
