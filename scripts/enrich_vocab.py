from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Dict, Iterable, Mapping, MutableMapping, Sequence

RESTORE_KEYS: Sequence[str] = (
    "russian_examples",
    "english_examples",
    "part_of_speech",
    "level",
)


def load_json(path: Path) -> Mapping[str, MutableMapping]:
    with path.open("r", encoding="utf-8") as file:
        return json.load(file)


def load_reference(reference_dir: Path, glob: str) -> Dict[str, MutableMapping]:
    reference: Dict[str, MutableMapping] = {}
    for path in sorted(reference_dir.glob(glob)):
        data = load_json(path)
        for term, payload in data.items():
            if term in reference:
                continue  # keep first occurrence
            reference[term] = payload
    if not reference:
        raise SystemExit(
            f"No reference data found in {reference_dir} with pattern {glob}"
        )
    return reference


def restore_fields(
    entry: MutableMapping,
    reference_entry: Mapping[str, MutableMapping],
    keys_to_restore: Iterable[str],
) -> Dict:
    enriched = dict(entry)
    for key in keys_to_restore:
        if key in reference_entry:
            enriched[key] = reference_entry[key]
    return enriched


def enrich_file(
    path: Path,
    reference: Mapping[str, MutableMapping],
    output_dir: Path,
    suffix: str,
    keys_to_restore: Iterable[str],
) -> Path:
    data = load_json(path)
    enriched_data = {}
    missing: list[str] = []

    for term, payload in data.items():
        ref = reference.get(term)
        if ref is None:
            missing.append(term)
            enriched_data[term] = dict(payload)
            continue
        enriched_data[term] = restore_fields(payload, ref, keys_to_restore)

    output_dir.mkdir(parents=True, exist_ok=True)
    output_path = output_dir / f"{path.stem}{suffix}{path.suffix}"
    with output_path.open("w", encoding="utf-8") as file:
        json.dump(enriched_data, file, ensure_ascii=False, indent=2)

    if missing:
        print(f"[WARN] {path.name}: {len(missing)} terms not found in reference")
    print(f"{path.name} -> {output_path}")
    return output_path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Enrich minified vocabulary files with fields from reference data.",
    )
    parser.add_argument(
        "--input-dir",
        type=Path,
        default=Path("vocabulary"),
        help="Directory containing minified files (default: vocabulary)",
    )
    parser.add_argument(
        "--input-glob",
        default="*.json",
        help="Glob to select minified files (default: *.json)",
    )
    parser.add_argument(
        "--reference-dir",
        type=Path,
        default=Path("words"),
        help="Directory with full vocabulary_*.json reference files (default: words)",
    )
    parser.add_argument(
        "--reference-glob",
        default="vocabulary_*.json",
        help="Glob for reference files (default: vocabulary_*.json)",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=None,
        help="Where to write enriched files (default: same as input-dir)",
    )
    parser.add_argument(
        "--suffix",
        default="_enriched",
        help="Suffix before extension for output files (default: _enriched)",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    input_dir: Path = args.input_dir
    input_glob: str = args.input_glob
    output_dir: Path | None = args.output_dir
    suffix: str = args.suffix

    reference = load_reference(args.reference_dir, args.reference_glob)
    files = sorted(input_dir.glob(input_glob))
    if not files:
        raise SystemExit(f"No input files match {input_glob} in {input_dir}")

    final_output_dir = output_dir or input_dir

    for src in files:
        enrich_file(src, reference, final_output_dir, suffix, RESTORE_KEYS)


if __name__ == "__main__":
    main()
