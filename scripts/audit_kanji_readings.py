#!/usr/bin/env python3
"""Kanji readings audit: remove dead readings, fix popular_words, ensure chunk coverage."""

import json
import glob
import argparse
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KANJI_PATH = ROOT / "cdn" / "dictionary" / "kanji.json"
FURIGANA_PATH = ROOT / "cdn" / "dictionaries" / "JmdictFurigana.txt"
CHUNK_GLOB = str(ROOT / "cdn" / "dictionary" / "chunk_*.json")
JLPT_GLOB = str(ROOT / "cdn" / "well_known_set" / "jlpt_n*.json")
REPORT_PATH = ROOT / "scripts" / "kanji_audit_changes.json"

KATAKANA_TO_HIRAGANA = str.maketrans(
    "ァアイウエオカキクケコサシスセソタチツテトナニヌネノ"
    "ハヒフヘホマミムメモヤユヨラリルレロワヲン"
    "ッャュョヮ"
    "ガギグゲゴザジズゼゾダヂヅデドバビブベボパピプペポ"
    "ヴ",
    "ぁあいうえおかきくけこさしすせそたちつてとなにぬねの"
    "はひふへほまみむめもやゆよらりるれろわをん"
    "っゃゅょゎ"
    "がぎぐげござじずぜぞだぢづでどばびぶべぼぱぴぷぺぽ"
    "ゔ",
)


def kata_to_hira(s: str) -> str:
    return s.translate(KATAKANA_TO_HIRAGANA)


def is_kanji(ch: str) -> bool:
    cp = ord(ch)
    return 0x4E00 <= cp <= 0x9FFF or 0x3400 <= cp <= 0x4DBF or 0xF900 <= cp <= 0xFAFF


def parse_furigana_spans(word: str, spans_str: str) -> list[tuple[int, str]]:
    """Return list of (kanji_position_in_word, reading_in_hiragana)."""
    results = []
    for span in spans_str.split(";"):
        span = span.strip()
        if not span:
            continue
        pos_str, reading = span.split(":", 1)
        if "-" in pos_str:
            start_s, end_s = pos_str.split("-", 1)
            start, end = int(start_s), int(end_s)
        else:
            start = end = int(pos_str)
        reading_hira = kata_to_hira(reading)
        for pos in range(start, end + 1):
            if pos < len(word) and is_kanji(word[pos]):
                results.append((pos, reading_hira))
    return results


def build_alive_readings(path: Path) -> dict[str, set[str]]:
    """Phase 1: Parse JmdictFurigana.txt → { kanji_char: set of hiragana readings }."""
    alive: dict[str, set[str]] = defaultdict(set)
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n\r")
            if not line or "|" not in line:
                continue
            parts = line.split("|")
            if len(parts) < 3:
                continue
            word = parts[0]
            spans_str = parts[2]
            if not spans_str:
                continue
            for _pos, reading in parse_furigana_spans(word, spans_str):
                alive[word[_pos]].add(reading)
    return alive


def load_chunks() -> dict[str, dict]:
    """Load all chunk files → { word: { english_translation, russian_translation } }."""
    chunks: dict[str, dict] = {}
    for f in sorted(glob.glob(CHUNK_GLOB)):
        with open(f, "r", encoding="utf-8") as fh:
            data = json.load(fh)
        for word, entry in data.items():
            chunks[word] = entry
    return chunks


def load_jlpt_lists() -> dict[str, list[str]]:
    """Load JLPT lists → { 'N5': [words...], ... }."""
    jlpt: dict[str, list[str]] = {}
    for f in sorted(glob.glob(JLPT_GLOB)):
        with open(f, "r", encoding="utf-8") as fh:
            data = json.load(fh)
        level = data.get("level", "")
        words = data.get("words", [])
        if level and words:
            jlpt[level] = words
    return jlpt


def load_jlpt_word_set() -> set[str]:
    """Load all JLPT words into a single set for fast membership testing."""
    words: set[str] = set()
    for f in sorted(glob.glob(JLPT_GLOB)):
        with open(f, "r", encoding="utf-8") as fh:
            data = json.load(fh)
        words.update(data.get("words", []))
    return words


JLPT_ORDER = ["N5", "N4", "N3", "N2", "N1"]


def jlpt_priority(level: str) -> int:
    return JLPT_ORDER.index(level) if level in JLPT_ORDER else 99


def reading_matches_on(on_hira: str, alive_set: set[str]) -> bool:
    """Check if an ON reading (converted to hiragana) matches any alive reading."""
    for alive in alive_set:
        if alive == on_hira:
            return True
        if alive.startswith(on_hira) or on_hira.startswith(alive):
            return True
    return False


def reading_matches_kun(kun: str, alive_set: set[str]) -> bool:
    """Check if a KUN reading matches any alive reading."""
    prefix = kun.split(".")[0] if "." in kun else kun
    for alive in alive_set:
        if alive.startswith(prefix) or prefix.startswith(alive):
            return True
    return False


def word_covers_reading(
    word: str, kanji: str, reading_hira: str, reading_type: str, furigana_lookup: dict[str, list[tuple[str, str]]]
) -> bool:
    """Check if a word demonstrates the given reading for the kanji."""
    if kanji not in word:
        return False
    entries = furigana_lookup.get(word, [])
    if not entries:
        return False
    for _full_reading, spans_str in entries:
        for pos, span_reading in parse_furigana_spans(word, spans_str):
            if word[pos] == kanji:
                span_hira = span_reading
                if reading_type == "on":
                    if span_hira == reading_hira or span_hira.startswith(reading_hira) or reading_hira.startswith(span_hira):
                        return True
                else:
                    prefix = reading_hira.split(".")[0] if "." in reading_hira else reading_hira
                    if span_hira.startswith(prefix) or prefix.startswith(span_hira):
                        return True
    return False


def build_furigana_lookup(path: Path) -> dict[str, list[tuple[str, str]]]:
    """Build word → [(full_reading, spans_str), ...] from JmdictFurigana."""
    lookup: dict[str, list[tuple[str, str]]] = defaultdict(list)
    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n\r")
            if not line or "|" not in line:
                continue
            parts = line.split("|")
            if len(parts) < 3:
                continue
            word = parts[0]
            full_reading = parts[1]
            spans_str = parts[2]
            lookup[word].append((full_reading, spans_str))
    return lookup


def main():
    parser = argparse.ArgumentParser(description="Kanji readings audit")
    parser.add_argument("--apply", action="store_true", help="Apply changes to kanji.json")
    args = parser.parse_args()

    print("=== Kanji Readings Audit ===")
    print(f"Mode: {'APPLY' if args.apply else 'DRY-RUN'}")
    print()

    print("Phase 1: Building alive-readings index from JmdictFurigana...")
    alive_readings = build_alive_readings(FURIGANA_PATH)
    print(f"  Kanji chars with alive readings: {len(alive_readings)}")

    print("Phase 1b: Building furigana lookup for reading-coverage matching...")
    furigana_lookup = build_furigana_lookup(FURIGANA_PATH)
    print(f"  Words in furigana lookup: {len(furigana_lookup)}")

    print("Loading chunk files...")
    chunks = load_chunks()
    print(f"  Total chunk entries: {len(chunks)}")

    print("Loading JLPT word lists...")
    jlpt_lists = load_jlpt_lists()
    for lvl, words in sorted(jlpt_lists.items()):
        print(f"  {lvl}: {len(words)} words")

    jlpt_word_set = load_jlpt_word_set()
    print(f"  Total JLPT words: {len(jlpt_word_set)}")

    print()
    print("Phase 2: Loading kanji.json...")
    with open(KANJI_PATH, "r", encoding="utf-8") as f:
        kanji_data = json.load(f)
    kanji_list = kanji_data["kanji"]
    print(f"  Total kanji: {len(kanji_list)}")

    removed_readings_detail: list[dict] = []
    changes_by_kanji: dict[str, dict] = {}
    total_on_removed = 0
    total_kun_removed = 0
    total_words_removed = 0
    total_words_added = 0
    kanji_modified = 0

    new_kanji_list = []

    for entry in kanji_list:
        kanji = entry["kanji"]
        jlpt = entry["jlpt"]
        change: dict = {
            "jlpt": jlpt,
            "readings_removed": [],
            "words_removed": [],
            "words_added": [],
            "final_on_readings": list(entry["on_readings"]),
            "final_kun_readings": list(entry["kun_readings"]),
            "final_popular_words": list(entry.get("popular_words", [])),
        }

        alive_set = alive_readings.get(kanji, set())

        new_on = []
        for r in entry["on_readings"]:
            r_hira = kata_to_hira(r)
            if alive_set and reading_matches_on(r_hira, alive_set):
                new_on.append(r)
            else:
                change["readings_removed"].append({"reading": r, "type": "on"})
                removed_readings_detail.append({"kanji": kanji, "reading": r, "type": "on", "reason": "dead_in_jmdict"})
                total_on_removed += 1

        new_kun = []
        for r in entry["kun_readings"]:
            if alive_set and reading_matches_kun(r, alive_set):
                new_kun.append(r)
            else:
                change["readings_removed"].append({"reading": r, "type": "kun"})
                removed_readings_detail.append({"kanji": kanji, "reading": r, "type": "kun", "reason": "dead_in_jmdict"})
                total_kun_removed += 1

        # Backup alive readings for safety net (Phase 4b)
        alive_on_backup = list(new_on)
        alive_kun_backup = list(new_kun)

        # Phase 3a: Remove broken popular_words and non-JLPT words
        new_popular = []
        for w in entry.get("popular_words", []):
            if w in chunks and chunks[w].get("russian_translation", "").strip() and w in jlpt_word_set:
                new_popular.append(w)
            else:
                change["words_removed"].append(w)
                total_words_removed += 1

        # Phase 4: Smart word selection — JLPT-only, ensure every reading is covered
        all_remaining_readings = [(r, "on") for r in new_on] + [(r, "kun") for r in new_kun]

        # Step 1: Check which readings are already covered by existing popular_words
        covered_readings: set[int] = set()
        for i, (reading, rtype) in enumerate(all_remaining_readings):
            r_hira = kata_to_hira(reading) if rtype == "on" else reading
            if any(word_covers_reading(w, kanji, r_hira, rtype, furigana_lookup) for w in new_popular):
                covered_readings.add(i)

        # Step 2: Build JLPT-only candidate pool
        candidates: list[tuple[str, int]] = []
        for lvl in JLPT_ORDER:
            lvl_words = jlpt_lists.get(lvl, [])
            for w in lvl_words:
                if w in chunks and kanji in w and chunks[w].get("russian_translation", "").strip():
                    if w not in new_popular and w not in [c[0] for c in candidates]:
                        priority = abs(jlpt_priority(jlpt) - jlpt_priority(lvl))
                        candidates.append((w, priority))

        # Step 3: For each uncovered reading, find the best JLPT covering word
        uncovered = [(i, r, rt) for i, (r, rt) in enumerate(all_remaining_readings) if i not in covered_readings]
        for idx, reading, rtype in uncovered:
            r_hira = kata_to_hira(reading) if rtype == "on" else reading
            best_word = None
            best_priority = 999
            for w, p in candidates:
                if w in new_popular:
                    continue
                if word_covers_reading(w, kanji, r_hira, rtype, furigana_lookup):
                    if p < best_priority:
                        best_word = w
                        best_priority = p
            if best_word:
                new_popular.append(best_word)
                change["words_added"].append(best_word)
                total_words_added += 1
                covered_readings.add(idx)

        # Step 4: Remove readings that STILL have no covering JLPT word
        still_uncovered = [(i, r, rt) for i, (r, rt) in enumerate(all_remaining_readings) if i not in covered_readings]
        for idx, reading, rtype in still_uncovered:
            if rtype == "on":
                new_on.remove(reading)
                total_on_removed += 1
            else:
                new_kun.remove(reading)
                total_kun_removed += 1
            change["readings_removed"].append({"reading": reading, "type": rtype})
            removed_readings_detail.append({"kanji": kanji, "reading": reading, "type": rtype, "reason": "no_jlpt_covering_word"})

        # Phase 4b: Safety net — never leave a kanji with 0 readings
        if not new_on and not new_kun and (alive_on_backup or alive_kun_backup):
            new_on = alive_on_backup
            new_kun = alive_kun_backup
            change["readings_restored"] = len(new_on) + len(new_kun)

        # Step 5: Fill remaining slots with JLPT words
        final_readings_count = len(new_on) + len(new_kun)
        words_needed = max(final_readings_count, 1)
        if len(new_popular) < words_needed:
            candidates.sort(key=lambda x: (x[1], x[0]))
            for w, _p in candidates:
                if len(new_popular) >= words_needed:
                    break
                if w not in new_popular:
                    new_popular.append(w)
                    change["words_added"].append(w)
                    total_words_added += 1

        # Phase 5 validation: ensure popular_words >= 1
        if not new_popular:
            fallback = [
                w for w in chunks if kanji in w and chunks[w].get("russian_translation", "").strip()
            ]
            if fallback:
                new_popular.append(fallback[0])
                change["words_added"].append(fallback[0])
                total_words_added += 1

        change["final_on_readings"] = new_on
        change["final_kun_readings"] = new_kun
        change["final_popular_words"] = new_popular

        modified = (
            new_on != entry["on_readings"]
            or new_kun != entry["kun_readings"]
            or new_popular != entry.get("popular_words", [])
        )
        if modified:
            kanji_modified += 1

        new_entry = dict(entry)
        new_entry["on_readings"] = new_on
        new_entry["kun_readings"] = new_kun
        new_entry["popular_words"] = new_popular
        new_kanji_list.append(new_entry)

        if modified:
            changes_by_kanji[kanji] = change

    # Final validation
    kanji_with_empty_pw = sum(1 for k in new_kanji_list if not k.get("popular_words"))
    pw_in_chunks = sum(
        1
        for k in new_kanji_list
        for w in k.get("popular_words", [])
        if w in chunks and chunks[w].get("russian_translation", "").strip()
    )
    total_pw = sum(len(k.get("popular_words", [])) for k in new_kanji_list)
    coverage = (pw_in_chunks / total_pw * 100) if total_pw > 0 else 0

    summary = {
        "total_kanji": len(new_kanji_list),
        "kanji_modified": kanji_modified,
        "readings_removed": total_on_removed + total_kun_removed,
        "on_readings_removed": total_on_removed,
        "kun_readings_removed": total_kun_removed,
        "popular_words_removed": total_words_removed,
        "popular_words_added": total_words_added,
        "kanji_with_empty_popular_words_after": kanji_with_empty_pw,
        "coverage_percent": round(coverage, 2),
    }

    report = {
        "summary": summary,
        "changes_by_kanji": changes_by_kanji,
        "removed_readings_detail": removed_readings_detail,
    }

    print()
    print("=" * 60)
    print("AUDIT SUMMARY")
    print("=" * 60)
    print(f"  Total kanji:          {summary['total_kanji']}")
    print(f"  Kanji modified:       {summary['kanji_modified']}")
    print(f"  ON readings removed:  {summary['on_readings_removed']}")
    print(f"  KUN readings removed: {summary['kun_readings_removed']}")
    print(f"  Total readings removed: {summary['readings_removed']}")
    print(f"  Popular words removed: {summary['popular_words_removed']}")
    print(f"  Popular words added:   {summary['popular_words_added']}")
    print(f"  Empty popular_words:   {summary['kanji_with_empty_popular_words_after']}")
    print(f"  Coverage:              {summary['coverage_percent']}%")
    print()

    if not args.apply:
        print("DRY-RUN: No changes applied to kanji.json")

    report_json = json.dumps(report, ensure_ascii=False, indent=2)
    with open(REPORT_PATH, "w", encoding="utf-8") as f:
        f.write(report_json)
    print(f"Report saved to: {REPORT_PATH}")

    # Save removed popular words for the companion creation guard
    # (KnowledgeSet::create_companion_vocab_cards reads the Rust constant
    # origa/src/dictionary/removed_popular_words.rs — sync it manually
    # after an audit run; see the constant's MAINTENANCE note).
    removed_words = sorted(set(
        w for ch in changes_by_kanji.values() for w in ch["words_removed"]
    ))
    removed_words_path = ROOT / "scripts" / "removed_popular_words.json"
    with open(removed_words_path, "w", encoding="utf-8") as f:
        json.dump(removed_words, f, ensure_ascii=False, indent=2)
    print(f"Removed words list saved to: {removed_words_path} ({len(removed_words)} words)")

    if args.apply:
        kanji_data["kanji"] = new_kanji_list
        with open(KANJI_PATH, "w", encoding="utf-8") as f:
            json.dump(kanji_data, f, ensure_ascii=False, indent=2)
            f.write("\n")
        print(f"APPLIED: kanji.json updated at {KANJI_PATH}")

    print()
    print("Done.")


if __name__ == "__main__":
    main()
