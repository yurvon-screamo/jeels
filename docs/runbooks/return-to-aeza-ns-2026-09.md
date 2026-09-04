# Return to Aeza NS — RU Cloudflare-NS blocking incident (2026-09-03)

**Status:** EXECUTED (zone restore pending Aeza NS propagation for MX/SPF/TXT)
**ADR:** [ADR-046](../decisions/ADR-046-return-to-aeza-ns-ru-cloudflare-blocking.md)
**Registrar:** OnlineNIC, Inc. (domain expires **2026-11-23** — renew before!)
**Aeza IDs:** service `1095525`, domain (v2) `2776`. API v2 auth: `X-API-Key` header
(**not** `Authorization: Bearer` — returns 401 `not_auth`; live-checked 2026-09-03).
Token: access doc `access/origa/aeza-dns-api.md` (uwuwu-cli access).

## What happened

NS were switched to Cloudflare on 2026-08-23 (undocumented). Since 2026-08-26 TSPU intercepts
DNS to Cloudflare from RU → whole zone SERVFAIL from Russia. Delegation returned to
`a`/`b.aeza-dns.net` on 2026-09-03 15:03 UTC. Aeza DB had only the Railway records; mail records
(MX ×2 / SPF / apple-domain) existed only in the CF zone and were restored via API from DoH cache.

## Verification

**Pre-flight (MUST pass before any NS switch — incl. rollback):**

```bash
# DNSSEC must be OFF (or DS re-signed) — stale DS at the parent + validating
# resolvers = SERVFAIL for everyone, worse than this incident
curl -s https://rdap.verisign.com/net/v1/domain/uwuwu.net | jq '.secureDNS'
dig DS uwuwu.net @a.gtld-servers.net +norecurse +noall +answer   # expect: empty
```

**Post-cutover:**

```bash
# Delegation (RDAP — authoritative truth, bypasses DNS caches)
curl -s https://rdap.verisign.com/net/v1/domain/uwuwu.net | jq '.nameservers'

# Authoritative answers (a=178.236.249.103, b=178.236.250.104)
dig @178.236.249.103 origa.uwuwu.net CNAME +short       # c2qj368z.up.railway.app (native Railway target since 2026-09-03)
dig @178.236.249.103 app.origa.uwuwu.net A +short      # 69.46.46.46 (CNAME → Railway, AAAA bug NOT hidden)
dig @178.236.249.103 s3.origa.uwuwu.net A +short       # 69.46.46.46
dig @178.236.249.103 pass.uwuwu.net A +short           # 69.46.46.55
dig @178.236.249.103 uwuwu.net MX +short               # mx01/02.mail.icloud.com
dig @178.236.249.103 uwuwu.net TXT +short              # spf1 + apple-domain

# MX/SPF/apple-domain propagation status (2026-09-03): authoritative NS a/b still
# return empty for MX/TXT at the time of writing (DB->NS lag, verified by direct
# dig); public resolvers may serve them from pre-cutover cache until TTL 3600
# expires (~16:10 UTC). Authoritative dig is the only valid check - cached
# public answers are NOT propagation evidence. Re-check per follow-up #1 of ADR-046.

# Public resolvers (expect mix of old/new NS for up to 48h — cache window)
curl -s "https://dns.google/resolve?name=origa.uwuwu.net&type=A" | jq '.Answer'

# Hosts serving
curl -sSI https://origa.uwuwu.net/ | head -3      # 200, server: railway-hikari
curl -sSI https://s3.origa.uwuwu.net/manifest.json | head -3

# From RU (use Yandex DNS — Google/CF DoH degrade from RU and are not valid tools there)
nslookup origa.uwuwu.net 77.88.8.8
nslookup app.origa.uwuwu.net      # ISP resolver
```

## Zone inventory (post-restore)

See `docs/backups/uwuwu.net.aeza.2026-09-03.zone` (authoritative dump via API).
`cdn.origa` → `origa-cdn.t3.tigrisbucket.io` is a legacy ADR-037 leftover (orphaned bucket).

## Gotchas (learned live, 2026-09-03)

1. **CF NS stop answering the moment delegation is switched** — zone contents are then
   unqueryable via the old NS; recover values only from resolver caches (DoH TTL window) or
   CF dashboard export. Dump the zone BEFORE switching delegation.
2. **Aeza DB → NS propagation is slow** (minutes…48 h). API shows records immediately; NS lag.
   Do not treat "record in API" as "record live".
3. **Zone drift between providers is real**: mail records lived only in CF (added ~July–Aug).
   Any future NS move must diff the FULL zone (all types, incl. apex MX/TXT/CAA), not a
   hand-copied record list.
4. Aeza new NS scheme is `a`/`b.aeza-dns.net` (old `ns1–4` still answer but are not the
   delegated set anymore).
5. **Record-type swap (e.g. A → CNAME) is not atomic** in the Aeza API: DNS forbids A+CNAME
   coexisting at one name, so it must be DELETE → POST. Safety pattern used for `origa`
   (2026-09-03): make the swap iso-IP (CNAME target resolves to the same IP the A-record had)
   so any mixed resolver state converges to identical routing — client-invisible by
   construction. The DB gap lasts seconds; worst case if the DB→NS sync lands exactly in the
   gap, one unlucky resolver caches NODATA for the SOA minimum (3600). Prefer iso-IP swaps.

## Rollback (back to Cloudflare)

0. **Pre-flight: DNSSEC check** (mandatory — see Verification/pre-flight): confirm DS is absent
   at the parent AND DNSSEC is off in the CF zone settings. If DNSSEC was ever enabled in the
   CF panel, disable it and wait for DS removal at the registrar BEFORE switching NS, or every
   validating resolver will SERVFAIL the zone.
1. Re-sync: diff Aeza zone (API dump) against the CF zone object; push any records added in Aeza
   after 2026-09-03 INTO the CF zone (CF dashboard / API). Both zones must be identical first —
   the dormant CF zone still lacks nothing only if re-synced.
2. Confirm the CF zone is still assigned NS pair `ali`/`clay.ns.cloudflare.com` in the CF
   dashboard (Cloudflare may reassign NS pairs to dormant accounts).
3. Switch delegation at the registrar (OnlineNIC) — or via the Aeza panel NS toggle if it is
   wired to the registrar as it was in June 2026 (ADR-021 era).
4. Verify per the commands above (expect CF NS to answer after propagation).
