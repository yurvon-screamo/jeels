# AGENTS.md — Origa Core (`origa` crate)

Core business logic: domain models, use cases, traits, OCR, STT, dictionary. Rust edition 2024.

## Project Structure

```text
origa/src/
├── domain/
│   ├── error.rs            # OrigaError enum + ErrorCategory
│   ├── srs.rs              # FSRS spaced repetition
│   ├── knowledge/          # Card, Vocabulary, Kanji, Grammar, Phrase, Lesson, Stats
│   ├── memory/             # SRS memory state (value objects)
│   ├── tokenizer/          # Part-of-speech, translation domain types
│   └── grammar/            # Grammar forms + quiz generation
├── use_cases/              # ~20 business logic workflows
├── traits/                 # UserRepository, CdnProvider trait definitions
├── ocr/                    # NDLOCR-Lite pipeline (ONNX)
├── stt/                    # Whisper-based speech-to-text (ONNX)
└── dictionary/             # Furigana, grammar, kanji, phrase, vocabulary modules
```

## Error Handling

Single `OrigaError` enum (~40 variants) via `thiserror` 2.0, mapped to `ErrorCategory` (Domain / Infrastructure / Import).

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
pub enum OrigaError {
    #[error("Card with id {card_id} not found")]
    CardNotFound { card_id: Ulid },
    #[error("OCR processing failed: {reason}")]
    OcrFailed { reason: String },
}
```

Never `unwrap()` in production. Classify via `.category()` for UI handling.

## Conditional Compilation

Native vs WASM via `cfg(target_arch = "wasm32")`. Each module has `*.rs` (native) and `*_wasm.rs` counterparts. Native: `rusqlite`, `hound`. WASM: `ort` + `ort-web`.

## Key Dependencies

- **rs-fsrs** — spaced repetition algorithm
- **lindera** + SudachiDict — Japanese tokenization
- **ort** — ONNX Runtime (OCR + STT inference)
- **rkyv** — zero-copy dictionary deserialization
- **rusqlite** — SQLite for Anki import (native only)
- **ulid** — unique identifiers everywhere

## Conventions

- **IDs**: always `Ulid` — never raw strings or integers
- **Logging**: `tracing` only — never `println!`
- **Async**: `async fn` directly — never `#[async_trait]`
- **Dead code**: never `#[allow(dead_code)]`
- **Types**: explicit signatures on all public functions

## Testing

```bash
cargo test -p origa                    # All tests
cargo test -p origa test_name          # Specific test
cargo test -p origa -- --nocapture     # With output
```

## Testing Conventions

Tests are executable specifications. Four rules apply to every test in this crate. Canonical examples live in `domain/knowledge/daily_history_tests.rs`, `domain/srs.rs`, and `use_cases/tests/journeys/card_lifecycle.rs`.

### Naming — behavior, not method

Pattern (guideline, not a strict parser): `<subject>_<condition>_<expected_result>`. The goal is that the name alone tells you what behavior is verified — not which method is exercised.

- `merge_with_takes_higher_lessons` — what the merge does.
- `toggle_favorite_twice_returns_to_original_state` — multi-step behavior reads as a spec.
- `good_on_new_card_returns_positive_interval` — parametrized tests name the shared behavior.

No `test_` prefix — the `#[test]` attribute is the marker. One concept per test; if a name asserts two distinct observable outcomes that could fail independently, split into two tests. (A name may legitimately contain "and" when the outcomes are facets of one behavior, e.g. `rate_memory_again_on_new_card_returns_short_interval_and_learning_state`.) Multi-step journeys in `use_cases/tests/journeys/` use the same `<subject>_<condition>_<expected_result>` pattern — see `toggle_favorite_twice_returns_to_original_state` for a two-step journey that fits the standard pattern. Domain terms in names must match the canonical vocabulary in [`docs/glossary.md`](docs/glossary.md) — if a concept isn't there, add it before using the term.

### Structure — AAA with explicit comments

Non-trivial tests use `// Arrange` / `// Act` / `// Assert` comments (matches `rules-test-rule`):

```rust
#[tokio::test]
async fn toggle_favorite_twice_returns_to_original_state() {
    // Arrange
    let repo = InMemoryUserRepository::with_user(user_with_card("word"));
    let card_id = first_card_id(&repo).await;
    let use_case = ToggleFavoriteUseCase::new(&repo);

    // Act
    let first = use_case.execute(card_id).await.unwrap();
    let second = use_case.execute(card_id).await.unwrap();

    // Assert
    assert!(first, "first toggle turns favorite on");
    assert!(!second, "second toggle turns favorite off");
}
```

Trivial one-liners (`default_is_medium`, `daily_history_item_new_has_zero_defaults`) skip the comments. Extract setup into helpers (`user_with_card`, `first_card_id`) when the same arrangement repeats across tests.

### Parameterization — `rstest` when 3+ cases share a behavior

Rule: **3+ tests that differ only in input data and assert the same invariant MUST collapse into one `#[rstest]` parameterized test.**

```rust
#[rstest]
#[case::phrase_review(RateMode::PhraseReview, "PhraseReview")]
#[case::onboarding_scoring(RateMode::OnboardingScoring, "OnboardingScoring")]
#[case::grammar_review(RateMode::GrammarReview, "GrammarReview")]
#[case::kanji_review(RateMode::KanjiReview, "KanjiReview")]
#[case::short_term_backcompat(RateMode::ShortTerm, "FixationLesson")]
fn rate_mode_serde_roundtrip_preserves_wire_format(
    #[case] mode: RateMode,
    #[case] expected_json: &str,
) {
    let json = serde_json::to_string(&mode).unwrap();
    assert_eq!(json, format!("\"{expected_json}\""));
    let deserialized: RateMode = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, mode);
}
```

- Function name describes the **shared behavior** (`rate_mode_serde_roundtrip_preserves_wire_format`), not a single case.
- Each `#[case]` is a data variant; the body asserts the same invariant for all.
- Use `#[case::named]` to surface a sub-behavior in test output. The `short_term_backcompat` case above guards the only non-trivial `#[serde(rename = "FixationLesson")]` in `RateMode` — its name documents the invariant directly in test output.

**Do NOT collapse** tests whose bodies diverge in setup or assertions even when names look similar. Example: four "Again on new card" tests in `srs.rs` — `rate_memory_again_on_new_card_returns_short_interval_and_learning_state` (Standard), `phrase_review_again_returns_short_interval`, `grammar_review_again_returns_short_interval`, `kanji_review_again_returns_short_interval` — have per-mode richness: Standard asserts next-review date bounds + `CardState::Learning`, Phrase asserts `CardState::Learning`, Grammar/Kanji assert interval only. Collapsing them would either lose coverage or silently change what is asserted.

### Behavior, not implementation

Assert on observable outcomes (state, return values), not on internal method dispatch or private fields. A refactor that preserves behavior must not break tests. See `rules-test-rule` rules 1–2.

## Boundaries

**Always:** `cargo clippy -p origa -- -D warnings` + `cargo fmt` + all tests green before commit.
**Ask First:** changes to `domain/` or `Cargo.toml`.
**Never:** `unwrap()` in production, `#[async_trait]`, `#[allow(dead_code)]`, `println!`/`console.log`, removing tests.
