# Grammar Data v2 — Schema and Provenance

Status: active (schema v2). Supersedes `cdn/grammar/grammar.json` (v1, frozen).

## File layout

- `cdn/grammar/grammar_v2.json` — the live corpus (schema v2) and the single
  source of truth. `cdn/` is gitignored; the file is deployed via
  `scripts/deploy_cdn.py` (release-updated cache policy,
  `max-age=300, must-revalidate`). Edit it directly, then validate.
- `cdn/grammar/grammar.json` — legacy v1, frozen since v2 went live. Kept for
  old clients until sunset (see GitHub issue linked in the v2 PR).

The authoring scaffolding (v1→v2 converter, one-off content passes, N1 batch
files and the batch merger) lived under `scripts/` during the v2 build-out and
was removed once the corpus was finalized — the merged corpus above supersedes
it. History: the v2 PR and its predecessors in this repository.

## Schema (v2)

```jsonc
{
  "schema": 2,
  "grammar": [
    {
      "rule_id": "<ULID, 26 chars>",          // stable forever: SRS cards link by it
      "level": "N5|N4|N3|N2|N1",
      "keywords": [["なさい", "なざる"]],        // optional, search aliases
      "format_map": { /* optional, conjugation chains */ },
      "content": {
        "English": {
          "title": "～うと～うと",
          "short_description": "Whether A or B",
          "explanation": "markdown, no emoji",
          "warnings": ["..."],                 // lifted out of explanation
          "how_to_form": "| markdown table |",
          "examples": "``` fenced jp + translation ```",
          "nuances": {
            "common_mistakes": [ { "wrong": "...", "correct": "...", "note": "..." } ],
            "notes": [ { "tag": "register|variation|collocation|formality|other", "text": "..." } ]
          },
          "pro_tip": "...",
          "related_patterns": [ { "rule_id": "<ULID>", "relation": "pair|contrast|derived", "note": "..." } ]
        },
        "Russian": { /* same shape */ }
      }
    }
  ]
}
```

Hard invariants (enforced by `scripts/validate_grammar_v2.py`, exit 1 on error):

- No emoji anywhere in the data — presentation is the UI's job.
- No simplified-Chinese characters: Japanese text must encode in JIS X 0213
  (`euc_jis_2004`), beyond an explicit punctuation allowlist.
- `examples` must be fenced code blocks (bold group labels allowed).
- `related_patterns.rule_id` must resolve inside the corpus; no self-reference.
- `rule_id` is a valid ULID and unique; existing ids are never regenerated
  (user SRS cards reference them).

Non-blocking warnings: duplicate normalized titles (kept by user decision),
empty optional fields tracked for content passes.

## Pipeline

| Step | Tool |
|------|------|
| Edit content | `cdn/grammar/grammar_v2.json` directly (keep `rule_id`s stable — SRS cards reference them) |
| Validate | `python scripts/validate_grammar_v2.py [path]` |
| Deploy | `python scripts/deploy_cdn.py` |

Historical steps (v1→v2 conversion, CN-char and hollow-rule passes, N1 batch
authoring and merging) are complete and their one-off tooling was removed.

## Current state

- 515 legacy rules (N5–N2) converted from v1, then cleaned (CN-char fixes,
  hollow rules filled, format anomalies normalized).
- 205 N1 rules (14 thematic batches, writer + reviewer passes done; every rule
  survived the strict validator — zero emoji, zero CN chars, fenced examples,
  resolved references). All curated N1 topics are covered: as rules or as
  explicit variation notes on related rules; three topics were junk-skipped
  during curation.
- Phrase index re-enriched against v2 (`utils enrich-phrases-with-grammar`):
  grammar links precomputed per phrase; N1 keywords were audited so that only
  unambiguous compound forms participate in detection.

## Attribution

N1 topic lists were curated from two open sources (topics only — no prose was
copied; all explanations/examples are original):

- hanabira.org grammar lists — CC license, attribution requested:
  <https://hanabira.org>.
- jkindrix/japanese-language-data grammar points — CC BY-SA 4.0:
  <https://github.com/jkindrix/japanese-language-data>.

Grammar point names are facts; the CC attribution above is honored as good
practice for the curation effort. The source snapshots kept during curation
were removed along with the authoring scaffolding.
