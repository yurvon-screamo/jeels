"""Apply canonical RU+EN corrections to cdn/dictionary/radicals.json.

Second pass after apply_radicals_en.py: fixes non-canonical Russian labels
and keeps the English fields in sync. Validation-first — the script refuses
to write unless:
- every corrected radical exists in the file (exactly once),
- RU name/description contain Cyrillic, EN fields contain none,
- nothing is empty,
- only the four text fields change (strokeCount/kanji/jlpt untouched).

Run from the repo root:

    python scripts/apply_radicals_fixes.py            # apply + validate
    python scripts/apply_radicals_fixes.py --check    # validate only
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from radicals_canonical_fixes import CORRECTIONS  # noqa: E402

_REPO_ROOT = Path(__file__).resolve().parents[1]
RADICALS_PATH = _REPO_ROOT / "cdn" / "dictionary" / "radicals.json"

CYRILLIC = re.compile(r"[\u0400-\u04FF]")


def validate(radicals: dict[str, dict], corrections: dict[str, tuple[str, str, str, str]]) -> list[str]:
    errors: list[str] = []
    missing = [char for char in corrections if char not in radicals]
    if missing:
        errors.append(f"corrections reference radicals absent from the file: {missing}")
    for char, (ru_name, ru_desc, en_name, en_desc) in corrections.items():
        for label, value, must_be_cyrillic in (
            ("ru_name", ru_name, True),
            ("ru_desc", ru_desc, True),
            ("en_name", en_name, False),
            ("en_desc", en_desc, False),
        ):
            if not value:
                errors.append(f"{char}: empty {label}")
            has_cyrillic = bool(CYRILLIC.search(value))
            if must_be_cyrillic and not has_cyrillic:
                errors.append(f"{char}: {label} must be Russian but has no Cyrillic")
            if not must_be_cyrillic and has_cyrillic:
                errors.append(f"{char}: {label} must be English but contains Cyrillic")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="validate only, no writes")
    args = parser.parse_args()

    original_text = RADICALS_PATH.read_text(encoding="utf-8")
    data = json.loads(original_text)
    radicals = data["radicals"]

    malformed = [row for row in CORRECTIONS if len(row) != 5]
    if malformed:
        print(
            "VALIDATION ERROR: corrections rows must be "
            f"(char, ru_name, ru_desc, en_name, en_desc); got {malformed!r}"
        )
        return 1
    corrections = {char: rest for char, *rest in CORRECTIONS}

    if len(corrections) != len(CORRECTIONS):
        print("VALIDATION ERROR: duplicate radical in the corrections table")
        return 1

    errors = validate(radicals, corrections)
    if errors:
        for error in errors:
            print(f"VALIDATION ERROR: {error}")
        return 1

    before = {
        char: json.dumps(
            {k: v for k, v in record.items() if k not in {"name", "description", "name_en", "description_en"}},
            sort_keys=True,
            ensure_ascii=False,
        )
        for char, record in radicals.items()
    }

    if not args.check:
        # Snapshot the pre-correction state; .json.bak (from the EN pass)
        # already holds the pristine RU-only file, this one holds the
        # translated-but-not-yet-canonical state.
        backup_path = RADICALS_PATH.with_suffix(".json.bak2")
        if not backup_path.exists():
            shutil.copy2(RADICALS_PATH, backup_path)

        for char, (ru_name, ru_desc, en_name, en_desc) in corrections.items():
            radicals[char]["name"] = ru_name
            radicals[char]["description"] = ru_desc
            radicals[char]["name_en"] = en_name
            radicals[char]["description_en"] = en_desc

        tmp_fd, tmp_path = tempfile.mkstemp(dir=str(RADICALS_PATH.parent), suffix=".json")
        with open(tmp_fd, "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, indent="\t")
            f.write("\n")
        shutil.move(tmp_path, str(RADICALS_PATH))

    # Post-write invariant: nothing but the four text fields changed.
    after = json.loads(RADICALS_PATH.read_text(encoding="utf-8"))
    for char, record in after["radicals"].items():
        stripped = json.dumps(
            {k: v for k, v in record.items() if k not in {"name", "description", "name_en", "description_en"}},
            sort_keys=True,
            ensure_ascii=False,
        )
        if stripped != before[char]:
            print(f"INVARIANT VIOLATION: {char}: non-text fields changed")
            return 1

    changed = sum(
        1
        for char, (ru_name, _, _, _) in corrections.items()
        if after["radicals"][char]["name"] == ru_name
    )
    print(f"OK: {changed}/{len(corrections)} corrections applied; "
          f"{len(after['radicals'])} radicals total")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
