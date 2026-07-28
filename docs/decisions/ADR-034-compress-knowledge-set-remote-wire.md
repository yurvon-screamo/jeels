# ADR-034: Compress `knowledge_set` on the remote-sync wire (deflate over JSON)

## Status

Accepted

## Date

2026-07-29

## Context

The `User` aggregate (`origa/src/domain/user.rs`) holds the entire
`KnowledgeSet` — every `StudyCard` with its FSRS `MemoryState`, `ReviewLog`
history, `deleted_cards`, and `lesson_history`. For an active learner this set
grows to multiple megabytes.

`HybridUserRepository::save_sync` (`origa_ui/src/repository/hybrid_repository.rs`)
serializes the **whole** user to JSON and PUTs it to TrailBase as a single
`user`-table row on every sync checkpoint: auth, onboarding, imports,
`toggle_favorite`, create/delete card. The `knowledge_set` field is a plain
JSON string inside that body. Symptoms, as reported for active users:

- every checkpoint PUT ships ~10 MB;
- UI blocks while the encode + network round-trip runs;
- on mobile, the bandwidth cost is paid per request.

TrailBase's default request body limit is **10 MB** (`crates/core/src/server/mod.rs`,
`request_size_limit_bytes.map_or(10 * 1024 * 1024, ...)`). Active users were
already on the edge of that limit, so the uncompressed format is not just
slow — it is one card away from rejected writes.

### Scope of this ADR

This is a **band-aid** that shrinks the wire payload of the existing
full-snapshot PUT. It does **not** restructure the aggregate (no normalization,
no delta sync, no per-card rows). Those are tracked as a future "variant C"
option; this ADR buys the time to do them deliberately. The hot path (rating a
card during a lesson) already writes local-only (`save`, not `save_sync`), so
the bandwidth fix targets the checkpoint PUTs only.

## Decision

Compress `knowledge_set` on the remote wire using **deflate over the existing
serde-JSON serialization**, with a self-describing magic-prefix discriminator.
The codec lives entirely in the repository layer (`origa_ui/src/repository/
knowledge_set_codec.rs`); `domain/` is untouched.

### Wire format

- **new:** `"DEFLATE;" + base64(deflate(json_string))`
- **legacy (existing rows):** plain JSON. JSON objects always start with `{`,
  and `{` cannot appear at the prefix position, so presence/absence of
  `DEFLATE;` is an unambiguous, self-describing format discriminator. No
  companion column is required, so there is no field/value-desync risk.

base64 is unavoidable: the TrailBase column is `TEXT` (`Option<String>`), so
raw deflate bytes cannot be stored. Its ~33% overhead is more than offset by
deflate's compression ratio.

### Compression level: 6 (data-driven)

Chosen by a PoC gate (`origa_ui/tests/knowledge_set_format_poc.rs`, run with
`--ignored`) on a representative ~8 MiB fixture (6000 cards × 8 reviews), in
release:

| format | wire size | ratio | enc+decode |
| --- | --- | --- | --- |
| raw JSON (baseline) | 8.22 MB | 1.00x | 126 ms |
| bincode + base64 | — | — | **roundtrip FAILED** (see Alternatives) |
| deflate(JSON) level 1 | 2.60 MB | 3.15x | 143 ms |
| **deflate(JSON) level 6** | **1.75 MB** | **4.69x** | **197 ms** |
| deflate(JSON) level 9 | 1.73 MB | 4.75x | 293 ms |

Level 6 is the ratio/latency sweet spot. Level 9 buys +0.06x ratio for +100 ms
encode; level 1 loses meaningful ratio. WASM projects to ~2x native
(≈400 ms), within the sync-checkpoint latency budget (≤500 ms; checkpoints are
not the rating hot-path).

### Error policy: read-recover, write-strict

A single parser (`decode_strict`) underlies both policies, so roundtrip tests
that exercise strict also cover the recovering path.

- **Read is tolerant.** `decode` never fails: any corruption (truncated base64,
  bad deflate stream, malformed JSON, unknown prefix) is logged at `warn!` and
  replaced with an empty `KnowledgeSet`. This **preserves the existing
  self-heal**: `KnowledgeSet::merge(empty)` is a no-op
  (`origa/src/domain/knowledge/mod.rs:131`), so a corrupt remote merges as
  nothing, and the next `save_local_and_sync_remote` overwrites the corrupt
  row with the device's local data. The user never loses device-local progress.
- **Write is strict.** `user_to_json` returns `Result`; a serialization failure
  on any field surfaces as `Err` instead of silently writing a corrupt fallback
  to the remote. This is a deliberate asymmetry from read: be tolerant of
  damaged input, strict about damaged output.

### Back-compat

- **Read:** the discriminator selects legacy-JSON or deflated at decode time.
  No batch migration script; rows are read in either format.
- **Write:** always deflated after upgrade. **Lazy self-migration** — the first
  `save_sync` after upgrade rewrites an existing legacy row in the new format.
- **Pre-deploy:** back up the TrailBase database before release (the project's
  standing data-loss requirement).

## Alternatives Considered

### A1: rkyv (zero-copy binary) + flate2

The "modular" stack already used for dictionaries. Rejected after
investigation: `KnowledgeSet` carries `#[serde(flatten)] stats: StatsTracker`
(`knowledge/mod.rs:76`) and a custom `#[serde(deserialize_with =
"deserialize_study_cards")]` — neither maps to rkyv without mass-deriving
`Archive`/`Serialize`/`Deserialize` across ~15 domain types (`StudyCard`, `Card`
enum, four card types, `MemoryState`, `ReviewLog`, `Stability`, `Difficulty`,
`StatsTracker`, …). `ulid 1.2` and `chrono 0.4` (current configs) lack rkyv
features, so they would need wrappers. That turns a band-aid into a deep
`domain/` invasion (an "Ask First" zone) with real risk to newtype invariants
(`Stability::new` / `Difficulty::new` validate ranges that rkyv would bypass).
Out of proportion to the goal.

### A2: bincode (compact binary) + base64

Already in `origa_ui` deps, zero compression CPU cost. The PoC **disqualified**
it: `bincode 1.x` is incompatible with `serde(flatten)` — `KnowledgeSet`'s
`#[serde(flatten)] stats` makes `bincode::serialize` fail outright (flatten
requires self-describing `deserialize_any`). This is exactly why the format
choice was made data-driven rather than assumed.

### A3: HTTP-level `Content-Encoding: gzip` on the request body

The canonical payload-compression mechanism. Rejected for scope: TrailBase
(axum) ships **without** `RequestDecompressionLayer` in its default middleware
stack, so enabling it requires a server-side deployment with access to the
backend config — outside an app-only band-aid. Application-level codec is
self-contained and server-agnostic.

### A4: Debounce / delta sync

Debounce `save_sync` to coalesce writes; or send per-card deltas instead of the
full snapshot. Rejected for this ADR: debounce conflicts with the
error-returning semantics of the checkpoint use cases (auth/onboarding/imports
return `Err` on remote failure and must propagate). Delta sync requires
server-side merge, versioning, and conflict resolution — that is the future
"variant C" restructure, not a band-aid.

## Consequences

### Positive

- **~4.69x smaller wire payload** on the checkpoint PUT (10 MB → ~2 MB for an
  active user), well under TrailBase's 10 MB body limit.
- **Zero new dependencies** — `flate2`, `base64`, `serde_json` are all already
  in `origa_ui`. `flate2` is already proven in WASM by the dictionary loader
  (`origa_ui/src/loaders/dictionary.rs`).
- **`domain/` untouched.** The codec is a repository-layer concern; the
  aggregate and its invariants are unchanged.
- **Self-heal preserved.** The read-recover policy keeps the existing
  corrupt-remote → empty → local-overwrites-remote behavior; corruption is now
  also surfaced via `tracing::warn!` rather than staying silent.
- **Write-strict closes a latent silent-fallback bug.** `user_to_json` no longer
  `unwrap_or_else`s serialize failures into corrupt fallback values written to
  the remote.

### Negative — downgrade and data-loss windows

Three distinct scenarios, with the mitigation that covers each:

1. **New-client codec bug** (the new code corrupts data on write). Mitigation:
   local IndexedDB is the authoritative device copy and is untouched by this
   change; back up the TrailBase DB before release. The local copy lets the
   device recover.
2. **Existing device fails to decode a new-format remote** (decode path broken).
   Mitigation: read-recover resolves to empty, self-heal re-uploads local data.
   Covered by the regression test in
   `trailbase_repository_tests.rs::userrow_to_user_self_heals_on_corrupt_knowledge_set`.
3. **Fresh login on a downgraded (pre-upgrade) client** against a remote already
   written in the deflated format. The old client has no local copy on that
   device; it sees a non-JSON `knowledge_set` string, `serde_json::from_str`
   fails, and the pre-existing `unwrap_or_default` yields an empty
   `KnowledgeSet`; a subsequent `save_sync` would overwrite the remote with
   empty. **This is a residual risk that neither local-authoritative nor the
   codec can mitigate.** The only mitigation is rollout discipline: staged
   rollout and no rollback past this change (Tauri auto-update / web instant
   reload keep the version-spread window small). For mobile with app-store
   updates this is acceptable, but it must be known and not rolled back
   carelessly.

- **`base64` adds ~33%** over raw deflate bytes. Accepted: the column is `TEXT`,
  so raw bytes are not storable without a DB migration (out of band-aid scope).
  The net 4.69x is comfortably above the goal.
- **CPU on the checkpoint path.** encode + decode ≈ 197 ms (native release) /
  ≈ 400 ms (projected WASM) on an ~8 MiB set. Acceptable for a non-hot-path
  checkpoint; if a future user exceeds this and the freeze becomes noticeable,
  deflate level can be lowered or encode moved to a worker (future work).
- **Unbounded decompression allocation.** `inflate` decodes into a `Vec<u8>`
  with no upper bound on the decoded size. On the current trust model this is
  academic: the `knowledge_set` column is written and read by this same
  authenticated client, and realistic corruption (a truncated or bit-flipped
  deflate stream) produces an early decode error rather than a size explosion.
  If the threat model ever broadens (e.g. the column becomes writable by a path
  other than this client), a decoded-size cap must be added to `inflate` to
  prevent a maliciously crafted stream from exhausting WASM client memory.

## Verification

| Check | Command | Result |
| --- | --- | --- |
| Format choice gate | `cargo test -p origa_ui --test knowledge_set_format_poc --release -- --nocapture --ignored` | deflate(JSON) L6 chosen; bincode disqualified by roundtrip |
| Codec unit tests | `cargo test -p origa_ui repository::knowledge_set_codec` | 5 functions / 7 cases passed (roundtrip, legacy-decode, carries-prefix, smaller-than-json, recovering-`#[rstest]`×3: corrupt-legacy / corrupt-prefixed / unknown-prefix) |
| Wire roundtrip + self-heal | `cargo test -p origa_ui repository::trailbase_repository` | 2 passed (UserRow roundtrip, corrupt self-heal) |
| Lint | `cargo clippy -p origa_ui --all-targets -- -D warnings` | 0 warnings |
| Format | `cargo fmt -p origa_ui -- --check` | clean |

## References

- `origa_ui/src/repository/knowledge_set_codec.rs` — the codec
- `origa_ui/src/repository/trailbase_repository.rs` — `user_to_json` (write-strict), `to_user` (read-recover)
- `origa_ui/tests/knowledge_set_format_poc.rs` — the data-driven PoC gate
- ADR scope note: "variant C" (aggregate normalization / delta sync) deferred — see plan v3 review record.
