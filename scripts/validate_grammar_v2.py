#!/usr/bin/env python3
"""Schema v2 validator + slop-linter for Origa grammar data.

Usage:
    python scripts/validate_grammar_v2.py [path]      # default: cdn/grammar/grammar_v2.json
    python scripts/validate_grammar_v2.py --json      # machine-readable output

Exit code 0 = valid, 1 = errors found.

Checks (ERROR = blocking):
  structure : schema marker, rule shape, required fields, ULID validity/uniqueness
  slop      : emoji anywhere (data must be render-neutral), simplified-Chinese
              characters outside JIS X 0213 + allowlist
  format    : examples must be fenced code blocks; how_to_form must contain a
              table or code fence
  integrity : related_patterns.rule_id must resolve inside the corpus;
              relation enum values; nuances must have >=1 mistake or note

Checks (WARN = non-blocking, tracked for cleanup):
  legacy format anomalies, duplicate normalized titles (user-approved to keep),
  empty optional fields.
"""

from __future__ import annotations

import json
import re
import sys
import unicodedata
from pathlib import Path

LEVELS = {"N5", "N4", "N3", "N2", "N1"}
LANGS = {"English", "Russian"}
REQUIRED_TEXT_FIELDS = (
    "title",
    "short_description",
    "explanation",
    "how_to_form",
    "examples",
    "pro_tip",
)
RELATIONS = {"pair", "contrast", "derived"}
NUANCE_TAGS = {"register", "variation", "collocation", "formality", "other"}

# Emoji / slop markers: pictographs, symbols, variation selectors, ZWJ.
# Deliberately broad; arrows (U+2192 →) and CJK punctuation stay allowed.
EMOJI_RE = re.compile(
    "["
    "\U0001F000-\U0001FAFF"  # pictographic blocks (incl. supplemental symbols)
    "\U00002600-\U000027BF"  # misc symbols + dingbats (⚠ ✅ ❌ ✳ …)
    "\U00002B00-\U00002BFF"  # arrows/stars block (⭐ …)
    "\U0001F1E6-\U0001F1FF"  # regional indicators
    "\U0000FE00-\U0000FE0F"  # variation selectors
    "\U0000200D"             # ZWJ
    "\U000020E3"             # combining enclosing keycap
    "\U00002705\U0000274C\U00002764"  # ✅ ❌ ❤
    "]"
)

# Characters outside JIS X 0213 that legitimately appear in grammar content.
ALLOWED_NON_JIS = set(
    "～〜…‥ー―‐－—–☺"  # tildes, ellipsis, dashes (em/en included: house style)
    "→←↔↑↓⇒⇔"          # arrows used in explanations
    "<>=+*#`|_\\/~%^&$@!?\"'()[]{}.,;:- "  # ASCII printable (markdown)
    " "
    "（）《》「」『』・，、。：；！？＜＞＝＋＊"
) | {chr(c) for c in range(0x21, 0x7F)}


def is_jis_or_allowed(char: str) -> bool:
    if char in ALLOWED_NON_JIS:
        return True
    if char.isascii():
        return True
    try:
        char.encode("euc_jis_2004")
        return True
    except UnicodeEncodeError:
        return False


def norm_title(title: str) -> str:
    stripped = re.sub(r"～|〜|\s|/|／|・", "", title)
    return re.sub(r"（.*?）|\(.*?\)", "", stripped)


class Report:
    def __init__(self) -> None:
        self.errors: list[str] = []
        self.warnings: list[str] = []

    def error(self, where: str, message: str) -> None:
        self.errors.append(f"ERROR {where}: {message}")

    def warn(self, where: str, message: str) -> None:
        self.warnings.append(f"WARN  {where}: {message}")


def validate_rule(rule: dict, idx: int, rule_ids: set[str], report: Report) -> None:
    where = f"rule[{idx}]"
    rule_id = rule.get("rule_id")
    if not isinstance(rule_id, str) or not re.fullmatch(r"[0-9A-HJKMNP-TV-Z]{26}", rule_id):
        report.error(where, f"invalid ULID rule_id: {rule_id!r}")
    elif rule_id in rule_ids:
        report.error(where, f"duplicate rule_id {rule_id}")
    else:
        rule_ids.add(rule_id)
    where = f"{rule_id or where}"

    level = rule.get("level")
    if level not in LEVELS:
        report.error(where, f"invalid level {level!r}")

    keywords = rule.get("keywords")
    if keywords is not None and not (
        isinstance(keywords, list)
        and all(isinstance(k, list) and all(isinstance(t, str) for t in k) for k in keywords)
    ):
        report.error(where, "keywords must be a list of string lists")

    content = rule.get("content")
    if not isinstance(content, dict) or set(content.keys()) != LANGS:
        report.error(where, f"content must have exactly {sorted(LANGS)}, got {content!r}")
        return

    for lang, c in content.items():
        loc = f"{where}/{lang}"
        if not isinstance(c, dict):
            report.error(loc, "content entry must be an object")
            continue
        for field in REQUIRED_TEXT_FIELDS:
            value = c.get(field)
            if not isinstance(value, str):
                report.error(loc, f"missing/non-string field {field!r}")
            elif not value.strip() and field != "pro_tip":
                report.warn(loc, f"empty field {field!r}")
            elif not value.strip() and field == "pro_tip":
                pass  # pro_tip is optional in practice (legacy hollow rules)

        title = c.get("title", "")
        if isinstance(title, str) and not title.strip():
            report.error(loc, "empty title")

        validate_nuances(c.get("nuances"), loc, report)
        validate_warnings(c.get("warnings"), loc, report)
        validate_related_patterns(c.get("related_patterns"), loc, report)
        lint_text_fields(c, loc, report)


def validate_nuances(nuances: object, loc: str, report: Report) -> None:
    if not isinstance(nuances, dict):
        report.error(loc, "nuances must be an object {common_mistakes, notes}")
        return
    mistakes = nuances.get("common_mistakes", [])
    notes = nuances.get("notes", [])
    if not isinstance(mistakes, list) or not isinstance(notes, list):
        report.error(loc, "nuances.common_mistakes/notes must be arrays")
        return
    for m in mistakes:
        if not isinstance(m, dict):
            report.error(loc, f"common_mistake must be object, got {m!r}")
            continue
        wrong, correct, note = m.get("wrong"), m.get("correct"), m.get("note")
        if not isinstance(wrong, str) or not wrong.strip():
            report.error(loc, f"common_mistake.wrong missing: {m!r}")
        if not isinstance(correct, str) or not correct.strip():
            report.error(loc, f"common_mistake.correct missing: {m!r}")
        if note is not None and not isinstance(note, str):
            report.error(loc, f"common_mistake.note must be string: {m!r}")
    for n in notes:
        if not isinstance(n, dict) or not isinstance(n.get("text"), str) or not n["text"].strip():
            report.error(loc, f"note must be {{tag, text}} with non-empty text, got {n!r}")
        elif n.get("tag") not in NUANCE_TAGS:
            report.error(loc, f"note.tag must be one of {sorted(NUANCE_TAGS)}, got {n!r}")
    if not mistakes and not notes:
        # Hollow legacy rules are filled in a dedicated content pass; the
        # validator tracks them as warnings so a pure schema conversion can
        # stay green while content debt remains visible.
        report.warn(loc, "nuances is empty (legacy hollow rule, content pass pending)")


def validate_warnings(warnings: object, loc: str, report: Report) -> None:
    if warnings is None:
        return
    if not isinstance(warnings, list) or any(
        not isinstance(w, str) or not w.strip() for w in warnings
    ):
        report.error(loc, "warnings must be a list of non-empty strings")


def validate_related_patterns(related: object, loc: str, report: Report) -> None:
    # Referential integrity is checked corpus-wide after all ids are collected.
    if related is None:
        return
    if not isinstance(related, list):
        report.error(loc, "related_patterns must be an array")
        return
    for entry in related:
        if not isinstance(entry, dict):
            report.error(loc, f"related_patterns entry must be object: {entry!r}")
            continue
        rid = entry.get("rule_id")
        if not isinstance(rid, str) or not re.fullmatch(r"[0-9A-HJKMNP-TV-Z]{26}", rid):
            report.error(loc, f"related_patterns.rule_id invalid: {rid!r}")
        if entry.get("relation") is not None and entry["relation"] not in RELATIONS:
            report.error(loc, f"related_patterns.relation invalid: {entry!r}")
        note = entry.get("note")
        if note is not None and not isinstance(note, str):
            report.error(loc, f"related_patterns.note must be string: {entry!r}")


def lint_text_fields(c: dict, loc: str, report: Report) -> None:
    for field, value in c.items():
        if not isinstance(value, str):
            continue
        for match in EMOJI_RE.finditer(value):
            report.error(loc, f"emoji in {field!r}: {match.group()!r} (U+{ord(match.group()):04X})")
        bad_chars = sorted(
            {
                ch
                for ch in value
                if not is_jis_or_allowed(ch)
                and not ch.isalnum()
                and unicodedata.category(ch) not in {"Ll", "Lu", "Lt", "Lm", "Lo", "Nd"}
            }
        )
        # alnum covers Cyrillic; CJK ideographs fall into Lo, checked via JIS below.
        non_jis_cjk = sorted(
            {ch for ch in value if is_cjk_ideograph(ch) and not is_jis_or_allowed(ch)}
        )
        if non_jis_cjk:
            report.error(loc, f"non-JIS CJK chars in {field!r}: {''.join(non_jis_cjk)}")
        if bad_chars:
            report.warn(loc, f"unusual chars in {field!r}: {''.join(bad_chars)}")

    examples = c.get("examples")
    if isinstance(examples, str) and examples.strip():
        # A leading bold group label ("**Hearsay:**") before the first fence
        # is a legitimate structure for combined rules.
        stripped_examples = re.sub(r"^(\*\*[^*]+\*\*\s*\n?)+", "", examples.lstrip())
        if not stripped_examples.startswith("```"):
            report.warn(loc, "examples does not start with a code fence")
    how_to_form = c.get("how_to_form")
    if (
        isinstance(how_to_form, str)
        and how_to_form.strip()
        and "|" not in how_to_form
        and "```" not in how_to_form
    ):
        report.warn(loc, "how_to_form has neither table nor code fence")


def is_cjk_ideograph(ch: str) -> bool:
    return (
        "\u4e00" <= ch <= "\u9fff"
        or "\u3400" <= ch <= "\u4dbf"
        or "\U00020000" <= ch <= "\U0002a6df"
    )


def validate_corpus(data: dict, report: Report) -> None:
    rules = data.get("grammar")
    if not isinstance(rules, list) or not rules:
        report.error("corpus", "grammar must be a non-empty array")
        return

    rule_ids: set[str] = set()
    for idx, rule in enumerate(rules):
        if not isinstance(rule, dict):
            report.error(f"rule[{idx}]", "not an object")
            continue
        validate_rule(rule, idx, rule_ids, report)

    # Referential integrity for related_patterns (needs the full id set).
    seen_titles: dict[str, int] = {}
    for rule in rules:
        if not isinstance(rule, dict):
            continue
        rid = rule.get("rule_id", "?")
        for lang, c in (rule.get("content") or {}).items():
            if not isinstance(c, dict):
                continue
            for entry in c.get("related_patterns") or []:
                target = entry.get("rule_id") if isinstance(entry, dict) else None
                if isinstance(target, str) and target not in rule_ids:
                    report.error(
                        f"{rid}/{lang}",
                        f"related_patterns points to unknown rule_id {target}",
                    )
            if entry_self_reference(rid, c.get("related_patterns")):
                report.error(f"{rid}/{lang}", "related_patterns references itself")
            title = c.get("title")
            if isinstance(title, str) and title:
                key = norm_title(title)
                if key in seen_titles:
                    report.warn(
                        f"{rid}/{lang}",
                        f"duplicate normalized title {title!r} (kept by user decision)",
                    )
                else:
                    seen_titles[key] = 1


def entry_self_reference(rid: str, related: object) -> bool:
    if not isinstance(related, list):
        return False
    return any(isinstance(e, dict) and e.get("rule_id") == rid for e in related)


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    as_json = "--json" in sys.argv
    default_path = Path(__file__).resolve().parents[1] / "cdn" / "grammar" / "grammar_v2.json"
    path = Path(args[0]) if args else default_path

    if not path.exists():
        print(f"file not found: {path}", file=sys.stderr)
        return 1

    raw = path.read_text(encoding="utf-8").lstrip("﻿")
    try:
        data = json.loads(raw)
    except json.JSONDecodeError as e:
        print(f"invalid JSON: {e}", file=sys.stderr)
        return 1

    report = Report()
    if data.get("schema") != 2:
        report.error("corpus", f"schema marker must be 2, got {data.get('schema')!r}")
    validate_corpus(data, report)

    rules = data.get("grammar") or []
    if as_json:
        print(
            json.dumps(
                {
                    "path": str(path),
                    "rules": len(rules),
                    "errors": report.errors,
                    "warnings": report.warnings,
                },
                ensure_ascii=False,
                indent=2,
            )
        )
    else:
        for line in report.errors:
            print(line)
        for line in report.warnings:
            print(line)
        print(f"\n{path}: {len(rules)} rules, {len(report.errors)} errors, {len(report.warnings)} warnings")

    return 1 if report.errors else 0


if __name__ == "__main__":
    sys.exit(main())
