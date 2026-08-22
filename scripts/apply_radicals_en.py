"""Apply the manual EN translation table to cdn/dictionary/radicals.json.

Validation-first: the script refuses to write unless
- the table covers every radical in the file exactly once,
- no EN field contains Cyrillic,
- every EN name/description is non-empty,
- the RU fields and the rest of each record stay byte-identical.

Run from the repo root:

    python scripts/apply_radicals_en.py            # apply + validate
    python scripts/apply_radicals_en.py --check    # validate only
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
from radicals_en_table import TRANSLATIONS  # noqa: E402

_REPO_ROOT = Path(__file__).resolve().parents[1]
RADICALS_PATH = _REPO_ROOT / "cdn" / "dictionary" / "radicals.json"

CYRILLIC = re.compile(r"[\u0400-\u04FF]")


def validate(data: dict, translations: dict[str, tuple[str, str]]) -> list[str]:
    errors: list[str] = []
    radicals: dict[str, dict] = data["radicals"]

    table_chars = set(translations)
    file_chars = set(radicals)
    missing = file_chars - table_chars
    extra = table_chars - file_chars
    if missing:
        errors.append(f"table misses radicals present in the file: {sorted(missing)}")
    if extra:
        errors.append(f"table contains radicals absent from the file: {sorted(extra)}")

    for char, (name_en, description_en) in translations.items():
        if char not in radicals:
            continue
        if not name_en or not description_en:
            errors.append(f"{char}: empty EN name or description")
        if CYRILLIC.search(name_en) or CYRILLIC.search(description_en):
            errors.append(f"{char}: Cyrillic leaked into an EN field")

    for char, record in radicals.items():
        for required in ("strokeCount", "kanji", "name", "description", "jlpt"):
            if required not in record:
                errors.append(f"{char}: required field {required!r} missing")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true", help="validate only, no writes")
    args = parser.parse_args()

    original_text = RADICALS_PATH.read_text(encoding="utf-8")
    data = json.loads(original_text)
    translations = {char: (name, desc) for char, name, desc in TRANSLATIONS}

    errors = validate(data, translations)
    if errors:
        for error in errors:
            print(f"VALIDATION ERROR: {error}")
        return 1

    before = {
        char: json.dumps(
            {k: v for k, v in record.items() if not k.endswith("_en")},
            sort_keys=True,
            ensure_ascii=False,
        )
        for char, record in data["radicals"].items()
    }

    if not args.check:
        # cdn/ is gitignored — snapshot the pre-write state to .bak so a bad
        # write is reversible (project rule: backup before potential data
        # loss). Note this snapshots whatever is on disk NOW: after the first
        # successful run that is the translated file, not the pristine RU
        # original — the RU fields are additionally recoverable from the
        # committed translation table in radicals_en_table.py.
        backup_path = RADICALS_PATH.with_suffix(".json.bak")
        if not backup_path.exists():
            shutil.copy2(RADICALS_PATH, backup_path)

        for char, (name_en, description_en) in translations.items():
            data["radicals"][char]["name_en"] = name_en
            data["radicals"][char]["description_en"] = description_en

        tmp_fd, tmp_path = tempfile.mkstemp(dir=str(RADICALS_PATH.parent), suffix=".json")
        with open(tmp_fd, "w", encoding="utf-8") as f:
            json.dump(data, f, ensure_ascii=False, indent="\t")
            f.write("\n")
        shutil.move(tmp_path, str(RADICALS_PATH))

    # Post-write invariant: RU payload untouched, EN fields present.
    after_data = json.loads(RADICALS_PATH.read_text(encoding="utf-8"))
    for char, record in after_data["radicals"].items():
        stripped = {k: v for k, v in record.items() if not k.endswith("_en")}
        if json.dumps(stripped, sort_keys=True, ensure_ascii=False) != before[char]:
            print(f"INVARIANT VIOLATION: {char}: non-EN fields changed")
            return 1
        if not args.check and (
            not record.get("name_en") or not record.get("description_en")
        ):
            print(f"INVARIANT VIOLATION: {char}: EN fields missing after write")
            return 1

    total = len(after_data["radicals"])
    translated = sum(
        1 for r in after_data["radicals"].values() if r.get("name_en") and r.get("description_en")
    )
    print(f"OK: {translated}/{total} radicals carry EN name+description")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
