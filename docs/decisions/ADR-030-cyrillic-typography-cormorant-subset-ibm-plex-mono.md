# ADR-030: Cyrillic typography — Cormorant subset + IBM Plex Mono swap

## Status

Accepted

> **Updated (2026-07-31): CJK regression-guard moved to Noto Sans JP.**
> `subset_fonts.py::verify_glyph_coverage` now runs the CJK `MUST_HAVE` check
> against `noto-sans-jp-400-*` (was `noto-serif-jp-400-*`), because Noto Serif JP
> was removed when CJK was unified on Noto Sans JP — see ADR-028 §Status (2026-07-31).

## Date

2026-07-07

## Context

ADR-028 self-hosted four font families with CJK fallback, but the subset policy
excluded Cyrillic from every non-CJK font. The app's UI is bilingual (Russian +
English): labels, navigation, descriptions, set titles, furigana glosses, and
onboarding all render Russian text. With Cyrillic absent from the subset:

- **Cormorant Garamond** headings containing Russian words (e.g. set titles,
  level names) fell back to the system serif. On Xiaomi MIUI/HyperOS that is a
  sans-serif (MiSans), breaking the visual contract that all headings use the
  display serif.
- **DM Mono** — the body/label/UI mono — has **no Cyrillic glyphs at all** in
  its source. Every Russian UI string rendered through a system mono fallback,
  which on most Android devices is a sans-serif. The interface lost its
  typewriter voice on roughly half of its text.

Both source fonts already ship the glyphs (Cormorant Garamond's variable TTF
covers Cyrillic; IBM Plex Mono was designed for Latin, Cyrillic, Greek, and
Vietnamese). The bug was a subset policy decision, not a font coverage gap.

A secondary issue: `font-display: swap` on Noto JP meant the browser could
paint a Chinese system CJK glyph (Xiaomi's MiSans CJK SC) before Noto JP
arrived — a brief but visible "wrong glyph" flash on first load, even though
the cache is correct afterwards.

## Decision

1. **Add Cyrillic to the Cormorant Garamond subset.** Extend the
   `--unicodes` range in `scripts/subset_fonts.py` with
   `U+0400-052F,U+1C80-1C88,U+20B4,U+2DE0-2DFF,U+A640-A69F`
   (Cyrillic + Supplement + Extended-A/B/C + Hryvnia).

2. **Replace DM Mono with IBM Plex Mono.** DM Mono cannot be salvaged for
   Cyrillic — it has no Cyrillic source glyphs to subset. IBM Plex Mono ships
   Latin, Latin-Extended (Vietnamese diacritics), and full Cyrillic in every
   weight (300 Light, 400 Regular, 500 Medium, + italic 400), matching the
   existing weight axis one-for-one. Subset with the same Latin+Cyrillic+Latin-
   Extended range as Cormorant.

3. **Noto JP: `font-display: block`.** Switch from `swap` to `block` on both
   Noto Sans JP and Noto Serif JP `@font-face`. `block` gives the font a short
   invisible-fallback window (≈3s) before falling back, so a Chinese system
   CJK glyph is never painted as a transient on first paint. Cormorant and IBM
   Plex Mono keep `swap`: Latin/Cyrillic system fallbacks are visually close
   enough that a brief swap is preferable to invisible text.

4. **Noto JP: expand `unicode-range`.** Add `U+3400-4DBF` (CJK Unified
   Ideographs Extension A) and `U+F900-FAFF` (CJK Compatibility Ideographs) so
   the @font-face hint matches the corpus extraction's `KANJI_RANGES`. The
   subset itself only includes kanji present in `cdn/` content (so a rare
   Extension-A kanji absent from the corpus still falls back), but the
   `unicode-range` declaration now matches what the subset can serve.

5. **Noto JP: `<link rel="preload">`** for both woff2 in `index.html` so the
   browser starts the fetch in parallel with the WASM bootstrap instead of
   waiting for the first paint that references a CJK glyph.

6. **Set card title wraps.** The `truncate` class on `set_card.rs` Heading was
   clipping Cyrillic set titles mid-letter (no word boundary, hard cut with
   ellipsis). Replaced with `break-words` so long titles wrap inside the card
   instead of disappearing.

### Subset script resilience

`scripts/subset_fonts.py` now degrades gracefully when a source font cannot be
downloaded (transient network failure, GitHub release-asset block): it preserves
the existing woff2 for that logical name and continues with the rest, instead
of `unlink`-ing every woff2 up front and aborting mid-run. The CI path (full
network access) is unaffected; the local-dev path no longer loses CJK coverage
on a single failed fetch.

## Alternatives considered

1. **Keep DM Mono, accept Cyrillic fallback.** Rejected: half the UI renders in
   a system sans-serif, breaking the typewriter aesthetic that is the app's
   signature visual identity. Not fixable by subsetting — the glyphs do not
   exist in the source.

2. **Add a separate Cyrillic-only font.** Rejected: a third Latin-family
   introduces another fallback hop and another CDN payload, with no benefit
   over using a font that already covers both ranges (IBM Plex Mono does).

3. **Switch to a different Latin mono with Cyrillic (JetBrains Mono, Fira
   Mono, Roboto Mono).** Considered. IBM Plex Mono was chosen for its
   availability as static-weight TTFs (matches the existing `latin_static`
   source shape), per-weight subsetting (no variable-italic complications),
   and visual character closer to DM Mono's typewriter feel than the more
   geometric JetBrains/Fira alternatives.

4. **`font-display: optional` instead of `block`.** Rejected: `optional`
   abandons the fetch if it does not finish in the brief block window, leaving
   the user on the Chinese fallback forever for that session. `block` always
   recovers to Noto JP once loaded.

## Consequences

- **Payload:** Cormorant subsets grow (~67 KB → ~136 KB normal, ~52 KB → ~97 KB
  italic) due to Cyrillic glyphs. IBM Plex Mono subsets are ~27–31 KB each
  vs DM Mono's ~15 KB (Cyrillic + Latin-Extended add bulk). Total fonts
  payload rises by roughly ~250 KB across 6 Latin/Cyrillic files; still well
  under the NFR-1 ≤6 MB ceiling dominated by the two ~1.4–2 MB Noto JP files.
- **Local dev:** `font_face.rs` FACES table updated synchronously (IBM Plex
  Mono entries + `LATIN_CYRILLIC_RANGE`), so the runtime-injected `@font-face`
  matches the hardcoded `input.css` block. Production still relies on
  `input.css` only (CI checkout has empty `cdn/fonts/`, see ADR-028 / PR #240);
  the two layers stay consistent so neither leaks DM Mono in any environment.
- **Regression guards:** `subset_fonts.py` now asserts Cyrillic coverage
  (`CYRILLIC_MUST_HAVE`, the full Russian alphabet) on `ibm-plex-mono-400-*`
  after every subset run, alongside the existing CJK `MUST_HAVE` check on
  `noto-serif-jp-400-*`. A future subset range regression aborts the run.
- **Deploy order matters:** the new woff2 files must exist on the CDN before
  the branch updating `input.css` hashes is merged — otherwise production
  fetches the new URLs and gets 404 (CJK/Latin falls back to system fonts
  until deploy completes). Same constraint as ADR-028.

## References

- ADR-028 — Self-hosted typography with CJK fallback (the foundation this
  builds on).
- PR #240 — Hardcoded `@font-face` in `input.css` (production source of truth,
  since `cdn/fonts/` is gitignored and CI checkout has no woff2 to enumerate).
- PR #221 — Original self-hosted typography + subset pipeline.
