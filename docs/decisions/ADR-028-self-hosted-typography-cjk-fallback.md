# ADR-028: Self-hosted typography with CJK fallback

## Status

Accepted

> **Updated by [ADR-030](ADR-030-cyrillic-typography-cormorant-subset-ibm-plex-mono.md)**
> (2026-07-07): DM Mono replaced with IBM Plex Mono (Cyrillic coverage);
> Cormorant subset extended with Cyrillic; Noto JP switched to
> `font-display: block`. The Cyrillic-less DM Mono rows in the table below
> are historical — see ADR-030 for the current font set.

> **Updated (2026-07-22): offline-first bundling implemented.** Alternative #5
> (bundle fonts for true offline first launch) is now in effect. trunk cannot
> accept assets outside the `index.html` directory (trunk-rs/trunk#1045), so
> instead of a direct cross-dir `copy-dir` the canonical `cdn/fonts/` is staged
> into `origa_ui/public/fonts/` by `build.rs::stage_fonts_for_bundling`
> (gitignored build mirror, same pattern as `public/dictionaries/unidic/`), and
> the existing trunk `copy-dir href="public"` ships it at `dist/fonts/`.
> `font_face.rs` is the single canonical `@font-face` source (the hardcoded
> rules previously in `input.css` were dead — the runtime `<style>` won the
> cascade — and are removed); it routes the URL per build flavour: Tauri →
> `/fonts/<file>` (local origin, CSP `font-src 'self'` already permits it, no
> asset-protocol needed), web → `cdn_url("/fonts/<file>")`. Per-face
> `font-display` (CJK `block`, Latin/Cyr `swap`) lives in `FaceSpec.display`,
> fixing a dormant regression where the runtime path forced `swap` on CJK and
> would have flashed Chinese glyphs on Xiaomi. Trade-off accepted: every Tauri
> installer (Win/Linux/macOS/Android) grows by ~3.5 MB (2 Noto JP dominate);
> justified by correct CJK on the very first launch and offline operation. Web
> builds are unchanged.

> **Updated (2026-07-31): CJK unified on Noto Sans JP; Noto Serif JP removed.**
> Both `--font-serif` and `--font-mono` now fall through to a single CJK family
> — Noto Sans JP (Gothic) — so kanji and kana never mix Mincho/Gothic within a
> word (the split previously broke Japanese typography on Xiaomi). Noto Serif JP
> is dropped from `font_face.rs`, `subset_fonts.py`, and `end2end/cdn-manifest.txt`.
> The Noto Serif JP rows in the font-set table below are historical — see
> DESIGN.md §Система шрифтов for the current font set.

## Date

2026-07-03

## Context

Users reported that on a Xiaomi tablet (MIUI/HyperOS) all CJK characters —
kana and kanji — render with **Chinese** glyphs instead of Japanese. Root cause:

- `origa_ui/input.css:1` loaded Cormorant Garamond + DM Mono via a Google Fonts
  CSS `@import` (online-only). Neither font contains CJK glyphs.
- The font stacks were `"Cormorant Garamond", serif` and `"DM Mono", monospace`
  — no CJK family, so the browser fell back to the system CJK font.
- On Xiaomi/Android the system CJK serif is Chinese (MiSans CJK SC). Shared
  han/kanji characters (食, 海, 語, 難, …) have different stroke forms and
  proportions in Chinese vs Japanese — visually wrong for a Japanese-learning
  app. Kana rendered through whatever the system picked.

Additionally the Google Fonts `@import` made the desktop/mobile app depend on
the internet and on a third-party CDN: Tauri builds must work offline after a
first fetch, and Google Fonts is slow or blocked in some regions.

## Decision

Self-host all four font families on the project CDN (`<cdn>/fonts/`) and inject
`@font-face` at runtime so the same `ORIGA_CDN_BASE_URL` that serves every
other asset also serves fonts (single source of truth; E2E runs against the
local CDN).

```css
--font-serif: "Cormorant Garamond", "Noto Serif JP", serif;
--font-mono: "DM Mono", "Noto Sans JP", monospace;
```

The browser selects a font per character within the stack. Each `@font-face`
declares a `unicode-range`, so Latin text resolves to Cormorant/DM Mono and CJK
resolves to the matching Noto JP family — yielding correct Japanese glyph forms
on every platform.

### Font set and subsetting

| Family | Source | Weights | Subset |
| ------ | ------ | ------- | ------ |
| Cormorant Garamond | google/fonts (variable TTF) | 300–700 + italic | Latin, Latin-1, punctuation |
| DM Mono | google/fonts (static TTF) | 300, 400, 500 + italic 400 | Latin, Latin-1, punctuation |
| Noto Serif JP | notofonts Subset OTF (Sans2.004/Serif2.003) | 400 (Regular) | app CJK corpus: kana + punctuation + kanji present in cdn/ content |
| Noto Sans JP | notofonts Subset OTF | 400 (Regular) | same CJK corpus |

Noto ships Regular (400) only initially; Medium (500) is a documented follow-up
if visual review finds CJK headings too light next to Cormorant 500.

`scripts/subset_fonts.py` regenerates the woff2 set: it scans `cdn/` JSON for the
real CJK vocabulary (excluding `dictionary/radicals.json`, whose radical→kanji
reference list would otherwise pull in ~12k unrendered ideographs), subsets with
`pyftsubset`, and writes content-hash-versioned filenames.

### Cache-bust via content-hash names

Each output is `<logical>-<sha8>.woff2`. `origa_ui/build.rs` enumerates
`cdn/fonts/` at build time and emits a `FONT_FILES` table the runtime CSS builder
consumes. A regenerated subset changes the hash → new URL, so an immutable
(`max-age=31536000`) CDN edge cache can never serve a stale font. This is the
PR #182 (edge-cache poisoning) mitigation, and it is strictly safer than the
stable-name + manual `deploy_cdn.py --force` pattern used for ML models.

## Alternatives considered

1. **Google Fonts CDN (`@import`/`<link>`).** Rejected: online-only; Tauri
   desktop/mobile must work offline after first fetch, and the CDN is slow or
   blocked in some regions. Also still had no CJK family.
2. **Hardcoded `@font-face` in `input.css`.** Rejected: the font URL would be a
   literal `https://s3.origa.uwuwu.net`, breaking the E2E isolation model (E2E
   serves a **local** CDN at `http://localhost:8080`) and duplicating the
   `ORIGA_CDN_BASE_URL` constant instead of keeping a single source of truth.
3. **trunk / build.rs build-time CSS generation.** Considered: emit the
   `@font-face` block as a build artifact from `env!("ORIGA_CDN_BASE_URL")`.
   This would avoid the runtime CSS string and the FOUT window before the
   injected `<style>` is parsed. Rejected in favour of the runtime builder for
   simplicity: the build step already emits the `FONT_FILES` table, the runtime
   composes the CSS from it + `cdn_url()`, and graceful degradation (empty
   `FONT_FILES` → no `@font-face`, system fallback) is trivial. The FOUT is
   masked by the app loading overlay and the immutable HTTP cache after first
   load.
4. **unicode-range splitting of the full Noto JP.** Rejected: more files and
   more moving parts; the corpus subset already keeps each Noto file ~1.4–2 MB.
5. **Bundle fonts in `tauri/resources/fonts/`.** Rejected as the default: bloats
   every installer/APK by ~3–5 MB. Retained as the follow-up path to true
   offline-first (offline on the very first launch).

## Consequences

- **Payload:** ~3.5 MB total across 8 woff2 files (2 Noto JP dominate at
  ~1.4/1.9 MB; Cormorant + DM Mono are <300 KB combined). Within the NFR-1
  target of ≤6 MB.
- **Offline:** after the first online launch the WebView HTTP cache holds the
  immutable woff2; subsequent launches render CJK correctly offline. The very
  first launch still needs internet to fetch the fonts (mitigation: future
  bundling in `tauri/resources/fonts/`).
- **FOUT:** before Noto JP is fetched/cached, CJK briefly uses the system
  fallback (Chinese forms on Xiaomi). `font-display: swap` plus the app loading
  overlay keep this unobtrusive; it disappears once cached.
- **Maintenance:** when the kanji corpus grows, rerun
  `uv run --with "fonttools[woff]" scripts/subset_fonts.py`, then
  `python scripts/deploy_cdn.py`. The hash-renamed files and the regenerated
  `end2end/cdn-manifest.txt` FONTS block keep CDN and CI in sync. **Deploy to
  S3 before merging the branch that updates `cdn-manifest.txt`**, otherwise CI
  cannot download the fonts and E2E silently falls back to system glyphs.
- **Subset completeness:** the corpus is the app's actual vocabulary (kanji
  dictionary + word sets + phrases + grammar + pitch). A kanji outside the
  corpus (e.g. an OCR result for rare text) falls back to the system font — no
  worse than today, and regenerable on demand.
