"""Bundle phrases/data/*.json into 4 parts (~12 MB each) for CDN.

198 individual chunks → 4 bundle files. Each bundle is a JSON object
keyed by chunk ID. WASM peak memory = 1 bundle (~12 MB) at a time.

Output:
  cdn/phrases/data_bundle_0.json  (p0000-p0049)
  cdn/phrases/data_bundle_1.json  (p0050-p0099)
  cdn/phrases/data_bundle_2.json  (p0100-p0149)
  cdn/phrases/data_bundle_3.json  (p0150-p0197)

Deterministic: sorted keys, ensure_ascii=False, compact separators.
"""

import json
import sys
from pathlib import Path

CHUNKS_PER_BUNDLE = 50


def main() -> int:
    project_root = Path(__file__).resolve().parent.parent
    data_dir = project_root / "cdn" / "phrases" / "data"
    out_dir = project_root / "cdn" / "phrases"

    if not data_dir.is_dir():
        print(f"ERROR: {data_dir} not found", file=sys.stderr)
        return 1

    files = sorted(data_dir.glob("p*.json"))
    total = len(files)
    bundle_count = (total + CHUNKS_PER_BUNDLE - 1) // CHUNKS_PER_BUNDLE
    print(f"Bundling {total} chunks into {bundle_count} files ({CHUNKS_PER_BUNDLE} chunks each)...", flush=True)

    for bundle_idx in range(bundle_count):
        start = bundle_idx * CHUNKS_PER_BUNDLE
        end = min(start + CHUNKS_PER_BUNDLE, total)
        bundle: dict[str, list] = {}

        for f in files[start:end]:
            key = f.stem
            with f.open("r", encoding="utf-8") as fh:
                bundle[key] = json.load(fh)

        out_path = out_dir / f"data_bundle_{bundle_idx}.json"
        out_path.write_text(
            json.dumps(bundle, sort_keys=True, ensure_ascii=False, separators=(",", ":")),
            encoding="utf-8",
        )
        size_mb = out_path.stat().st_size / (1024 * 1024)
        chunks = end - start
        print(f"  {out_path.name}: {chunks} chunks, {size_mb:.1f} MB", flush=True)

    print(f"Done: {bundle_count} bundles", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
