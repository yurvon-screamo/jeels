# Grammar Data v2 — Schema and Provenance

Status: active (schema v2). Supersedes `cdn/grammar/grammar.json` (v1, frozen).

## File layout

- `cdn/grammar/grammar_v2.json` — the live corpus (schema v2). `cdn/` is
  gitignored; the file is deployed via `scripts/deploy_cdn.py` (release-updated
  cache policy, `max-age=300, must-revalidate`).
- `cdn/grammar/grammar.json` — legacy v1, frozen since v2 went live. Kept for
  old clients until sunset (see GitHub issue linked in the v2 PR).
- `scripts/data/n1/*.json` — hand-authored N1 batches (source of truth for
  review; merged by `scripts/merge_n1_grammar.py`).
- `scripts/data/n1/remaining_topics_backlog.csv` — curated N1 topics not yet
  written (with dedup/junk flags from topic curation).
- `scripts/data/n1/source_*.csv|json` — upstream topic-list sources (provenance
  snapshots, see Attribution).

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
| Validate | `python scripts/validate_grammar_v2.py [path]` |
| Convert v1 → v2 (done once) | `python scripts/convert_grammar_v2.py` |
| Content passes (done) | `scripts/fix_grammar_cn_chars.py`, `scripts/fill_hollow_grammar_rules.py` |
| Author N1 batches | hand-written files in `scripts/data/n1/batch_*.json` |
| Merge batches | `python scripts/merge_n1_grammar.py [--dry-run]` — assigns ULIDs, resolves `related_titles` to rule_ids, transactional |
| Deploy | `python scripts/deploy_cdn.py` |

Batch authoring notes: a batch rule carries `level: "N1"`, both languages,
and may use `"related_titles": [["～ざるを得ない", "pair", "note..."]]` which the
merger resolves against the merged corpus. The merge aborts without writing on
any validation error.

## Current state

- 515 legacy rules (N5–N2) converted from v1, then cleaned (CN-char fixes,
  hollow rules filled, format anomalies normalized).
- 141 N1 rules across 9 thematic batches (writer/reviewer pass done in-session;
  every batch survived the strict validator — zero emoji, zero CN chars,
  fenced examples, resolved references).
- Remaining N1 topics: see `remaining_topics_backlog.csv` (status `OK` rows
  not yet covered by a batch; ~144 topics after family merges). Continue by
  adding `batch_10_*.json` files in the same format.

## Attribution

N1 topic lists were curated from two open sources (topics only — no prose was
copied; all explanations/examples are original):

- hanabira.org grammar lists — CC license, attribution requested:
  <https://hanabira.org> (source snapshot: `source_hanabira_n1.csv`).
- jkindrix/japanese-language-data grammar points — CC BY-SA 4.0
  (source snapshot: `source_jkindrix_grammar.json`).

Grammar point names are facts; the CC attribution above is honored as good
practice for the curation effort.
