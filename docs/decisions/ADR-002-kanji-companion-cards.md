# ADR-002: Kanji Companion Vocabulary Cards

## Status

Accepted. Note (2026-09-05): the startup reconciliation migration
(`MigrateKanjiCompanionsUseCase`) described below has been removed — all
decks are migrated. Companion creation now happens at kanji-add time
(onboarding sets / manual add / sets page); the removed-words guard moved
into `KnowledgeSet::create_companion_vocab_cards` (see
`docs/acquaintance-mode.md`).

## Date

2026-05-17

## Context

Kanji were taught in isolation — users learned characters without seeing them in the context of actual words. The kanji JSON data already contains `popular_words` (common words using each kanji) with links to the vocabulary dictionary, but this data was not used in lessons.

Without word context, users had no way to practice reading kanji in real words during their study sessions. The `popular_words` field in the kanji JSON represents an untapped resource: words that already exist in the vocabulary dictionary with translations.

## Decision

### What happens when a kanji card is created

1. Up to 3 `VocabularyCard` are automatically created from the kanji's `popular_words`, selecting only words that have matching entries in the vocabulary dictionary.
2. These companion cards are added to the user's `KnowledgeSet` alongside the kanji card itself.

### What happens during lesson building

1. After the lesson builder produces its standard card selection, a post-processing step adds companion vocabulary cards for each kanji present in the lesson.
2. This guarantee holds even if it causes the lesson to exceed the configured lesson size.
3. A hard cap of 15 total companion cards per lesson prevents unbounded growth for advanced users who have many kanji scheduled on the same day.

### Migration for existing users

A dedicated migration use case (`MigrateKanjiCompanionsUseCase`) iterates over existing kanji cards and creates companion vocabulary cards that are missing. This handles users who already have kanji cards from before this feature was introduced.

## Architecture

### Implicit link between kanji and companions

Companion cards are regular `VocabularyCard` instances. There is no `parent_kanji` field. The link between a kanji and its companions is resolved at lesson-build time by:

1. For each kanji card in the lesson, look up `popular_words` from the kanji JSON dictionary (CDN data).
2. Search the user's `KnowledgeSet` for `VocabularyCard` matching each word text.

This works because `popular_words` is stable (sourced from CDN JSON) and the number of cards per kanji is small (≤3).

### Why no explicit link field

Adding `parent_kanji: Option<String>` to `VocabularyCard` would make the relationship explicit and lookups O(1). However, it was rejected because:

- Changing `VocabularyCard` struct breaks deserialization for existing users with serialized data in IndexedDB.
- The implicit approach avoids any migration of stored data.
- The lookup cost is acceptable at current scale (linear scan over a small set).

### Shared creation logic

`KnowledgeSet::create_companion_vocab_cards()` contains the creation logic. Both `CreateKanjiCardUseCase` and `ImportOnboardingSetsUseCase` delegate to this method via a `User` proxy. This ensures onboarding also gets companion cards.

### Lesson post-processing in a separate module

The `kanji_companions.rs` module contains `add_kanji_companions()`, called after the main lesson building step. This keeps `lesson_builder.rs` focused on core FSRS scheduling logic.

### Idempotency

`DuplicateCard` errors from the knowledge set are handled gracefully. Calling companion creation multiple times (e.g., during migration or retry) produces the same result without side effects.

## Files Changed

- `origa/src/domain/knowledge/vocabulary.rs` — `from_known_word()` public constructor
- `origa/src/domain/knowledge/mod.rs` — `create_companion_vocab_cards()`, `MAX_COMPANION_WORDS = 3`
- `origa/src/domain/user.rs` — Proxy method to `KnowledgeSet::create_companion_vocab_cards()`
- `origa/src/domain/knowledge/kanji_companions.rs` — **NEW** — `add_kanji_companions()`, `MAX_COMPANION_CARDS_PER_LESSON = 15`
- `origa/src/use_cases/create_kanji_card.rs` — Companion creation after kanji card creation
- `origa/src/use_cases/import_onboarding_sets.rs` — Companion creation during onboarding import
- `origa/src/use_cases/migrate_kanji_companions.rs` — **NEW** — Migration use case for existing users

## Alternatives Considered

### Add `parent_kanji: Option<String>` to VocabularyCard

Would make the kanji→companion link explicit and enable O(1) lookups. **Rejected** because changing `VocabularyCard` struct breaks backward compatibility of serialization for existing users with data stored in IndexedDB.

### New card variant `KanjiCompanionVocab`

Would distinguish companion cards from regular vocabulary in the type system. **Rejected** because it adds a new `Card` variant that must be handled throughout the entire card processing pipeline (display, FSRS, serialization, etc.) for minimal benefit.

### Show companions only in kanji card view (no separate cards)

The simplest option: display popular words on the kanji card itself without creating separate cards. **Rejected** because it prevents FSRS from independently scheduling the companion words for spaced repetition review.

## Consequences

- Two constants with different purposes: `MAX_COMPANION_WORDS` (3, creation limit per kanji) and `MAX_COMPANION_CARDS_PER_LESSON` (15, total companion cards allowed in one lesson). These serve different stages of the pipeline and must not be confused.
- Companion cards increment `new_cards_studied_today` when rated — this is intentional. They are real vocabulary cards that contribute to the user's study count.
- `find_vocab_card` in `kanji_companions.rs` performs a linear scan over the `KnowledgeSet`. Acceptable at current scale, but may need indexing if the knowledge set grows significantly.
- Lessons may exceed the configured lesson size when companions are added. The 15-card cap is the only bound. This is by design: seeing kanji in context takes priority over strict size limits.
