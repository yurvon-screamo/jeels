#!/usr/bin/env python3
"""
Script to analyze evaluation results and generate summary statistics.
"""

import argparse
import json
import statistics
from pathlib import Path
from typing import Any, Dict, List


def load_evaluation_file(file_path: Path) -> List[Dict[str, Any]]:
    """Load evaluation results from JSON file."""
    with open(file_path, "r", encoding="utf-8") as f:
        return json.load(f)


def calculate_statistics(scores: List[float]) -> Dict[str, float]:
    """Calculate statistics for a list of scores."""
    if not scores:
        return {}

    return {
        "count": len(scores),
        "mean": statistics.mean(scores),
        "median": statistics.median(scores),
        "min": min(scores),
        "max": max(scores),
        "stdev": statistics.stdev(scores) if len(scores) > 1 else 0.0,
    }


def analyze_evaluation_results(
    evaluation_dir: Path = Path("evaluation_results"),
) -> Dict[str, Any]:
    """Analyze all evaluation result files."""
    results = {}

    # Find all evaluation files
    evaluation_files = sorted(evaluation_dir.glob("*_evaluation.json"))

    if not evaluation_files:
        print(f"No evaluation files found in {evaluation_dir}")
        return {}

    for file_path in evaluation_files:
        # Extract level from filename (e.g., vocabulary_n5_evaluation.json -> N5)
        level = (
            file_path.stem.replace("vocabulary_", "").replace("_evaluation", "").upper()
        )

        data = load_evaluation_file(file_path)
        scores = [item["score"] for item in data if item.get("score") is not None]
        words_without_score = [
            item["word"] for item in data if item.get("score") is None
        ]

        stats = calculate_statistics(scores)

        # Count words below threshold (0.8)
        words_below_08 = [
            item
            for item in data
            if item.get("score") is not None and item["score"] < 0.8
        ]

        # Find words with best and worst scores
        sorted_data = sorted(
            [item for item in data if item.get("score") is not None],
            key=lambda x: x["score"],
        )

        results[level] = {
            "total_words": len(data),
            "words_with_score": len(scores),
            "words_without_score": len(words_without_score),
            "words_below_08": len(words_below_08),
            "words_below_08_percent": (len(words_below_08) / len(scores) * 100)
            if scores
            else 0,
            "statistics": stats,
            "worst_words": sorted_data[:10] if len(sorted_data) >= 10 else sorted_data,
            "best_words": sorted_data[-10:] if len(sorted_data) >= 10 else sorted_data,
        }

    return results


def print_summary_table(results: Dict[str, Any], evaluation_dir: Path) -> None:
    """Print summary table with statistics."""
    print("\n" + "=" * 100)
    print("ИТОГОВАЯ СТАТИСТИКА ПО КАЧЕСТВУ ПЕРЕВОДОВ И ПРИМЕРОВ")
    print("=" * 100)

    # Header
    print(
        f"\n{'Уровень':<10} {'Всего':<10} {'Со скором':<12} {'< 0.8':<10} {'% < 0.8':<10} "
        f"{'Среднее':<12} {'Медиана':<12} {'Мин':<10} {'Макс':<10} {'Стд. откл.':<12}"
    )
    print("-" * 120)

    # Data rows
    for level in sorted(results.keys()):
        r = results[level]
        stats = r["statistics"]
        if stats:
            print(
                f"{level:<10} {r['total_words']:<10} {r['words_with_score']:<12} "
                f"{r['words_below_08']:<10} {r['words_below_08_percent']:<10.1f} "
                f"{stats['mean']:<12.4f} {stats['median']:<12.4f} "
                f"{stats['min']:<10.4f} {stats['max']:<10.4f} {stats['stdev']:<12.4f}"
            )

    # Overall statistics
    all_scores = []
    total_words = 0
    words_with_score = 0
    total_words_below_08 = 0

    for level_data in results.values():
        total_words += level_data["total_words"]
        words_with_score += level_data["words_with_score"]
        total_words_below_08 += level_data["words_below_08"]

    # Get all scores properly
    for file_path in sorted(evaluation_dir.glob("*_evaluation.json")):
        data = load_evaluation_file(file_path)
        all_scores.extend(
            [item["score"] for item in data if item.get("score") is not None]
        )

    if all_scores:
        overall_stats = calculate_statistics(all_scores)
        overall_below_08_percent = (
            (total_words_below_08 / words_with_score * 100) if words_with_score else 0
        )
        print("-" * 120)
        print(
            f"{'ИТОГО':<10} {total_words:<10} {words_with_score:<12} "
            f"{total_words_below_08:<10} {overall_below_08_percent:<10.1f} "
            f"{overall_stats['mean']:<12.4f} {overall_stats['median']:<12.4f} "
            f"{overall_stats['min']:<10.4f} {overall_stats['max']:<10.4f} {overall_stats['stdev']:<12.4f}"
        )

    print("\n" + "=" * 100)


def print_worst_words(results: Dict[str, Any], top_n: int = 20) -> None:
    """Print words with worst scores."""
    print(f"\n{'=' * 100}")
    print(f"ТОП-{top_n} СЛОВ С НАИХУДШИМИ ОЦЕНКАМИ")
    print("=" * 100)

    all_worst = []
    for level, level_data in results.items():
        for word_data in level_data.get("worst_words", []):
            if word_data.get("score") is not None:
                all_worst.append({**word_data, "level": level})

    all_worst.sort(key=lambda x: x["score"])
    all_worst = all_worst[:top_n]

    print(f"\n{'Уровень':<10} {'Слово':<30} {'Оценка':<10}")
    print("-" * 100)
    for item in all_worst:
        print(f"{item['level']:<10} {item['word']:<30} {item['score']:<10.4f}")


def print_best_words(results: Dict[str, Any], top_n: int = 20) -> None:
    """Print words with best scores."""
    print(f"\n{'=' * 100}")
    print(f"ТОП-{top_n} СЛОВ С НАИЛУЧШИМИ ОЦЕНКАМИ")
    print("=" * 100)

    all_best = []
    for level, level_data in results.items():
        for word_data in level_data.get("best_words", []):
            if word_data.get("score") is not None:
                all_best.append({**word_data, "level": level})

    all_best.sort(key=lambda x: x["score"], reverse=True)
    all_best = all_best[:top_n]

    print(f"\n{'Уровень':<10} {'Слово':<30} {'Оценка':<10}")
    print("-" * 100)
    for item in all_best:
        print(f"{item['level']:<10} {item['word']:<30} {item['score']:<10.4f}")


def print_level_details(results: Dict[str, Any]) -> None:
    """Print detailed statistics for each level."""
    print("\n" + "=" * 100)
    print("ДЕТАЛЬНАЯ СТАТИСТИКА ПО УРОВНЯМ")
    print("=" * 100)

    for level in sorted(results.keys()):
        r = results[level]
        stats = r["statistics"]

        if not stats:
            continue

        print(f"\n{level}:")
        print(f"  Всего слов: {r['total_words']}")
        print(f"  С оценкой: {r['words_with_score']}")
        print(f"  Без оценки: {r['words_without_score']}")
        print(
            f"  С оценкой < 0.8: {r['words_below_08']} ({r['words_below_08_percent']:.1f}%)"
        )
        print(f"  Средняя оценка: {stats['mean']:.4f}")
        print(f"  Медиана: {stats['median']:.4f}")
        print(f"  Минимум: {stats['min']:.4f}")
        print(f"  Максимум: {stats['max']:.4f}")
        print(f"  Стандартное отклонение: {stats['stdev']:.4f}")

        # Score distribution
        if r.get("worst_words") and r.get("best_words"):
            worst_score = r["worst_words"][0]["score"] if r["worst_words"] else None
            best_score = r["best_words"][-1]["score"] if r["best_words"] else None
            if worst_score is not None and best_score is not None:
                print(f"  Диапазон: {worst_score:.4f} - {best_score:.4f}")


def main():
    parser = argparse.ArgumentParser(
        description="Analyze evaluation results and generate summary statistics"
    )
    parser.add_argument(
        "--evaluation-dir",
        type=str,
        default="evaluation_results",
        help="Directory containing evaluation result files",
    )
    parser.add_argument(
        "--top-n",
        type=int,
        default=20,
        help="Number of top/bottom words to show",
    )
    parser.add_argument(
        "--no-details",
        action="store_true",
        help="Skip detailed level statistics",
    )

    args = parser.parse_args()

    evaluation_dir = Path(args.evaluation_dir)

    if not evaluation_dir.exists():
        print(f"Error: Directory {evaluation_dir} does not exist")
        return

    # Analyze results
    results = analyze_evaluation_results(evaluation_dir)

    if not results:
        print("No results to analyze")
        return

    # Print summary table
    print_summary_table(results, evaluation_dir)

    # Print worst words
    print_worst_words(results, args.top_n)

    # Print best words
    print_best_words(results, args.top_n)

    # Print detailed statistics
    if not args.no_details:
        print_level_details(results)

    print("\n" + "=" * 100)


if __name__ == "__main__":
    main()
