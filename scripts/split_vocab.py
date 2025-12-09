from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Iterator, Mapping, MutableMapping, Sequence, Tuple


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Split vocabulary JSON files into chunks of N entries."
    )
    parser.add_argument(
        "files",
        nargs="+",
        type=Path,
        help="Пути к исходным vocabulary_*.json файлам.",
    )
    parser.add_argument(
        "--chunk-size",
        type=int,
        default=1000,
        help="Сколько записей помещать в один файл (по умолчанию 1000).",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=None,
        help="Куда писать результат (по умолчанию папка исходного файла).",
    )
    return parser.parse_args()


def load_json(path: Path) -> Mapping[str, MutableMapping]:
    with path.open("r", encoding="utf-8") as file:
        return json.load(file)


def chunk_items(
    items: Sequence[Tuple[str, MutableMapping]], size: int
) -> Iterator[Sequence[Tuple[str, MutableMapping]]]:
    for start in range(0, len(items), size):
        yield items[start : start + size]


def write_chunk(
    src: Path,
    dst_dir: Path,
    chunk_index: int,
    chunk: Sequence[Tuple[str, MutableMapping]],
) -> Path:
    dst_dir.mkdir(parents=True, exist_ok=True)
    output_path = dst_dir / f"{src.stem}_part{chunk_index + 1}{src.suffix}"
    with output_path.open("w", encoding="utf-8") as file:
        json.dump(dict(chunk), file, ensure_ascii=False, indent=2)
        file.write("\n")
    return output_path


def split_file(path: Path, chunk_size: int, output_dir: Path | None) -> None:
    if chunk_size <= 0:
        raise SystemExit("--chunk-size должно быть положительным числом")

    data = load_json(path)
    if not isinstance(data, Mapping):
        raise SystemExit(f"Ожидался объект JSON в {path}")

    items = list(data.items())
    if not items:
        print(f"{path.name}: нет записей, пропускаю")
        return

    target_dir = output_dir or path.parent
    for index, chunk in enumerate(chunk_items(items, chunk_size)):
        dst = write_chunk(path, target_dir, index, chunk)
        print(f"{path.name} [{len(chunk)} записей] -> {dst.name}")


def main() -> None:
    args = parse_args()
    for file_path in args.files:
        split_file(file_path, args.chunk_size, args.output_dir)


if __name__ == "__main__":
    main()


