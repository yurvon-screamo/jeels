"""Bundle kanji SVGs into JLPT-level JSON bundles.

Reads individual SVG files from cdn/kanji_animations/ and cdn/kanji_frames/,
groups by JLPT level (from cdn/dictionary/kanji.json), writes JSON bundles:

    cdn/kanji_animations_n5.json  {"一": "<svg>...", ...}
    cdn/kanji_frames_n5.json     {"一": "<svg>...", ...}
    ... (n4, n3, n2, n1)

Cache-Control: immutable (truly-static, content-hash in filename via deploy).

Deterministic: sorted keys, ensure_ascii=False, compact separators.
"""

import json
import sys
from pathlib import Path

JLPT_LEVELS = ["n5", "n4", "n3", "n2", "n1"]


def main() -> int:
    project_root = Path(__file__).resolve().parent.parent
    cdn = project_root / "cdn"

    kanji_json_path = cdn / "dictionary" / "kanji.json"
    if not kanji_json_path.is_file():
        print(f"ERROR: {kanji_json_path} not found", file=sys.stderr)
        return 1

    with kanji_json_path.open("r", encoding="utf-8") as f:
        kanji_data = json.load(f)

    kanji_list = kanji_data["kanji"]
    by_level: dict[str, list[str]] = {lvl: [] for lvl in JLPT_LEVELS}
    for entry in kanji_list:
        level = entry["jlpt"].lower()
        if level in by_level:
            by_level[level].append(entry["kanji"])

    for svg_type, src_dir_name in [("kanji_animations", "kanji_animations"), ("kanji_frames", "kanji_frames")]:
        src_dir = cdn / src_dir_name
        if not src_dir.is_dir():
            print(f"  {src_dir_name}/ not found, skipping", flush=True)
            continue

        for level in JLPT_LEVELS:
            kanjis = by_level[level]
            bundle: dict[str, str] = {}

            for kanji in sorted(kanjis):
                svg_path = src_dir / f"{kanji}.svg"
                if svg_path.is_file():
                    bundle[kanji] = svg_path.read_text(encoding="utf-8")

            if not bundle:
                continue

            out_path = cdn / f"{svg_type}_{level}.json"
            out_path.write_text(
                json.dumps(bundle, sort_keys=True, ensure_ascii=False, separators=(",", ":")),
                encoding="utf-8",
            )
            size_kb = out_path.stat().st_size / 1024
            print(f"  {out_path.name}: {len(bundle)} kanji, {size_kb:.0f} KB", flush=True)

    print("Done.", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
