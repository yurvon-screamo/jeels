#!/usr/bin/env python3


import argparse
import asyncio
import json
import os
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

import aiohttp
from tqdm.asyncio import tqdm


def load_config(config_path: str = "config.toml") -> Dict[str, Any]:
    """Load configuration from TOML file."""
    import tomllib

    with open(config_path, "rb") as f:
        return tomllib.load(f)


def build_improved_prompt(
    word: str,
    japanese_level: str,
    current_russian_translation: str,
    current_english_translation: str,
    current_russian_examples: List[Dict[str, str]],
    current_english_examples: List[Dict[str, str]],
    current_part_of_speech: str,
) -> str:
    """Build improved prompt with current (incorrect) translation information."""
    russian_examples_str = json.dumps(
        current_russian_examples, ensure_ascii=False, indent=2
    )
    english_examples_str = json.dumps(
        current_english_examples, ensure_ascii=False, indent=2
    )

    return f"""Ты — помощник для изучения языков.
Твоя задача: Создай переводы и примеры использования слова: '{word}' для студентов уровня {japanese_level}.

ВАЖНО: Текущий перевод и примеры были оценены как недостаточно качественные. 
Твоя задача — создать более точные и качественные переводы и примеры.

Текущие некорректные перевод и примеры:

- Русский: "{current_russian_translation}"
- Английский: "{current_english_translation}"
- Часть речи: {current_part_of_speech}

Текущие русские примеры:
{russian_examples_str}

Текущие английские примеры:
{english_examples_str}

Требования к переводам:
1. Русский перевод: ответь 1 предложением на русском языке. Должен быть более точным и понятным чем текущий.
2. Английский перевод: ответь 1 предложением на английском языке. Должен быть более точным и понятным чем текущий.
3. Не повторяй слово в ответе, потому что твой ответ будет использоваться как обратная сторона карточки и нужно иметь возможность их переворачивать и прогонять в обратном направлении.
4. Не указывай в ответе чтение или транскрипцию, студент умеет читать.
5. Выдай просто ответ без вводных или объяснений зачем и для кого это.
6. Если слово состоит из 1 кандзи, то объясни его значение как слово, а не как кандзи.
7. Учти контекст и уровень сложности слова для уровня {japanese_level}.

Требования к примерам:
1. Создай 2 простых примера использования слова для каждого языка. Примеры должны быть ЛУЧШЕ текущих.
2. Максимально простая грамматика.
3. Короткие простые предложения.
4. Ориентируйся на уровень {japanese_level}.
5. Примеры должны быть более естественными и понятными чем текущие.

Требования к части речи:
Определи часть речи слова и верни одно из: Noun, Verb, Adjective, Adverb, Pronoun, Preposition, Conjunction, Interjection, Particle, Other.
Если текущая часть речи "{current_part_of_speech}" корректна, используй её. Если нет — исправь.

Ответ должен быть СТРОГО валидным JSON, без markdown разметки (без ```json):
{{
  "russian_translation": "улучшенный перевод на русском",
  "english_translation": "improved translation in English",
  "part_of_speech": "Noun",
  "russian_examples": [
    {{
      "text": "предложение на японском",
      "translation": "перевод предложения на русском"
    }}
  ],
  "english_examples": [
    {{
      "text": "предложение на японском",
      "translation": "translation of the sentence in English"
    }}
  ]
}}"""


async def call_llm(
    session: aiohttp.ClientSession,
    prompt: str,
    base_url: str,
    model: str,
    temperature: float,
    api_key: Optional[str] = None,
) -> Optional[str]:
    """Call LLM API with OpenAI-compatible format."""
    headers = {
        "Content-Type": "application/json",
    }

    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"

    url = base_url.rstrip("/") + "/chat/completions"

    payload = {
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "temperature": temperature,
    }

    try:
        async with session.post(
            url, headers=headers, json=payload, timeout=aiohttp.ClientTimeout(total=120)
        ) as response:
            response.raise_for_status()
            data = await response.json()

            # Handle different response formats (including thinking models)
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

            return content

    except Exception as e:
        print(f"Error calling LLM: {e}")
        return None


def clean_json_response(response: str) -> str:
    """Clean JSON response from markdown formatting."""
    return response.strip().replace("```json", "").replace("```", "").strip()


def parse_llm_response(response: str) -> Optional[Dict[str, Any]]:
    """Parse LLM JSON response."""
    try:
        cleaned = clean_json_response(response)
        return json.loads(cleaned)
    except json.JSONDecodeError as e:
        print(f"Failed to parse JSON: {e}")
        print(f"Response: {response[:500]}")
        return None


async def regenerate_word_content(
    session: aiohttp.ClientSession,
    semaphore: asyncio.Semaphore,
    word: str,
    word_data: Dict[str, Any],
    base_url: str,
    model: str,
    temperature: float,
    api_key: Optional[str] = None,
) -> tuple[str, Optional[Dict[str, Any]]]:
    """Regenerate content for a single word."""
    async with semaphore:
        japanese_level = word_data.get("level", "N5")
        current_russian = word_data.get("russian_translation", "")
        current_english = word_data.get("english_translation", "")
        current_russian_examples = word_data.get("russian_examples", [])
        current_english_examples = word_data.get("english_examples", [])
        current_pos = word_data.get("part_of_speech", "Other")

        prompt = build_improved_prompt(
            word,
            japanese_level,
            current_russian,
            current_english,
            current_russian_examples,
            current_english_examples,
            current_pos,
        )

        response = await call_llm(
            session, prompt, base_url, model, temperature, api_key
        )
        if not response:
            return word, None

        parsed = parse_llm_response(response)
        return word, parsed


async def regenerate_word_content_with_retries(
    session: aiohttp.ClientSession,
    semaphore: asyncio.Semaphore,
    word: str,
    word_data: Dict[str, Any],
    base_url: str,
    model: str,
    temperature: float,
    api_key: Optional[str] = None,
    max_retries: int = 3,
) -> tuple[str, Optional[Dict[str, Any]], int]:
    """
    Regenerate content for a single word with retries.

    Returns: (word, content, retry_count)
    """
    for attempt in range(max_retries):
        result_word, result_content = await regenerate_word_content(
            session,
            semaphore,
            word,
            word_data,
            base_url,
            model,
            temperature,
            api_key,
        )

        if result_content is not None:
            return result_word, result_content, attempt

        # Wait before retry
        if attempt < max_retries - 1:
            await asyncio.sleep(1)

    return word, None, max_retries


def find_words_to_regenerate(
    evaluation_dir: Path = Path("evaluation_results"),
    threshold: float = 1.0,
) -> Dict[str, List[Dict[str, Any]]]:
    """
    Find all words with incorrect translations (score < 1.0).

    New format: 1.0 = correct, 0.0 = incorrect/lying, None = failed to check
    """
    words_to_regenerate = {}

    evaluation_files = sorted(evaluation_dir.glob("*_evaluation.json"))

    for eval_file in evaluation_files:
        level = (
            eval_file.stem.replace("vocabulary_", "").replace("_evaluation", "").upper()
        )

        with open(eval_file, "r", encoding="utf-8") as f:
            evaluation_data = json.load(f)

        # Find words with score < 1.0 (includes 0.0 and None)
        # Score 0.0 = incorrect translation, None = failed to validate
        low_score_words = [
            item
            for item in evaluation_data
            if item.get("score") is None or item.get("score", 1.0) < threshold
        ]

        if low_score_words:
            words_to_regenerate[level] = low_score_words

    return words_to_regenerate


def load_vocabulary_file(words_dir: Path, level: str) -> Dict[str, Any]:
    """Load vocabulary file for a specific level."""
    vocab_file = words_dir / f"vocabulary_{level.lower()}.json"
    if not vocab_file.exists():
        return {}

    with open(vocab_file, "r", encoding="utf-8") as f:
        return json.load(f)


def save_vocabulary_file(words_dir: Path, level: str, data: Dict[str, Any]) -> None:
    """Save vocabulary file for a specific level."""
    vocab_file = words_dir / f"vocabulary_{level.lower()}.json"

    # Create backup
    if vocab_file.exists():
        backup_file = vocab_file.with_suffix(".json.backup")
        import shutil

        shutil.copy2(vocab_file, backup_file)
        print(f"  Backup created: {backup_file}")

    with open(vocab_file, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)


async def regenerate_words(
    words_dir: Path,
    evaluation_dir: Path,
    base_url: str,
    model: str,
    temperature: float,
    api_key: Optional[str],
    threshold: float,
    max_concurrent: int = 30,
    max_retries: int = 3,
    dry_run: bool = False,
    output_file: Optional[str] = None,
) -> None:
    """Regenerate content for all words with low scores."""
    print("Finding words with incorrect translations to regenerate...")
    words_to_regenerate = find_words_to_regenerate(evaluation_dir, threshold)

    total_words = sum(len(words) for words in words_to_regenerate.values())
    incorrect_count = sum(
        len([w for w in words if w.get("score") == 0.0])
        for words in words_to_regenerate.values()
    )
    failed_count = sum(
        len([w for w in words if w.get("score") is None])
        for words in words_to_regenerate.values()
    )

    print(
        f"Found {total_words} words to regenerate across {len(words_to_regenerate)} levels"
    )
    print(f"  - Incorrect translations (score=0.0): {incorrect_count}")
    print(f"  - Failed to validate (score=None): {failed_count}")

    if dry_run:
        print("\nDRY RUN MODE - No changes will be made")
        for level, words in words_to_regenerate.items():
            print(f"\n{level}: {len(words)} words")
            for word_data in words[:5]:  # Show first 5
                print(f"  - {word_data['word']}: score {word_data['score']:.4f}")
        return

    stats = {
        "total": 0,
        "success": 0,
        "failed": 0,
        "skipped": 0,
        "retries": 0,
    }

    # Create semaphore for limiting concurrent requests
    semaphore = asyncio.Semaphore(max_concurrent)

    async with aiohttp.ClientSession() as session:
        for level, words_list in words_to_regenerate.items():
            print(f"\n{'=' * 80}")
            print(f"Processing level {level}: {len(words_list)} words")
            print("=" * 80)

            # Load vocabulary file
            vocabulary = load_vocabulary_file(words_dir, level)
            if not vocabulary:
                print(f"  Warning: Vocabulary file for {level} not found, skipping")
                continue

            # Filter words that exist in vocabulary
            valid_words = []
            skipped_words = []
            for word_data in words_list:
                word = word_data["word"]
                if word not in vocabulary:
                    skipped_words.append(word)
                    continue
                valid_words.append((word_data, vocabulary[word]))

            if skipped_words:
                print(f"  Skipping {len(skipped_words)} words not found in vocabulary")
                stats["skipped"] += len(skipped_words)

            if not valid_words:
                print("  No valid words to process")
                continue

            # Create tasks for parallel processing
            tasks = []
            word_scores = {}
            for word_data, word_vocab_data in valid_words:
                word = word_data["word"]
                score = word_data.get("score", 0.0)
                word_scores[word] = score
                stats["total"] += 1

                task = regenerate_word_content_with_retries(
                    session,
                    semaphore,
                    word,
                    word_vocab_data,
                    base_url,
                    model,
                    temperature,
                    api_key,
                    max_retries,
                )
                tasks.append(task)

            # Process all words concurrently with progress bar
            print(
                f"  Regenerating {len(tasks)} words with {max_concurrent} concurrent requests..."
            )
            results = []
            for coro in tqdm.as_completed(
                tasks, desc=f"Regenerating {level}", total=len(tasks)
            ):
                result = await coro
                results.append(result)

            # Process results
            updated_count = 0
            failed_count = 0
            regenerated_content = {} if output_file else None

            for word, new_content, retry_count in results:
                score = word_scores.get(word, 0.0)

                if retry_count > 0:
                    stats["retries"] += retry_count

                if not new_content:
                    failed_count += 1
                    stats["failed"] += 1
                    continue

                # Store regenerated content
                if output_file:
                    regenerated_content[word] = {
                        "level": level,
                        "old_score": score,
                        "old_content": {
                            "russian_translation": vocabulary[word].get(
                                "russian_translation", ""
                            ),
                            "english_translation": vocabulary[word].get(
                                "english_translation", ""
                            ),
                            "part_of_speech": vocabulary[word].get(
                                "part_of_speech", ""
                            ),
                            "russian_examples": vocabulary[word].get(
                                "russian_examples", []
                            ),
                            "english_examples": vocabulary[word].get(
                                "english_examples", []
                            ),
                        },
                        "new_content": new_content,
                    }
                else:
                    # Update vocabulary entry
                    vocabulary[word]["russian_translation"] = new_content.get(
                        "russian_translation",
                        vocabulary[word].get("russian_translation", ""),
                    )
                    vocabulary[word]["english_translation"] = new_content.get(
                        "english_translation",
                        vocabulary[word].get("english_translation", ""),
                    )
                    vocabulary[word]["part_of_speech"] = new_content.get(
                        "part_of_speech",
                        vocabulary[word].get("part_of_speech", "Other"),
                    )
                    vocabulary[word]["russian_examples"] = new_content.get(
                        "russian_examples", vocabulary[word].get("russian_examples", [])
                    )
                    vocabulary[word]["english_examples"] = new_content.get(
                        "english_examples", vocabulary[word].get("english_examples", [])
                    )

                updated_count += 1
                stats["success"] += 1

            # Save results
            if output_file and regenerated_content:
                # Append to output file
                output_path = Path(output_file)
                all_regenerated = {}
                if output_path.exists():
                    with open(output_path, "r", encoding="utf-8") as f:
                        all_regenerated = json.load(f)
                all_regenerated[level] = regenerated_content

                with open(output_path, "w", encoding="utf-8") as f:
                    json.dump(all_regenerated, f, ensure_ascii=False, indent=2)
                print(f"\n  Saved regenerated content for {level} to {output_file}")
            elif updated_count > 0:
                # Save updated vocabulary
                print(f"\n  Saving {level} vocabulary...")
                save_vocabulary_file(words_dir, level, vocabulary)
                print(
                    f"  Updated {updated_count} words, failed {failed_count}, skipped {len(skipped_words)}"
                )

    # Print final statistics
    print("\n" + "=" * 80)
    print("FINAL STATISTICS")
    print("=" * 80)
    print(f"Total words processed: {stats['total']}")
    print(f"Successfully regenerated: {stats['success']}")
    print(f"Failed: {stats['failed']}")
    print(f"Skipped: {stats['skipped']}")
    print(f"Total retries: {stats['retries']}")
    print(
        f"Success rate: {(stats['success'] / stats['total'] * 100) if stats['total'] > 0 else 0:.1f}%"
    )


def main():
    parser = argparse.ArgumentParser(
        description="Regenerate content for words with low evaluation scores"
    )
    parser.add_argument(
        "--words-dir",
        type=str,
        default="words",
        help="Directory containing vocabulary JSON files",
    )
    parser.add_argument(
        "--evaluation-dir",
        type=str,
        default="evaluation_results",
        help="Directory containing evaluation result files",
    )
    parser.add_argument(
        "--config",
        type=str,
        default="config.toml",
        help="Path to config.toml file",
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=1.0,
        help="Score threshold (words with score < this will be regenerated). Default: 1.0 (only incorrect translations)",
    )
    parser.add_argument(
        "--max-concurrent",
        type=int,
        default=30,
        help="Maximum number of concurrent requests",
    )
    parser.add_argument(
        "--max-retries",
        type=int,
        default=3,
        help="Maximum number of retries for failed words",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be regenerated without making changes",
    )
    parser.add_argument(
        "--output-file",
        type=str,
        help="Save regenerated content to JSON file instead of updating vocabulary files",
    )

    args = parser.parse_args()

    # Load configuration
    config = load_config(args.config)
    llm_config = config.get("llm", {}).get("openai", {})

    base_url = llm_config.get("base_url", "http://127.0.0.1:8001/v1")
    model = llm_config.get("model", "llm_instruct")
    temperature = llm_config.get("temperature", 0.3)
    env_var_name = llm_config.get("env_var_name", "OPENROUTER_API_KEY")
    api_key = os.getenv(env_var_name)

    print("Configuration:")
    print(f"  Base URL: {base_url}")
    print(f"  Model: {model}")
    print(f"  Temperature: {temperature}")
    print(f"  API Key: {'***' if api_key else 'Not set'}")

    if not api_key and not args.dry_run:
        print("\nWarning: API key not set. Set environment variable or use --dry-run")
        sys.exit(1)

    words_dir = Path(args.words_dir)
    evaluation_dir = Path(args.evaluation_dir)

    if not words_dir.exists():
        print(f"Error: Directory {words_dir} does not exist")
        sys.exit(1)

    if not evaluation_dir.exists():
        print(f"Error: Directory {evaluation_dir} does not exist")
        sys.exit(1)

    asyncio.run(
        regenerate_words(
            words_dir,
            evaluation_dir,
            base_url,
            model,
            temperature,
            api_key,
            args.threshold,
            args.max_concurrent,
            args.max_retries,
            args.dry_run,
            args.output_file,
        )
    )


if __name__ == "__main__":
    main()
