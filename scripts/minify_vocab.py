from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Dict, Iterable, Mapping, MutableMapping, Sequence

DROP_KEYS: Sequence[str] = (
    "russian_examples",
    "english_examples",
    "part_of_speech",
    "level",
)


def load_json(path: Path) -> Mapping[str, MutableMapping]:
    with path.open("r", encoding="utf-8") as file:
        return json.load(file)


def strip_fields(entry: MutableMapping, keys_to_drop: Iterable[str]) -> Dict:
    return {key: value for key, value in entry.items() if key not in keys_to_drop}


def minify_file(path: Path, output_dir: Path, suffix: str) -> Path:
    data = load_json(path)
    cleaned = {term: strip_fields(payload, DROP_KEYS) for term, payload in data.items()}

    output_dir.mkdir(parents=True, exist_ok=True)
    output_path = output_dir / f"{path.stem}{suffix}{path.suffix}"
    with output_path.open("w", encoding="utf-8") as file:
        json.dump(cleaned, file, ensure_ascii=False, separators=(",", ":"))
    return output_path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Create minified vocabulary JSON files without example/metadata fields."
        )
    )
    parser.add_argument(
        "--input-dir",
        type=Path,
        default=Path("words"),
        help="Directory containing vocabulary_*.json files (default: words)",
    )
    parser.add_argument(
        "--glob",
        default="vocabulary_*.json",
        help="Glob pattern to pick source files (default: vocabulary_*.json)",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=None,
        help="Where to write results (default: same as input files)",
    )
    parser.add_argument(
        "--suffix",
        default="_min",
        help="Suffix added before extension for output files (default: _min)",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    input_dir: Path = args.input_dir
    output_dir: Path | None = args.output_dir
    suffix: str = args.suffix

    files = sorted(input_dir.glob(args.glob))
    if not files:
        raise SystemExit(f"No files match pattern {args.glob} in {input_dir}")

    final_output_dir = output_dir or input_dir

    for src in files:
        dst = minify_file(src, final_output_dir, suffix)
        print(f"{src.name} -> {dst}")


if __name__ == "__main__":
    main()
