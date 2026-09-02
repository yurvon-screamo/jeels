# ADR-045: Sync short-circuit for large knowledge sets

## Status

Accepted

## Date

2026-09-02

## Context

An N2 onboarding import grows the user's `knowledge_set` to ~6.3k study
cards — ~8 MB of JSON (~2.2 MB deflated on the server). Every home mount
ran `run_sync` → `merge_current_user` → a chain of full materializations of
that record in single-threaded WASM:

- remote GET + inflate + parse (×2: `merge_current_user` and again inside
  `remote.save`, which re-fetched the row just to learn its record id);
- local IndexedDB parse, merge, serialize, write;
- a **read-back**: `save_local_and_sync_remote` re-read and re-parsed the
  local record after writing it (added by refactoring commit `179003b8`
  with no functional reason);
- a write-only `user_cache` cloning the full `User` into resident memory
  (3 write sites, zero reads);
- `persist_user` on home unconditionally saved (and deep-cloned) the user
  after every `get_current_user` — twice per mount.

Peak memory and multi-second main-thread serialization on top of the
already-resident dictionaries/tokenizer got an iPhone WKWebView jetsam-
killed after onboarding (the app then crashed on every launch), and froze
the UI on the "Finish" button. Sentry stayed silent: an OOM kill cannot
report anything.

## Decision

### Steady state: fingerprint short-circuit

`SyncMeta` (`origa::use_cases::sync`) persists beside the local user record:

- `last_synced_fingerprint` — sha256 over the **raw server row** with
  `updated_at` excluded (it changes on every PATCH even when content does
  not). The fingerprint is **structural** — computed from the wire JSON
  value, so columns the `UserRow` struct does not declare still
  participate (`wire_fingerprint`). Evolution risk: any *future* volatile
  server column (auth metadata, counters) would silently degrade skip into
  a permanent full path — a documented failure mode, not a correctness bug.
- `dirty` + `dirty_epoch` — a local mutation sets the flag and increments
  the epoch.

`sync_merge` fetches the remote row **raw** (no decode) and returns early
when `!dirty && fingerprint matches && a local record exists`: one network
GET and one hash — no inflate, no parse, no merge, no writes, no PATCH.
The local-existence conjunct (a cheap IndexedDB key count) keeps the full
path reachable as the recovery route when the local store is missing or
corrupted — a clean fingerprint must never strand a user without local
data. A fresh device (no local record) seeds the local store straight from
the fetched row and records the fetched fingerprint **without pushing the
identical content back**.

### Full path (anything changed)

decode → merge → `mark_dirty` **before** the local write (crash between
the local save and the push leaves the flag set, so the next sync
re-pushes) → local save → push by record id → **re-fetch the raw row and
record its fingerprint**. The fingerprint is server-authoritative on
purpose: deriving it from the request body would silently break skip
matching whenever the server normalizes anything on storage.

### Lost-update protection (epoch CAS)

The sync captures `dirty_epoch` **after its own** `mark_dirty` and passes
it to `record_sync`, which clears the dirty flag only when the epoch is
unchanged. A card rated while the sync's push is in flight (seconds for a
large set) keeps the flag set, so the next sync re-merges and pushes the
newer local state instead of dropping it.

### Record-id threading

`remote.save` uses `session.record_id` (populated on create and self-healed
after every successful update) instead of re-fetching the full row; a
missing id falls back to a raw fetch (no decode). A failed update
re-resolves the row: a **live** record (whatever its id) retries the update
once and propagates the error on a second failure — creating a duplicate
row over a live record is never acceptable; only a genuinely missing row
falls through to create. A server row without a numeric id is an explicit
error, not a silent skip (a skip would make the caller create a duplicate).

### Home mount

`persist_user` saves only when `recalculate_user_jlpt_progress` actually
changed `jlpt_progress` (checked **before** any clone), so a no-change
mount neither writes nor deep-copies the user. User-action saves
(`save`, `save_sync`) mark the meta dirty **before** the local write — the
same crash-safety ordering as the full sync cycle, with the
`save_sync` window spanning the entire multi-second remote push.

### Storage

The meta record lives as a `sync_meta` key inside the existing IndexedDB
`users` store (out-of-line keys): no schema version bump, no migration.
`list_users` bounds its scan to the `user:` key prefix so the meta record
never surfaces as a "corrupted user entry". Account deletion resets the
meta to unsynced so a future login cannot inherit a stale fingerprint.

## Materialization map (honest)

- Full path before: ~9-10 heavy operations (see Context).
- Full path after: 2 raw GETs (merge + post-push re-fetch) with **one**
  decode, one local parse, two serializations, one PATCH; `user_cache`
  removed; read-back removed.
- Steady state after: 1 raw GET (no decode), 2 local parses (the home init
  effect and `run_sync` each load the user — an accepted remainder), zero
  writes, zero PATCH.

## Threat model

- **Fingerprint collisions**: sha256 over canonical field order —
  negligible.
- **Corrupted-but-present local record**: the skip-path probe is a keyed
  count, so a record that exists but no longer parses does not disable the
  skip. The window is narrow (corruption after the last successful sync
  with no subsequent user write — any write marks the meta dirty and
  re-seeds the record through the full path) and accepted: parsing the
  multi-megabyte record on every mount to detect it would defeat the
  short-circuit.
- **Duplicate rows by email** (legacy create races): the sync picks the
  smallest record id deterministically.
- **Meta write failure after a successful PATCH**: the flag stays dirty →
  one extra full sync. Accepted.
- **Concurrent local mutation between the sync's `local.get` and
  `local.save` can be overwritten by the merge result** — a pre-existing
  race in the old code, not worsened by this design; named here so it is
  not "discovered" as a regression later. WASM's single thread makes the
  window an await-interleave only.
- **Stale cached record id** (row re-created elsewhere): the update fails,
  the row is re-resolved and retried, then created.

## Verification

- `origa::domain::sync` unit tests (skip truth table, epoch CAS, crash
  windows).
- `wire_fingerprint` tests (updated_at stability, per-column sensitivity,
  key-order stability, unknown-column sensitivity).
- `sync_merge` orchestration tests against in-memory spies: steady state
  performs exactly one fetch and zero writes; the recorded fingerprint is
  server-authoritative (a spy that "normalizes" on save still settles into
  skip); a mutation inside the sync window keeps the flag; a failed push
  keeps the flag; duplicate rows resolve to the smallest id and settle into
  skip; a missing local record takes the full path despite a clean
  fingerprint; a fresh-device restore does not push identical content back.
- WASM (browser) tests for the IndexedDB pieces: the `sync_meta` roundtrip
  and missing-record fallback, the `user:` key range keeping the meta out
  of user listings, and the `has_any_user` existence probe.
- Device gates (before relying on it): first launch after updating on the
  6.3k-card account (the riskiest path — the full path still runs once)
  must survive; repeated launches and home mounts must show only GETs
  (no PATCH) in the server logs and no recurring
  "Skipping corrupted user entry" console noise.

## Future work (explicitly out of scope)

- Deduplicating the two per-mount local user loads.
- Moving serialization off the main thread (web worker) — the plan B if
  the first-launch full path still jetsams on low-memory devices.
- A compact binary wire format (bincode/rkyv) instead of deflated JSON.
- Co-locating the dirty-flag write with the user write in one IndexedDB
  transaction (currently one extra tiny write per user-action save).
- ETag/If-None-Match on the row GET to skip even the 2 MB download.
