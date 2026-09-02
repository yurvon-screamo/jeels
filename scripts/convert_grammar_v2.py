#!/usr/bin/env python3
"""Convert cdn/grammar/grammar.json (v1) into grammar_v2.json (schema v2).

Semantic purity contract: this pass ONLY re-packs data —
  * nuances: raw markdown bullets -> {common_mistakes[], notes[]}
             (- "❌ wrong → ✅ correct" pairs, "🔄" -> variation notes,
              "💡"/"✅"/plain bullets -> other notes)
  * explanation: "> ⚠️ ..." blockquote lines -> warnings[]
  * related_patterns: free text -> [{rule_id, note}] resolved against
             corpus titles; unresolvable references degrade to notes
             (information is never dropped)
  * emoji markers are stripped everywhere (render is UI's job)
Content wording itself is NOT touched here — content fixes live in
separate cleanup passes. v1 grammar.json stays untouched (frozen).
"""

from __future__ import annotations

import json
import re
import shutil
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "cdn" / "grammar" / "grammar.json"
DST = ROOT / "cdn" / "grammar" / "grammar_v2.json"

EMOJI_RE = re.compile(
    "["
    "\U0001F000-\U0001FAFF"  # pictographic blocks (incl. regional indicators, supplemental symbols)
    "\U00002600-\U000027BF"  # misc symbols + dingbats (incl. check/cross marks and hearts)
    "\U00002B00-\U00002BFF"  # arrows/stars block
    "\U0000FE00-\U0000FE0F"  # variation selectors
    "\U0000200D"             # ZWJ
    "\U000020E3"             # combining enclosing keycap
    "]"
)

STRIP_LABELS = ("**Important:**", "**Важно:**", "**Note:**", "**Примечание:**")


def strip_emoji(text: str) -> str:
    cleaned = EMOJI_RE.sub("", text)
    cleaned = re.sub(r"[ \t]{2,}", " ", cleaned)
    return cleaned.strip()


def norm_title(title: str) -> str:
    stripped = re.sub(r"～|〜|\s|/|／|・", "", title)
    return re.sub(r"（.*?）|\(.*?\)", "", stripped)


def parse_nuances(raw: str) -> dict:
    mistakes: list[dict] = []
    notes: list[dict] = []
    for raw_line in raw.split("\n"):
        line = raw_line.strip()
        if not line:
            continue
        line = re.sub(r"^-\s*", "", line)
        if line.startswith("❌"):
            body = line[1:].strip()
            if "→" in body:
                wrong, rest = body.split("→", 1)
                rest = rest.strip()
                if rest.startswith("✅"):
                    rest = rest[1:].strip()
                if " — " in rest:
                    correct, note = rest.split(" — ", 1)
                    mistakes.append(
                        {
                            "wrong": strip_emoji(wrong),
                            "correct": strip_emoji(correct),
                            "note": strip_emoji(note) or None,
                        }
                    )
                else:
                    mistakes.append(
                        {"wrong": strip_emoji(wrong), "correct": strip_emoji(rest), "note": None}
                    )
            else:
                # mistake without an explicit fix — keep as a note, never lose it
                notes.append({"tag": "other", "text": strip_emoji(body)})
        elif line.startswith("🔄"):
            notes.append({"tag": "variation", "text": strip_emoji(line[1:].strip())})
        elif line.startswith(("💡", "✅")):
            body = line.lstrip("💡✅").strip()
            notes.append({"tag": "other", "text": strip_emoji(body)})
        else:
            notes.append({"tag": "other", "text": strip_emoji(line)})
    return {"common_mistakes": mistakes, "notes": notes}


def extract_warnings(explanation: str) -> tuple[str, list[str]]:
    """Lift `> ⚠️ ...` blockquote groups into warnings; return cleaned text."""
    warnings: list[str] = []
    kept_lines: list[str] = []
    in_warning_block = False
    for raw_line in explanation.split("\n"):
        stripped = raw_line.strip()
        is_quote = stripped.startswith(">")
        quote_body = stripped.lstrip(">").strip()
        if is_quote and (quote_body.startswith("⚠") or (in_warning_block and quote_body)):
            in_warning_block = True
            quote_body = strip_emoji(quote_body)
            for label in STRIP_LABELS:
                if quote_body.startswith(label):
                    quote_body = quote_body[len(label) :].strip()
            if quote_body:
                warnings.append(strip_emoji(quote_body))
            continue
        in_warning_block = False
        kept_lines.append(raw_line.rstrip())
    cleaned = "\n".join(kept_lines)
    cleaned = re.sub(r"\n{3,}", "\n\n", cleaned).strip()
    return cleaned, warnings


def resolve_related(
    legacy: str | None,
    title_index: dict[str, str],
    self_id: str,
    self_titles: set[str],
) -> tuple[list[dict], list[dict]]:
    """Parse legacy related_patterns text; return (related, degraded_notes)."""
    related: list[dict] = []
    degraded: list[dict] = []
    if not legacy or not legacy.strip():
        return related, degraded

    candidates: list[tuple[str, str | None]] = []  # (target_title, note)

    for raw_line in legacy.split("\n"):
        line = raw_line.strip()
        if not line:
            continue
        # markdown table rows: | pattern | note |
        if line.startswith("|") and not re.match(r"^\|[\s\-|]+\|$", line):
            cells = [c.strip() for c in line.strip("|").split("|")]
            cells = [c for c in cells if c and not set(c) <= {"-", " "}]
            if cells and any(re.search(r"[ ぁ-んァ-ヶ一-龯ー～]", c) for c in cells):
                pattern = cells[0]
                note = " — ".join(cells[1:]) or None
                candidates.append((pattern, note))
            continue
        line = re.sub(r"^-\s*", "", line)
        # "См. также: X — note" / "See also: X — note"
        match = re.match(r"^(?:См\. ?также|See also)\s*:\s*(.+)$", line, re.IGNORECASE)
        if match:
            body = match.group(1).strip()
            target, note = split_target_note(body)
            candidates.append((target, note))
            continue
        # comma/、 separated list of patterns
        if re.search(r"[、,]", line) and re.search(r"[ ぁ-んァ-ヶ一-龯ー～]", line):
            for part in re.split(r"[、,]", line):
                part = part.strip()
                if part:
                    candidates.append((part, None))
            continue
        target, note = split_target_note(line)
        candidates.append((target, note))

    for target, note in candidates:
        key = norm_title(target)
        resolved = title_index.get(key)
        if resolved and resolved != self_id and key not in self_titles:
            entry = {"rule_id": resolved}
            clean_note = strip_emoji(note) if note else None
            if clean_note:
                entry["note"] = clean_note
            related.append(entry)
        elif key not in self_titles:
            degraded.append({"tag": "other", "text": strip_emoji(target)})
    return related, degraded


def split_target_note(body: str) -> tuple[str, str | None]:
    if " — " in body:
        target, note = body.split(" — ", 1)
        return target.strip(), note.strip()
    return body.strip(), None


def convert_rule(rule: dict, title_index: dict[str, str]) -> dict:
    self_id = rule["rule_id"]
    self_titles = {
        norm_title(c.get("title", "")) for c in rule["content"].values() if c.get("title")
    }
    new_rule: dict = {"rule_id": self_id, "level": rule["level"]}
    if "format_map" in rule:
        new_rule["format_map"] = rule["format_map"]
    if "keywords" in rule:
        new_rule["keywords"] = rule["keywords"]

    new_content: dict = {}
    for lang, c in rule["content"].items():
        explanation, warnings = extract_warnings(c.get("explanation") or "")
        nuances = parse_nuances(c.get("nuances") or "")
        related, degraded = resolve_related(
            c.get("related_patterns"), title_index, self_id, self_titles
        )
        for note in degraded:
            nuances["notes"].append(note)
        new_c = {
            "title": c["title"],
            "short_description": strip_emoji(c.get("short_description") or ""),
            "explanation": strip_emoji(explanation),
            "warnings": warnings,
            "how_to_form": strip_emoji(c.get("how_to_form") or ""),
            "examples": strip_emoji(c.get("examples") or ""),
            "nuances": nuances,
            "pro_tip": strip_emoji(c.get("pro_tip") or ""),
        }
        if related:
            new_c["related_patterns"] = related
        new_content[lang] = new_c
    new_rule["content"] = new_content
    return new_rule


def build_title_index(rules: list[dict]) -> dict[str, str]:
    index: dict[str, str] = {}
    for rule in rules:
        for c in rule["content"].values():
            title = c.get("title")
            if title:
                index.setdefault(norm_title(title), rule["rule_id"])
    return index


def main() -> int:
    data = json.loads(SRC.read_text(encoding="utf-8"))
    rules = data["grammar"]
    title_index = build_title_index(rules)

    converted = [convert_rule(rule, title_index) for rule in rules]
    out = {"schema": 2, "grammar": converted}

    if DST.exists():
        backup = DST.with_suffix(f".{int(time.time())}.backup.json")
        shutil.copy2(DST, backup)
        print(f"backup: {backup.name}")

    DST.write_text(
        json.dumps(out, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(f"converted {len(converted)} rules -> {DST}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
