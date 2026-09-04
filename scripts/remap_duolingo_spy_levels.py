"""[DEPRECATED] Diagnostic-only remapper for Spy x Family JLPT levels (#178 S-3).

The canonical source of truth is `origa_ui/build.rs` (`generate_well_known_meta`):
Duolingo levels are copied verbatim from each content file's own `level` field,
and every Spy x Family set is tagged N3 when
`cdn/well_known_set/well_known_sets_meta.json` is generated. `cargo build -p
origa_ui` overwrites the meta file, so any manual edits made by this script
would be discarded on the next build.

The Duolingo remapping branch was REMOVED (2026-09): the old title heuristic
("Section 1-2 -> N5, 3-4 -> N4, 5-6 -> N3") contradicted the corpus ground
truth carried by the content `level` fields (Section/Module 1-3 -> N5,
4 -> N4, 5-6 -> N3) and misread RU-series English titles ("Module 5 Section
16"), dropping 55 sets and mistagging 66. Re-adding it here would fight the
canonical source. See `origa/tests/well_known_sets_audit.rs` for the guarded
invariants (completeness + meta level == content level).

Spy x Family content files all carry `level: "N3"` in their own metadata
(verified across all 12 episodes); this script remains a fallback/diagnostic
tool to verify or repair a stale Spy tagging in the meta file without
rebuilding.

Run:
    python scripts/remap_duolingo_spy_levels.py --cdn cdn
    python scripts/remap_duolingo_spy_levels.py --cdn cdn --dry-run
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from _cdn_io import atomic_write_json

SPY_FAMILY_LEVEL = "N3"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--cdn",
        required=True,
        help="Path to cdn/ directory (containing well_known_set/)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show planned updates without writing files",
    )
    return parser.parse_args()


def is_spy_family(record: dict) -> bool:
    return record.get("set_type") == "SpyFamily"


def main() -> int:
    args = parse_args()
    meta_path = Path(args.cdn) / "well_known_set" / "well_known_sets_meta.json"
    if not meta_path.exists():
        print(f"Error: {meta_path} not found")
        return 1

    with open(meta_path, encoding="utf-8") as f:
        records = json.load(f)

    spy_changes: list[tuple[str, str, str]] = []

    for record in records:
        rid = record.get("id", "?")
        old_level = record.get("level")

        if is_spy_family(record):
            if SPY_FAMILY_LEVEL != old_level:
                record["level"] = SPY_FAMILY_LEVEL
                spy_changes.append((rid, old_level, SPY_FAMILY_LEVEL))

    print(f"Spy x Family updates: {len(spy_changes)}")
    for rid, old, new in spy_changes[:5]:
        print(f"  [{rid}] {old} -> {new}")
    if len(spy_changes) > 5:
        print(f"  ... and {len(spy_changes) - 5} more")

    if not args.dry_run and spy_changes:
        atomic_write_json(meta_path, records)
        print(f"\nWrote {len(spy_changes)} updates to {meta_path}")
    elif args.dry_run:
        print("\n--dry-run: no files modified.")
    else:
        print("\nNo changes needed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
