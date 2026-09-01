#!/usr/bin/env python3
"""Merge hand-authored N1 grammar batches into grammar_v2.json.

Batch files: scripts/data/n1/*.json, each {"rules": [...]} where a rule has
level=N1, content{English,Russian} in schema v2 shape, plus optional
"related_titles": [[title, relation?, note?], ...] — references are
resolved to rule_id against the merged corpus (existing 515 + earlier N1
batches) at merge time. rule_id (ULID) is assigned here, keeping authoring
clean and references title-based.

Every merged rule passes the same checks as validate_grammar_v2 (structure,
emoji ban, JIS charset, fenced examples). The merge is transactional: on any
batch error nothing is written.

Run: python scripts/merge_n1_grammar.py [--dry-run]
"""

from __future__ import annotations

import json
import os
import re
import shutil
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BATCH_DIR = ROOT / "scripts" / "data" / "n1"
PATH = ROOT / "cdn" / "grammar" / "grammar_v2.json"

CROCKFORD = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
_last_ulid = [0]


def new_ulid() -> str:
    """Monotonic ULID: 48-bit ms timestamp + 80 random bits, Crockford base32.

    Encoded most-significant first across 26 chars (128 bits / 5 bits with
    leading zero padding), matching the ULID spec and the `ulid` crate.
    """
    ms = int(time.time() * 1000)
    if ms <= _last_ulid[0]:
        ms = _last_ulid[0] + 1
    _last_ulid[0] = ms
    rand = int.from_bytes(os.urandom(10), "big")
    value = (ms << 80) | rand
    return "".join(
        CROCKFORD[(value >> shift) & 0x1F] for shift in range(125, -1, -5)
    )


def norm_title(title: str) -> str:
    stripped = re.sub(r"～|〜|\s|/|／|・", "", title)
    return re.sub(r"（.*?）|\(.*?\)", "", stripped)


EMOJI_RE = re.compile(
    "[\U0001F000-\U0001FAFF\U00002600-\U000027BF\U00002B00-\U00002BFF"
    "\U0001F1E6-\U0001F1FF\U0000FE00-\U0000FE0F\U0000200D\U00002705\U0000274C\U00002764]"
)


def check_text(value: str, loc: str, errors: list[str]) -> None:
    if EMOJI_RE.search(value):
        errors.append(f"{loc}: emoji in text")
    examples_bad = [ch for ch in value if not is_allowed(ch)]
    if examples_bad:
        errors.append(f"{loc}: non-JIS chars {''.join(sorted(set(examples_bad)))}")


def is_allowed(ch: str) -> bool:
    if ch.isascii():
        return True
    try:
        ch.encode("euc_jis_2004")
        return True
    except UnicodeEncodeError:
        pass
    return ch in "～〜…‥ー―‐－—–→←↔↑↓⇒⇔（）《》「」『』・，、。：；！？＜＞＝＋＊"


def check_rule(rule: dict, idx: int, errors: list[str]) -> None:
    loc = f"rule[{idx}]"
    if rule.get("level") != "N1":
        errors.append(f"{loc}: level must be N1")
    content = rule.get("content") or {}
    if set(content.keys()) != {"English", "Russian"}:
        errors.append(f"{loc}: content must have English+Russian")
        return
    for lang, c in content.items():
        for field in ("title", "short_description", "explanation", "how_to_form", "examples", "pro_tip"):
            if not isinstance(c.get(field), str) or not c[field].strip():
                errors.append(f"{loc}/{lang}: missing {field}")
        nuances = c.get("nuances") or {}
        mistakes = nuances.get("common_mistakes") or []
        notes = nuances.get("notes") or []
        if not mistakes and not notes:
            errors.append(f"{loc}/{lang}: nuances empty")
        for m in mistakes:
            if not m.get("wrong") or not m.get("correct"):
                errors.append(f"{loc}/{lang}: common_mistake missing wrong/correct")
        for n in notes:
            if n.get("tag") not in {"register", "variation", "collocation", "formality", "other"}:
                errors.append(f"{loc}/{lang}: bad note tag")
            if not n.get("text"):
                errors.append(f"{loc}/{lang}: empty note")
        for field in ("title", "short_description", "explanation", "how_to_form", "examples", "pro_tip"):
            check_text(c[field], f"{loc}/{lang}/{field}", errors)
        for m in mistakes:
            for k in ("wrong", "correct", "note"):
                if m.get(k):
                    check_text(m[k], f"{loc}/{lang}/mistake.{k}", errors)
        for n in notes:
            check_text(n["text"], f"{loc}/{lang}/note", errors)
        if not c["examples"].lstrip().startswith("```"):
            errors.append(f"{loc}/{lang}: examples not fenced")


def build_title_index(rules: list[dict]) -> dict[str, str]:
    index: dict[str, str] = {}
    for rule in rules:
        for c in rule["content"].values():
            title = c.get("title")
            if title:
                index.setdefault(norm_title(title), rule["rule_id"])
    return index


def main() -> int:
    dry_run = "--dry-run" in sys.argv
    data = json.loads(PATH.read_text(encoding="utf-8"))
    existing = data["grammar"]
    title_index = build_title_index(existing)
    existing_ids = {r["rule_id"] for r in existing}

    batch_files = sorted(BATCH_DIR.glob("*.json")) if BATCH_DIR.exists() else []
    if not batch_files:
        print(f"no batches in {BATCH_DIR}")
        return 1

    merged: list[dict] = []
    skipped_dupes: list[str] = []
    errors: list[str] = []
    for batch_path in batch_files:
        batch = json.loads(batch_path.read_text(encoding="utf-8"))
        for idx, rule in enumerate(batch.get("rules", [])):
            check_rule(rule, idx, errors)
            en_title = (rule.get("content", {}).get("English", {}) or {}).get("title", "")
            key = norm_title(en_title)
            if key and key in title_index:
                skipped_dupes.append(en_title)
                continue
            rule_id = new_ulid()
            if rule_id in existing_ids:
                errors.append(f"duplicate ulid {rule_id}")
            related_new = []
            for c in rule["content"].values():
                related_new.append(c.pop("related_titles", None) or [])
            # resolve titles -> rule_id (same list for both languages by design)
            related_refs = []
            for entry in related_new[0][: len(related_new[0])]:
                target_title = entry[0]
                target = title_index.get(norm_title(target_title))
                if not target:
                    errors.append(f"{en_title}: related title unresolved: {target_title!r}")
                    continue
                ref = {"rule_id": target}
                if len(entry) > 1 and entry[1]:
                    ref["relation"] = entry[1]
                if len(entry) > 2 and entry[2]:
                    ref["note"] = entry[2]
                related_refs.append(ref)
            if related_refs:
                for lang in ("English", "Russian"):
                    rule["content"][lang]["related_patterns"] = [dict(r) for r in related_refs]
            rule["rule_id"] = rule_id
            rule.pop("related_titles", None)
            merged.append(rule)
            title_index.setdefault(key, rule_id)

    if errors:
        print("MERGE ABORTED — batch errors:")
        for e in errors[:30]:
            print("  ", e)
        return 1

    if skipped_dupes:
        print(f"skipped {len(skipped_dupes)} duplicates: {', '.join(skipped_dupes[:10])}…")
    if dry_run:
        print(f"would merge {len(merged)} rules from {len(batch_files)} batches")
        return 0

    backup = PATH.with_suffix(f".{int(time.time())}.backup.json")
    shutil.copy2(PATH, backup)
    existing.extend(merged)
    PATH.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    n1_total = sum(1 for r in existing if r["level"] == "N1")
    print(f"merged {len(merged)} rules; N1 total now {n1_total}; backup: {backup.name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
