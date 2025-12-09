#!/usr/bin/env python3
"""
Script to evaluate translation correctness using LLM to detect incorrect translations.

Usage:
    python evaluate_translations.py

    # Process specific files
    python evaluate_translations.py --files vocabulary_n5.json vocabulary_n4.json

    # Override API settings
    python evaluate_translations.py --base-url http://127.0.0.1:8001/v1 --model "llm_instruct"

    # Set API key
    python evaluate_translations.py --api-key YOUR_API_KEY
    # or set environment variable: export OPENROUTER_API_KEY=YOUR_API_KEY

The script will:
1. Load vocabulary files from words/ directory (or specified directory)
2. For each word, use LLM to check if translations and examples are correct (not lies)
3. Save results to evaluation_results/ directory

Results include only word and overall score for each word.
Score: 1.0 = correct, 0.0 = incorrect/lying translation
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


def build_validation_prompt(japanese_text: str, translation: str, language: str) -> str:
    """Build prompt for LLM to validate translation correctness."""
    lang_name = "русском" if language == "ru" else "английском"

    return f"""Проверь правильность перевода японского текста на {lang_name} язык.

Японский текст: "{japanese_text}"
Перевод на {lang_name}: "{translation}"

Твоя задача: определить, является ли перевод ПРАВИЛЬНЫМ или это ВРАНЬЕ/НЕПРАВИЛЬНЫЙ перевод.

Важно:
- Перевод должен быть семантически правильным
- Не должно быть явных ошибок или вранья
- Небольшие неточности допустимы, но грубые ошибки - нет

Ответь ТОЛЬКО одним словом:
- "correct" если перевод правильный
- "incorrect" если перевод неправильный или содержит вранье

Ответ:"""


async def validate_translation_with_llm(
    session: aiohttp.ClientSession,
    semaphore: asyncio.Semaphore,
    japanese_text: str,
    translation: str,
    language: str,
    base_url: str,
    model: str,
    temperature: float,
    api_key: Optional[str] = None,
    debug: bool = False,
) -> Optional[float]:
    """
    Validate translation correctness using LLM.

    Returns: 1.0 if correct, 0.0 if incorrect, None if request fails.
    """
    async with semaphore:
        headers = {"Content-Type": "application/json"}

        if api_key:
            headers["Authorization"] = f"Bearer {api_key}"

        url = base_url.rstrip("/") + "/chat/completions"
        prompt = build_validation_prompt(japanese_text, translation, language)

        payload = {
            "model": model,
            "messages": [{"role": "user", "content": prompt}],
            "temperature": temperature,
        }

        try:
            async with session.post(
                url,
                headers=headers,
                json=payload,
                timeout=aiohttp.ClientTimeout(total=60),
            ) as response:
                if response.status != 200:
                    error_text = await response.text()
                    if debug:
                        print(f"Error {response.status}: {error_text[:200]}")
                    return None

                response.raise_for_status()
                data = await response.json()

                # Handle different response formats
                content = None
                if "choices" in data and len(data["choices"]) > 0:
                    choice = data["choices"][0]
                    if "message" in choice:
                        message = choice["message"]
                        # Try content first
                        content = message.get("content")
                        # If content is None, try reasoning fields (for thinking models)
                        if not content:
                            content = message.get("reasoning_content") or message.get(
                                "reasoning"
                            )

                    elif "text" in choice:
                        content = choice.get("text", "")

                if not content:
                    if debug:
                        print(f"No content in response: {data}")
                    return None

                content_lower = content.strip().lower()

                # Check if LLM says translation is correct
                # Look for "correct" but not "incorrect"
                if "incorrect" in content_lower:
                    return 0.0
                elif "correct" in content_lower:
                    return 1.0
                else:
                    # Try to parse JSON if LLM returned structured response
                    try:
                        parsed = json.loads(content)
                        if isinstance(parsed, dict):
                            if (
                                parsed.get("correct") is True
                                or parsed.get("result") == "correct"
                            ):
                                return 1.0
                            elif (
                                parsed.get("correct") is False
                                or parsed.get("result") == "incorrect"
                            ):
                                return 0.0
                    except (json.JSONDecodeError, ValueError):
                        pass

                    # Default to incorrect if unclear response
                    if debug:
                        print(f"Unclear response: {content[:100]}")
                    return 0.0

        except asyncio.TimeoutError:
            if debug:
                print(f"Timeout for: {japanese_text[:30]}...")
            return None
        except Exception as e:
            if debug:
                print(f"Exception: {type(e).__name__}: {str(e)[:200]}")
            return None


async def evaluate_word(
    session: aiohttp.ClientSession,
    semaphore: asyncio.Semaphore,
    word: str,
    word_data: Dict[str, Any],
    base_url: str,
    model: str,
    temperature: float,
    api_key: Optional[str] = None,
    debug: bool = False,
) -> Dict[str, Any]:
    """
    Evaluate translation correctness for a single word using LLM.

    Returns only word and overall score.
    Score: 1.0 = all translations correct, 0.0 = at least one is incorrect/lying
    """
    all_scores = []

    # Validate Russian translation
    russian_translation = word_data.get("russian_translation", "")
    if russian_translation:
        score = await validate_translation_with_llm(
            session,
            semaphore,
            word,
            russian_translation,
            "ru",
            base_url,
            model,
            temperature,
            api_key,
            debug,
        )
        if score is not None:
            all_scores.append(score)

    # Validate English translation
    english_translation = word_data.get("english_translation", "")
    if english_translation:
        score = await validate_translation_with_llm(
            session,
            semaphore,
            word,
            english_translation,
            "en",
            base_url,
            model,
            temperature,
            api_key,
            debug,
        )
        if score is not None:
            all_scores.append(score)

    # Validate Russian examples
    russian_examples = word_data.get("russian_examples", [])
    for example in russian_examples:
        japanese_text = example.get("text", "")
        russian_text = example.get("translation", "")
        if japanese_text and russian_text:
            score = await validate_translation_with_llm(
                session,
                semaphore,
                japanese_text,
                russian_text,
                "ru",
                base_url,
                model,
                temperature,
                api_key,
                debug,
            )
            if score is not None:
                all_scores.append(score)

    # Validate English examples
    english_examples = word_data.get("english_examples", [])
    for example in english_examples:
        japanese_text = example.get("text", "")
        english_text = example.get("translation", "")
        if japanese_text and english_text:
            score = await validate_translation_with_llm(
                session,
                semaphore,
                japanese_text,
                english_text,
                "en",
                base_url,
                model,
                temperature,
                api_key,
                debug,
            )
            if score is not None:
                all_scores.append(score)

    # Calculate overall score
    # If any translation is incorrect (0.0), overall score is 0.0
    # Otherwise, average of all scores
    # If no scores were obtained, return None
    overall_score = None
    if all_scores:
        if 0.0 in all_scores:
            overall_score = 0.0  # At least one translation is wrong
        else:
            overall_score = sum(all_scores) / len(all_scores)
    elif not all_scores:
        # No scores obtained - all requests failed
        overall_score = None

    return {
        "word": word,
        "score": overall_score,
    }


async def process_vocabulary_file(
    file_path: Path,
    base_url: str,
    model: str,
    temperature: float,
    api_key: Optional[str],
    output_dir: Path,
    max_concurrent: int,
    debug: bool = False,
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
                session,
                semaphore,
                word,
                word_data,
                base_url,
                model,
                temperature,
                api_key,
                debug,
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
    parser.add_argument("--base-url", type=str, help="Override LLM base URL")
    parser.add_argument("--model", type=str, help="Override LLM model name")
    parser.add_argument(
        "--temperature",
        type=float,
        help="Temperature for LLM (default from config)",
    )
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
        default=30,
        help="Maximum number of concurrent requests",
    )
    parser.add_argument(
        "--files",
        type=str,
        nargs="+",
        help="Specific vocabulary files to process (e.g., vocabulary_n5.json)",
    )
    parser.add_argument(
        "--debug",
        action="store_true",
        help="Enable debug mode to see error messages",
    )

    args = parser.parse_args()

    # Load configuration
    config = load_config(args.config)
    llm_config = config.get("llm", {}).get("openai", {})

    # Get API settings
    default_base_url = "http://127.0.0.1:8001/v1"
    default_model = "llm_thinking"

    base_url = args.base_url or llm_config.get("base_url", default_base_url)
    model = args.model or llm_config.get("model", default_model)
    temperature = args.temperature or llm_config.get("temperature", 0.3)
    env_var_name = llm_config.get("env_var_name", "OPENROUTER_API_KEY")
    api_key = args.api_key or os.getenv(env_var_name)

    print("Using LLM for translation validation:")
    print(f"  Base URL: {base_url}")
    print(f"  Model: {model}")
    print(f"  Temperature: {temperature}")
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
                    temperature,
                    api_key,
                    Path(args.output_dir),
                    args.max_concurrent,
                    args.debug,
                )
            else:
                print(f"Warning: File {vocab_file} does not exist, skipping")

    asyncio.run(process_all_files())


if __name__ == "__main__":
    main()
