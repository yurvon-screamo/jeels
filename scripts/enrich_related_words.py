from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path
from typing import Dict, Iterable, List, Tuple

import requests
import tomllib


def load_config(path: Path) -> Tuple[str, str, str | None]:
    with path.open("rb") as fh:
        cfg = tomllib.load(fh)

    def pick(d: Dict) -> Tuple[str | None, str | None, str | None]:
        return d.get("model"), d.get("base_url"), d.get("env_var_name")

    candidates = [
        pick(cfg),
        pick(cfg.get("reranker", {})),
        pick(cfg.get("reranker", {}).get("openai", {})),
    ]
    for model, base_url, env_var in candidates:
        if model and base_url:
            return str(model), str(base_url).rstrip("/"), env_var

    raise ValueError(
        "config.toml должен содержать model и base_url (например в [reranker.openai])"
    )


def build_score_payload(model: str, query: str, candidates: List[str]) -> Dict:
    return {"model": model, "text_1": query, "text_2": candidates}


def rerank(
    session: requests.Session,
    endpoint: str,
    headers: Dict[str, str],
    model: str,
    query: str,
    candidates: List[str],
) -> List[float]:
    payload = build_score_payload(model, query, candidates)
    resp = session.post(endpoint, headers=headers, json=payload, timeout=60)
    resp.raise_for_status()
    data = resp.json()
    scores = [item.get("score", 0.0) for item in data.get("data", [])]
    return scores


def render_progress(done: int, total: int) -> None:
    if total == 0:
        return
    pct = int(done / total * 100)
    bar_len = 28
    filled = int(bar_len * pct / 100)
    bar = "#" * filled + "-" * (bar_len - filled)
    sys.stdout.write(f"\r    [{bar}] {pct:3d}% ({done}/{total})")
    if done == total:
        sys.stdout.write("\n")
    sys.stdout.flush()


def atomic_write_json(path: Path, data: Dict) -> None:
    """Безопасная запись JSON: сначала во временный файл, потом replace."""
    tmp_path = path.with_suffix(path.suffix + ".tmp")
    tmp_path.write_text(
        json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    tmp_path.replace(path)


def best_related(
    session: requests.Session,
    endpoint: str,
    headers: Dict[str, str],
    model: str,
    word: str,
    entry: Dict,
    candidate_items: List[Tuple[str, Dict]],
    threshold: float,
) -> List[Tuple[str, float]]:
    query = " | ".join(
        filter(
            None,
            [
                f"Find related words (synonyms, antonyms, usage-near) for: {word}",
                entry.get("english_translation", ""),
                entry.get("russian_translation", ""),
            ],
        )
    )
    top: List[Tuple[str, float]] = []
    candidate_texts = [
        " | ".join(
            filter(
                None,
                [
                    cand_word,
                    cand_entry.get("english_translation", ""),
                    cand_entry.get("russian_translation", ""),
                ],
            )
        )
        for cand_word, cand_entry in candidate_items
    ]
    scores = rerank(session, endpoint, headers, model, query, candidate_texts)
    for (cand_word, _), score in zip(candidate_items, scores):
        if score >= threshold:
            top.append((cand_word, float(score)))
    top.sort(key=lambda x: x[1], reverse=True)
    return top


def process_file(
    path: Path,
    session: requests.Session,
    endpoint: str,
    headers: Dict[str, str],
    model: str,
    threshold: float,
    all_candidates: List[Tuple[str, Dict]],
    skip_existing: bool,
) -> None:
    print(f"[+] {path.name}: загрузка...")
    data: Dict[str, Dict] = json.loads(path.read_text(encoding="utf-8"))

    keys = list(data.keys())

    render_progress(0, len(keys))
    for idx, word in enumerate(keys, 1):
        entry = data[word]
        if skip_existing and "related_words" in entry:
            render_progress(idx, len(keys))
            continue
        candidate_items = [
            (cand, cand_entry) for cand, cand_entry in all_candidates if cand != word
        ]

        related = best_related(
            session=session,
            endpoint=endpoint,
            headers=headers,
            model=model,
            word=word,
            entry=entry,
            candidate_items=candidate_items,
            threshold=threshold,
        )

        entry["related_words"] = [
            {
                "word": cand,
                "score": round(score, 4),
            }
            for cand, score in related
        ]

        render_progress(idx, len(keys))
        atomic_write_json(path, data)

    print(f"[+] {path.name}: сохранено")


def main(argv: Iterable[str]) -> int:
    parser = argparse.ArgumentParser(
        description="Обогащение словарей полем related_words через reranker"
    )
    parser.add_argument(
        "--vocab-dir",
        default="vocabulary",
        help="Каталог с *.json (по умолчанию vocabulary)",
    )
    parser.add_argument(
        "--config",
        default="config.toml",
        help="Путь до config.toml с полями model, base_url, env_var_name",
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=0.98,
        help="Минимальный score для включения",
    )
    parser.add_argument(
        "--skip-existing",
        action="store_true",
        help="Пропускать слова, у которых уже есть related_words",
    )
    args = parser.parse_args(list(argv))

    model, base_url, env_var = load_config(Path(args.config))
    api_key = os.getenv(env_var) if env_var else None
    endpoint = f"{base_url}/score"

    headers = {
        "accept": "application/json",
        "Content-Type": "application/json",
    }
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"

    vocab_dir = Path(args.vocab_dir)
    files = sorted(vocab_dir.glob("n*_part*.json"))
    if not files:
        raise FileNotFoundError(f"Файлы n*_part*.json не найдены в {vocab_dir}")

    # Загружаем все файлы сразу, чтобы подбирать связанные слова кросс-файлово.
    all_data: Dict[Path, Dict[str, Dict]] = {
        path: json.loads(path.read_text(encoding="utf-8")) for path in files
    }
    global_candidates: List[Tuple[str, Dict]] = []
    for data in all_data.values():
        for w, meta in data.items():
            global_candidates.append((w, meta))

    print(f"[+] Загружено {len(global_candidates)} кандидатов")
    with requests.Session() as session:
        for path in files:
            process_file(
                path=path,
                session=session,
                endpoint=endpoint,
                headers=headers,
                model=model,
                threshold=args.threshold,
                all_candidates=global_candidates,
                skip_existing=args.skip_existing,
            )

    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
