#!/usr/bin/env python3
"""
Script to evaluate translation and example quality for words using reranker model.

Usage:
    python evaluate_translations.py

    # Process specific files
    python evaluate_translations.py --files vocabulary_n5.json vocabulary_n4.json

    # Override API settings
    python evaluate_translations.py --base-url http://127.0.0.1:8000 --model "BAAI/bge-reranker-v2-m3"

    # Set API key
    python evaluate_translations.py --api-key YOUR_API_KEY
    # or set environment variable: export OPENROUTER_API_KEY=YOUR_API_KEY

The script will:
1. Load vocabulary files from words/ directory (or specified directory)
2. For each word, calculate overall quality score based on translations and examples
3. Save results to evaluation_results/ directory

Results include only word and overall score for each word.
"""

import argparse
import asyncio
import json
import os
import sys
from pathlib import Path
from typing import Any, Dict, Optional

import aiohttp
from tqdm.asyncio import tqdm


def load_config(config_path: str = "config.toml") -> Dict[str, Any]:
    """Load configuration from TOML file."""
    import tomllib

    with open(config_path, "rb") as f:
        return tomllib.load(f)


async def score_with_reranker(
    session: aiohttp.ClientSession,
    semaphore: asyncio.Semaphore,
    text_1: str,
    text_2: str,
    base_url: str,
    model: str,
    api_key: Optional[str] = None,
    encoding_format: str = "float",
) -> Optional[float]:
    """
    Score similarity between two texts using reranker API.

    Returns the score or None if request fails.
    """
    async with semaphore:
        headers = {"accept": "application/json", "Content-Type": "application/json"}

        if api_key:
            headers["Authorization"] = f"Bearer {api_key}"

        # Build score URL - try /score endpoint first (from example)
        # If base_url ends with /v1, remove it and add /score
        if base_url.endswith("/v1"):
            score_url = base_url[:-3] + "/score"
        elif base_url.endswith("/v1/"):
            score_url = base_url[:-4] + "/score"
        else:
            score_url = base_url.rstrip("/") + "/score"

        payload = {
            "model": model,
            "encoding_format": encoding_format,
            "text_1": text_1,
            "text_2": text_2,
        }

        try:
            async with session.post(
                score_url,
                headers=headers,
                json=payload,
                timeout=aiohttp.ClientTimeout(total=30),
            ) as response:
                response.raise_for_status()
                data = await response.json()

                # Extract score from response (format from example)
                if (
                    "data" in data
                    and isinstance(data["data"], list)
                    and len(data["data"]) > 0
                ):
                    return data["data"][0].get("score")
                elif "score" in data:
                    return data["score"]
                else:
                    return None

        except Exception:
            return None


async def evaluate_word(
    session: aiohttp.ClientSession,
    semaphore: asyncio.Semaphore,
    word: str,
    word_data: Dict[str, Any],
    base_url: str,
    model: str,
    api_key: Optional[str] = None,
) -> Dict[str, Any]:
    """
    Evaluate translation and example quality for a single word.

    Returns only word and overall score.
    """
    all_scores = []

    # Score Russian translation
    russian_translation = word_data.get("russian_translation", "")
    if russian_translation:
        score = await score_with_reranker(
            session, semaphore, word, russian_translation, base_url, model, api_key
        )
        if score is not None:
            all_scores.append(score)

    # Score English translation
    english_translation = word_data.get("english_translation", "")
    if english_translation:
        score = await score_with_reranker(
            session, semaphore, word, english_translation, base_url, model, api_key
        )
        if score is not None:
            all_scores.append(score)

    # Score Russian examples
    russian_examples = word_data.get("russian_examples", [])
    for example in russian_examples:
        japanese_text = example.get("text", "")
        russian_text = example.get("translation", "")
        if japanese_text and russian_text:
            score = await score_with_reranker(
                session,
                semaphore,
                japanese_text,
                russian_text,
                base_url,
                model,
                api_key,
            )
            if score is not None:
                all_scores.append(score)

    # Score English examples
    english_examples = word_data.get("english_examples", [])
    for example in english_examples:
        japanese_text = example.get("text", "")
        english_text = example.get("translation", "")
        if japanese_text and english_text:
            score = await score_with_reranker(
                session,
                semaphore,
                japanese_text,
                english_text,
                base_url,
                model,
                api_key,
            )
            if score is not None:
                all_scores.append(score)

    # Calculate overall score
    overall_score = None
    if all_scores:
        overall_score = sum(all_scores) / len(all_scores)

    return {
        "word": word,
        "score": overall_score,
    }


async def process_vocabulary_file(
    file_path: Path,
    base_url: str,
    model: str,
    api_key: Optional[str] = None,
    output_dir: Path = Path("evaluation_results"),
    max_concurrent: int = 20,
) -> None:
    """Process a single vocabulary JSON file."""
    print(f"\nProcessing {file_path.name}...")

    with open(file_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    total_words = len(data)
    semaphore = asyncio.Semaphore(max_concurrent)

    print(
        f"  Evaluating {total_words} words with {max_concurrent} concurrent requests..."
    )

    # Process all words concurrently with progress bar
    results = []
    async with aiohttp.ClientSession() as session:
        # Create tasks for parallel processing
        tasks = []
        for word, word_data in data.items():
            task = evaluate_word(
                session, semaphore, word, word_data, base_url, model, api_key
            )
            tasks.append(task)

        for coro in tqdm.as_completed(
            tasks, desc=f"Evaluating {file_path.stem}", total=len(tasks)
        ):
            result = await coro
            results.append(result)

    # Sort results by word to maintain consistency
    results.sort(key=lambda x: x["word"])

    # Save results
    output_dir.mkdir(parents=True, exist_ok=True)
    output_file = output_dir / f"{file_path.stem}_evaluation.json"

    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(results, f, ensure_ascii=False, indent=2)

    # Print summary statistics
    print(f"\nSummary for {file_path.name}:")
    print(f"  Total words: {total_words}")

    scores = [r["score"] for r in results if r["score"] is not None]
    if scores:
        print(f"  Average score: {sum(scores) / len(scores):.4f}")
        print(f"  Min score: {min(scores):.4f}")
        print(f"  Max score: {max(scores):.4f}")

    print(f"  Results saved to: {output_file}")


def main():
    parser = argparse.ArgumentParser(
        description="Evaluate translation and example quality using reranker model"
    )
    parser.add_argument(
        "--words-dir",
        type=str,
        default="words",
        help="Directory containing vocabulary JSON files",
    )
    parser.add_argument(
        "--config", type=str, default="config.toml", help="Path to config.toml file"
    )
    parser.add_argument("--base-url", type=str, help="Override reranker base URL")
    parser.add_argument("--model", type=str, help="Override reranker model name")
    parser.add_argument(
        "--api-key", type=str, help="API key (or set via environment variable)"
    )
    parser.add_argument(
        "--output-dir",
        type=str,
        default="evaluation_results",
        help="Output directory for evaluation results",
    )
    parser.add_argument(
        "--max-concurrent",
        type=int,
        default=20,
        help="Maximum number of concurrent requests",
    )
    parser.add_argument(
        "--files",
        type=str,
        nargs="+",
        help="Specific vocabulary files to process (e.g., vocabulary_n5.json)",
    )

    args = parser.parse_args()

    # Load configuration
    config = load_config(args.config)
    reranker_config = config.get("reranker", {}).get("openai", {})

    # Get API settings
    # Default to example URL if not in config
    default_base_url = "http://127.0.0.1:8000"
    default_model = "BAAI/bge-reranker-v2-m3"

    base_url = args.base_url or reranker_config.get("base_url", default_base_url)
    model = args.model or reranker_config.get("model", default_model)
    env_var_name = reranker_config.get("env_var_name", "OPENROUTER_API_KEY")
    api_key = args.api_key or os.getenv(env_var_name)

    print("Using reranker:")
    print(f"  Base URL: {base_url}")
    print(f"  Model: {model}")
    print(f"  API Key: {'***' if api_key else 'Not set'}")

    # Find vocabulary files
    words_dir = Path(args.words_dir)
    if not words_dir.exists():
        print(f"Error: Directory {words_dir} does not exist")
        sys.exit(1)

    if args.files:
        vocabulary_files = [words_dir / f for f in args.files]
    else:
        vocabulary_files = list(words_dir.glob("vocabulary_*.json"))

    if not vocabulary_files:
        print(f"No vocabulary files found in {words_dir}")
        sys.exit(1)

    print(f"\nFound {len(vocabulary_files)} vocabulary file(s)")

    # Process each file
    async def process_all_files():
        for vocab_file in vocabulary_files:
            if vocab_file.exists():
                await process_vocabulary_file(
                    vocab_file,
                    base_url,
                    model,
                    api_key,
                    Path(args.output_dir),
                    args.max_concurrent,
                )
            else:
                print(f"Warning: File {vocab_file} does not exist, skipping")

    asyncio.run(process_all_files())


if __name__ == "__main__":
    main()
