"""Tiered Cache-Control policy for CDN objects.

Files are split into three categories by how often they change:

- TRULY-STATIC (immutable, 1 year): ML models (ndlocr/whisper), kanji stroke
  art, recorded audio, the compiled lindera system dictionary, and self-hosted
  web fonts. These move only on rare deliberate events — a model retrain, a
  lindera version bump, a font swap — never on a content release. Fonts are
  content-addressed (the file hash is in the name), so a changed glyph ships
  under a new URL and the immutable copy cannot poison the edge cache.
- RELEASE-UPDATED (5 min, must-revalidate): content JSON shipped or corrected
  every release (grammar, phrases, dictionary chunks, JLPT sets, pitch). This
  is the fix for the PR #182 edge-cache poisoning: S3 updated grammar.json but
  the CDN edge kept serving its year-long immutable copy until the cache was
  flushed by hand.
- ALWAYS-FRESH (no-cache): manifest.json — the client's change-detection
  beacon, re-fetched every session.

Any path matching no rule falls back to the conservative release-updated
policy: a 5-min 304 revalidation is cheap when unchanged and bounds staleness
to 5 min when it does change — strictly safer than immutable for the unknown.
"""

from __future__ import annotations

from typing import Final

IMMUTABLE: Final[str] = "public, max-age=31536000, immutable"
RELEASE_UPDATED: Final[str] = "public, max-age=300, must-revalidate"
NO_CACHE: Final[str] = "no-cache"

# A rule ending in "/" matches any path under that directory (startswith).
# A rule without "/" matches one exact path. Sets are disjoint by construction
# (no path matches two categories), so precedence below only orders the lookup.
_IMMUTABLE_RULES: Final[frozenset[str]] = frozenset(
    {
        "dictionaries/",
        "fonts/",
        "kanji_animations/",
        "kanji_frames/",
        # JLPT bundles (exact paths, no trailing /)
        "kanji_animations_n5.json",
        "kanji_animations_n4.json",
        "kanji_animations_n3.json",
        "kanji_animations_n2.json",
        "kanji_animations_n1.json",
        "kanji_frames_n5.json",
        "kanji_frames_n4.json",
        "kanji_frames_n3.json",
        "kanji_frames_n2.json",
        "kanji_frames_n1.json",
        "ndlocr/",
        "phrases/audio/",
        "whisper/",
    }
)

_RELEASE_UPDATED_RULES: Final[frozenset[str]] = frozenset(
    {
        "dictionary/",
        "grammar/",
        "phrases/data/",
        "phrases/phrase_index.json",
        "pitch/",
        "well_known_set/",
    }
)

_NO_CACHE_RULES: Final[frozenset[str]] = frozenset({"manifest.json"})


def _matches(path: str, rules: frozenset[str]) -> bool:
    for rule in rules:
        if rule.endswith("/"):
            if path.startswith(rule):
                return True
        elif path == rule:
            return True
    return False


def cache_control_for(path: str) -> str:
    if _matches(path, _NO_CACHE_RULES):
        return NO_CACHE
    if _matches(path, _IMMUTABLE_RULES):
        return IMMUTABLE
    if _matches(path, _RELEASE_UPDATED_RULES):
        return RELEASE_UPDATED
    return RELEASE_UPDATED
