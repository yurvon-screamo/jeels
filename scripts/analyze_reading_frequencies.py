#!/usr/bin/env python3
"""Exploratory: count per-reading frequency against the full JmdictFurigana corpus.

Reuses matching helpers from scripts/audit_kanji_readings.py. Reproducible basis
for the T=5 'rare reading' threshold decision.

Run: `python scripts/analyze_reading_frequencies.py`.
"""

import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

import audit_kanji_readings as audit  # noqa: E402

FURIGANA_PATH = audit.FURIGANA_PATH
KANJI_PATH = ROOT / "cdn" / "dictionary" / "kanji.json"


def main() -> None:
    print("Loading kanji.json...")
    with open(KANJI_PATH, "r", encoding="utf-8") as f:
        kanji_data = json.load(f)
    kanji_list = kanji_data["kanji"]
    print(f"  Total kanji: {len(kanji_list)}")

    print("Building furigana lookup (this takes a few seconds)...")
    furigana_lookup = audit.build_furigana_lookup(FURIGANA_PATH)
    print(f"  Words in furigana lookup: {len(furigana_lookup)}")

    print("Building kanji -> words index...")
    words_by_kanji: dict[str, list[str]] = defaultdict(list)
    for word in furigana_lookup:
        for ch in set(word):
            if 0x4E00 <= ord(ch) <= 0x9FFF:
                words_by_kanji[ch].append(word)
    print(f"  Kanji chars referenced in furigana corpus: {len(words_by_kanji)}")

    stats: dict[tuple[str, str, str], int] = {}
    per_kanji: dict[str, list[tuple[str, str, int]]] = defaultdict(list)

    total_on = 0
    total_kun = 0
    for entry in kanji_list:
        kanji = entry["kanji"]
        corpus_words = words_by_kanji.get(kanji, [])

        for r in entry["on_readings"]:
            r_hira = audit.kata_to_hira(r)
            freq = sum(
                1
                for w in corpus_words
                if audit.word_covers_reading(w, kanji, r_hira, "on", furigana_lookup)
            )
            stats[(kanji, r, "on")] = freq
            per_kanji[kanji].append((r, "on", freq))
            total_on += 1

        for r in entry["kun_readings"]:
            r_hira = audit.kata_to_hira(r)
            freq = sum(
                1
                for w in corpus_words
                if audit.word_covers_reading(w, kanji, r_hira, "kun", furigana_lookup)
            )
            stats[(kanji, r, "kun")] = freq
            per_kanji[kanji].append((r, "kun", freq))
            total_kun += 1

    total = total_on + total_kun
    print()
    print("=== Overall (corpus = full JmdictFurigana) ===")
    print(f"  Total readings analysed: {total}  (on={total_on}, kun={total_kun})")

    bucket = Counter()
    for (_k, _r, _t), freq in stats.items():
        if freq == 0:
            bucket["0"] += 1
        elif freq <= 1:
            bucket["1"] += 1
        elif freq <= 5:
            bucket["2-5"] += 1
        elif freq <= 20:
            bucket["6-20"] += 1
        elif freq <= 100:
            bucket["21-100"] += 1
        else:
            bucket["101+"] += 1

    print()
    print("=== Frequency distribution ===")
    for label in ["0", "1", "2-5", "6-20", "21-100", "101+"]:
        n = bucket[label]
        pct = 100.0 * n / total if total else 0.0
        print(f"  freq={label:>7}: {n:>5}  ({pct:5.1f}%)")

    print()
    print("=== Threshold scenarios (reading is 'rare' if freq <= T) ===")
    for t in [0, 1, 5, 10, 20]:
        rare = sum(1 for f in stats.values() if f <= t)
        kept = total - rare
        kanji_all_rare = 0
        kanji_partial = 0
        for kanji, readings in per_kanji.items():
            if not readings:
                continue
            n_rare = sum(1 for (_r, _t, f) in readings if f <= t)
            if n_rare == len(readings):
                kanji_all_rare += 1
            elif n_rare > 0:
                kanji_partial += 1
        print(
            f"  T={t:>3}: rare={rare:>5} ({100*rare/total:5.1f}%), "
            f"kept={kept:>5}; kanji all-rare={kanji_all_rare}, "
            f"kanji partial={kanji_partial}"
        )

    print()
    print("=== Spotlight: readings breakdown for known tricky kanji ===")
    for spotlight in ["生", "上", "下", "分", "日", "人", "気", "中", "大", "子", "行", "出", "厳", "咽"]:
        readings = per_kanji.get(spotlight, [])
        if not readings:
            continue
        readings_sorted = sorted(readings, key=lambda x: -x[2])
        parts = ", ".join(f"{r}({ty},f={f})" for (r, ty, f) in readings_sorted)
        print(f"  {spotlight}: {parts}")


if __name__ == "__main__":
    main()
