#!/usr/bin/env python3
"""Enrich cdn/dictionary/kanji.json with per-reading frequency data.

Adds a `reading_frequencies` map to each kanji entry, keyed by reading
exactly as it appears in on_readings/kun_readings, with the count of words
in the JmdictFurigana corpus that demonstrate this reading for this kanji.

Run order: `audit_kanji_readings.py --apply` MUST run first (it removes dead
readings). This script only adds the frequency map; it does not touch
on_readings, kun_readings, used_in, popular_words, radicals, or descriptions.

Usage:
    python scripts/enrich_kanji_reading_frequencies.py             # dry-run + validate
    python scripts/enrich_kanji_reading_frequencies.py --apply     # write kanji.json
    python scripts/enrich_kanji_reading_frequencies.py --validate  # validate only
"""

import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

import audit_kanji_readings as audit  # noqa: E402

FURIGANA_PATH = audit.FURIGANA_PATH
KANJI_PATH = ROOT / "cdn" / "dictionary" / "kanji.json"

RARE_THRESHOLD = 5

# (kanji, reading, type, expected_min_freq, expected_rare)
# Anchors derived from analyze_reading_frequencies.py spotlight output.
SPOTLIGHT: list[tuple[str, str, str, int, bool]] = [
    ("生", "セイ", "on", 1000, False),
    ("生", "なる", "kun", 10, False),
    ("中", "うち", "kun", 1, False),
    ("子", "ね", "kun", 1, False),
    ("上", "ショウ", "on", 1, False),
    ("厳", "おごそか", "kun", 0, True),
    ("厳", "いかめしい", "kun", 0, True),
]


def compute_frequencies(
    kanji_list: list[dict],
    furigana_lookup: dict[str, list[tuple[str, str]]],
    words_by_kanji: dict[str, list[str]],
) -> dict[str, dict[str, int]]:
    """Return { kanji_char: { reading: frequency } }."""
    result: dict[str, dict[str, int]] = {}
    for entry in kanji_list:
        kanji = entry["kanji"]
        corpus_words = words_by_kanji.get(kanji, [])
        freqs: dict[str, int] = {}

        for r in entry["on_readings"]:
            r_hira = audit.kata_to_hira(r)
            freq = sum(
                1
                for w in corpus_words
                if audit.word_covers_reading(w, kanji, r_hira, "on", furigana_lookup)
            )
            freqs[r] = freq

        for r in entry["kun_readings"]:
            r_hira = audit.kata_to_hira(r)
            freq = sum(
                1
                for w in corpus_words
                if audit.word_covers_reading(w, kanji, r_hira, "kun", furigana_lookup)
            )
            freqs[r] = freq

        result[kanji] = freqs
    return result


def build_words_by_kanji(
    furigana_lookup: dict[str, list[tuple[str, str]]],
) -> dict[str, list[str]]:
    """Index: kanji_char -> [words containing it]."""
    out: dict[str, list[str]] = defaultdict(list)
    for word in furigana_lookup:
        for ch in set(word):
            if 0x4E00 <= ord(ch) <= 0x9FFF:
                out[ch].append(word)
    return out


def validate_coverage(
    kanji_list: list[dict], freq_map: dict[str, dict[str, int]]
) -> list[str]:
    """Coverage post-condition: every reading has a frequency entry."""
    errors: list[str] = []
    for entry in kanji_list:
        kanji = entry["kanji"]
        all_readings = set(entry["on_readings"]) | set(entry["kun_readings"])
        freqs = freq_map.get(kanji, {})
        missing = all_readings - set(freqs.keys())
        extra = set(freqs.keys()) - all_readings
        if missing:
            errors.append(
                f"  {kanji}: missing frequencies for {sorted(missing)}"
            )
        if extra:
            errors.append(
                f"  {kanji}: frequency map has stale keys not in readings: {sorted(extra)}"
            )
    return errors


def validate_spotlight(freq_map: dict[str, dict[str, int]]) -> list[str]:
    """Spotlight anchors: known kanji/reading frequency bounds."""
    errors: list[str] = []
    for kanji, reading, _rtype, min_freq, expected_rare in SPOTLIGHT:
        freq = freq_map.get(kanji, {}).get(reading)
        if freq is None:
            errors.append(f"  spotlight {kanji}/{reading}: not in frequency map")
            continue
        if freq < min_freq:
            errors.append(
                f"  spotlight {kanji}/{reading}: freq={freq} < expected min {min_freq}"
            )
        is_rare = freq <= RARE_THRESHOLD
        if is_rare != expected_rare:
            errors.append(
                f"  spotlight {kanji}/{reading}: rare={is_rare}, expected={expected_rare} "
                f"(freq={freq}, threshold={RARE_THRESHOLD})"
            )
    return errors


def sanity_overcount(
    kanji_list: list[dict],
    freq_map: dict[str, dict[str, int]],
    words_by_kanji: dict[str, list[str]],
) -> list[str]:
    """Warn if sum(freq) exceeds corpus_words for a kanji.

    One corpus word can contribute to multiple readings of the same kanji only
    if it contains the kanji multiple times (rare). Anything beyond that hints
    at prefix-match inflation from word_covers_reading.
    """
    warnings: list[str] = []
    for entry in kanji_list:
        kanji = entry["kanji"]
        freqs = freq_map.get(kanji, {})
        total = sum(freqs.values())
        corpus_n = len(words_by_kanji.get(kanji, []))
        # Allow 2x headroom: a word with this kanji twice can cover 2 readings.
        if total > corpus_n * 2 and corpus_n > 0:
            warnings.append(
                f"  {kanji}: sum(freq)={total} > 2x corpus_words={corpus_n} "
                f"(possible prefix-match inflation)"
            )
    return warnings


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="Write enriched kanji.json")
    parser.add_argument(
        "--validate", action="store_true", help="Run validation checks only"
    )
    args = parser.parse_args()

    if args.validate and args.apply:
        print("error: --validate and --apply are mutually exclusive", file=sys.stderr)
        return 2

    print("=== enrich_kanji_reading_frequencies ===")
    print(f"Mode: {'APPLY' if args.apply else 'VALIDATE' if args.validate else 'DRY-RUN'}")
    print()

    print("Loading kanji.json...")
    with open(KANJI_PATH, "r", encoding="utf-8") as f:
        kanji_data = json.load(f)
    kanji_list = kanji_data["kanji"]
    print(f"  Total kanji: {len(kanji_list)}")

    print("Building furigana lookup (this takes a few seconds)...")
    furigana_lookup = audit.build_furigana_lookup(FURIGANA_PATH)
    print(f"  Words in furigana lookup: {len(furigana_lookup)}")

    print("Building kanji -> words index...")
    words_by_kanji = build_words_by_kanji(furigana_lookup)
    print(f"  Kanji chars in corpus: {len(words_by_kanji)}")

    print("Computing per-reading frequencies...")
    freq_map = compute_frequencies(kanji_list, furigana_lookup, words_by_kanji)
    total_readings = sum(len(f) for f in freq_map.values())
    rare_count = sum(
        1 for f in freq_map.values() for freq in f.values() if freq <= RARE_THRESHOLD
    )
    print(f"  Total readings covered: {total_readings}")
    print(
        f"  Rare readings (freq <= {RARE_THRESHOLD}): {rare_count} "
        f"({100*rare_count/total_readings:.1f}%)"
    )

    print()
    print("Coverage check: every reading has a frequency entry...")
    coverage_errors = validate_coverage(kanji_list, freq_map)
    if coverage_errors:
        print("  FAIL:")
        for e in coverage_errors[:20]:
            print(e)
        if len(coverage_errors) > 20:
            print(f"  ... and {len(coverage_errors) - 20} more")
        return 1
    print("  OK")

    print()
    print("Spotlight validation (known frequency anchors)...")
    spotlight_errors = validate_spotlight(freq_map)
    if spotlight_errors:
        print("  FAIL:")
        for e in spotlight_errors:
            print(e)
        return 1
    print("  OK")

    print()
    print("Sanity: over-counting check (prefix-match inflation)...")
    warnings = sanity_overcount(kanji_list, freq_map, words_by_kanji)
    if warnings:
        print(f"  WARN ({len(warnings)} kanji):")
        for w in warnings[:10]:
            print(w)
        if len(warnings) > 10:
            print(f"  ... and {len(warnings) - 10} more")
    else:
        print("  OK (no warnings)")

    if args.validate:
        print()
        print("Validation passed.")
        return 0

    if not args.apply:
        print()
        print("Dry-run complete. Re-run with --apply to write kanji.json.")
        return 0

    print()
    print("Applying enrichment to kanji.json...")
    size_before = KANJI_PATH.stat().st_size
    for entry in kanji_list:
        kanji = entry["kanji"]
        entry["reading_frequencies"] = freq_map[kanji]

    tmp_path = KANJI_PATH.with_suffix(".json.tmp")
    # Serialize with indent=2 for readability, then collapse reading_frequencies
    # dicts into single lines — the rest of the entry keeps multi-line format,
    # but the freq map is dense and looks better inline. Saves ~600KB.
    json_str = json.dumps(kanji_data, ensure_ascii=False, indent=2)
    json_str = _collapse_frequency_blocks(json_str)
    tmp_path.write_text(json_str, encoding="utf-8")
    tmp_path.replace(KANJI_PATH)
    size_after = KANJI_PATH.stat().st_size
    print(f"  Size: {size_before} -> {size_after} bytes (+{size_after - size_before})")
    print("Done.")
    return 0


_FREQ_BLOCK_RE = re.compile(r'"reading_frequencies":\s*\{[^}]*\}', re.DOTALL)


def _collapse_frequency_blocks(json_str: str) -> str:
    """Inline each `reading_frequencies: { ... }` block to a single line."""

    def _collapse(match: re.Match[str]) -> str:
        block = match.group(0)
        pairs = re.findall(r'"([^"]+)":\s*(\d+)', block)
        body = ", ".join(f'"{k}": {v}' for k, v in pairs)
        return f'"reading_frequencies": {{ {body} }}'

    return _FREQ_BLOCK_RE.sub(_collapse, json_str)


if __name__ == "__main__":
    sys.exit(main())
