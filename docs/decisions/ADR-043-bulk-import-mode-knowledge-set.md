# ADR-043: Bulk-import mode for KnowledgeSet (O(n) card creation)

## Status

Accepted. Note (2026-09-05): `MigrateKanjiCompanionsUseCase` (one of the
three motivating paths below) has been removed with the startup companion
migration — see the note in ADR-002.

## Date

2026-09-01

## Context

Bug: onboarding import is unusably slow for JLPT N2 users. Picking N2
queues the cumulative `jlpt_n5+n4+n3+n2` sets (~4.9k words → **6341 cards**:
vocabulary + kanji + companion vocab + grammar). Every
`KnowledgeSet::create_card` ran **two O(n) full scans**:

1. `validate_unique_card` — linear scan over all existing cards;
2. `recalculate_daily_stats` — two full passes (aggregate stats + rating
   fold) over all cards.

Per-card cost therefore grew linearly with card count → **O(n²) imports**.
Measured (native debug): N5 import = 916 cards / 47 ms (51 527 ns/card);
N2 import = 6341 cards / 1584 ms (249 799 ns/card) — **per-card ratio 4.86**
(the smoking gun; linear code stays near 1.0). On release-WASM in the
browser this froze the UI for the whole "Start import" step, and N1 /
extra app sets scale it further.

The same quadratic pattern existed in three paths:

- `ImportOnboardingSetsUseCase` (onboarding, the reported bug),
- `ImportAnkiPackUseCase` (Anki pack import),
- `MigrateKanjiCompanionsUseCase` (runs on **every cold start**, iterating
  companions per kanji + a delete loop with a full stats recalc per
  delete).

## Decision

Add a **transient bulk-import mode** to `KnowledgeSet`:

- `#[serde(skip)] import_dedup_index: Option<HashSet<(CardType, String)>>`
  — never serialized, rebuilt from `study_cards` on
  `begin_bulk_import()`, dropped on `end_bulk_import()`.
- While active: `create_card` checks uniqueness against the index (O(1))
  and **defers** the daily-stats recalc; `delete_card` also defers.
  `end_bulk_import()` performs **one** recalc for the whole batch.
- Dedup key = `(CardType, content_key())`: the type discriminates
  cross-type collisions (vocabulary「日」and kanji「日」are distinct cards —
  same semantics as the old same-type-only `match`).
- Outside the bracket, behavior is byte-for-byte unchanged (linear
  validate + per-card recalc).

`User::begin_bulk_import` / `end_bulk_import` (`pub(crate)`) wrap the three
use cases above. Result: N2 import **1584 ms → 139 ms (11.4x)**, per-card
ratio N2/N5 → ~1.0.

## Alternatives Considered

### Permanent content-key index (always maintained)

Pros: O(1) uniqueness everywhere, no bracket discipline.
Cons: the index must be rebuilt on every deserialization (custom
`Deserialize` for the whole struct — the wire format has a custom
study-cards visitor + `#[serde(flatten)]` stats), and maintained in every
mutation (`create`/`delete`/`update`/`merge`) forever. More invariants to
break for a hot path only imports need. Rejected: transient mode gets the
same complexity win with zero permanent invariants and no wire-format risk.

### Incremental stats (running aggregates in StatsTracker)

Pros: O(1) per create even outside imports.
Cons: stateful counters, float drift on subtract, and `recalculate` also
recomputes today's ratings from `last_review_date` — the incremental path
would duplicate that logic. Rejected: the final recalc is a **pure
function of the final card set + preserved day counters**, so deferring to
one call is provably equivalent (verified by
`bulk_import_creates_same_daily_stats_as_one_by_one_creation` and
`bulk_import_after_same_day_reviews_preserves_counters_and_ratings`).

### Batch-only fast path inside the use cases (local HashSet per import)

Rejected: companion creation (`create_companion_vocab_cards`) also needs
dedup, so the use case would have to replicate domain dedup logic — leaky.

## Consequences

- Bracket discipline: callers must pair `begin_bulk_import` with
  `end_bulk_import`. An unclosed bracket cannot corrupt persistence (early
  error returns drop the unsaved local `User`; the index is
  `#[serde(skip)]` so a restart clears it) but would leave daily stats
  stale until the next recalc trigger (any rate/delete).
- `update_card_content` and `merge` **drop the index** (degrade to the
  linear scan) instead of maintaining it: content replacement could alias
  keys (one key owned by two cards → false-negative uniqueness after a
  delete), so the invariant "one card ↔ one key" is made unbreakable
  rather than patched. These paths are not bulk paths.
- Only divergence from the per-card recalc sequence: a batch spanning
  UTC midnight loses the previous-day snapshot item the per-card path
  would have written (window ≈ 140 ms; accepted).
- Regression guard: `import_cost_grows_linearly_with_card_count`
  (journeys) runs the real production sets from `cdn/` best-of-3 per
  scenario and asserts per-card cost ratio N2/N5 < 2.0 (quadratic code
  measures ~4.86; linear ~1.0).
