# Utils CLI

Unified command-line interface for Japanese tokenization and OCR tools.

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/utils`.

## Usage

The CLI provides four main commands:

### `tokenize` - Tokenize Japanese Text

Tokenizes Japanese text and extracts vocabulary words.

```bash
# Tokenize a text string
utils tokenize "日本語のテキスト"

# Tokenize from a file
utils tokenize -f path/to/file.txt
utils tokenize path/to/file.txt  # Automatically detects if path exists
```

**Options:**

- `TEXT` - Text to tokenize or path to file
- `-f, --file` - Read text from file

### `ndlocr` - Japanese OCR

Performs OCR on Japanese text images using NDLOCR-Lite models.

```bash
# Basic usage
utils ndlocr -i image.png

# With custom model paths
utils ndlocr \
  -i image.png \
  --detector path/to/detector.onnx \
  --rec30 path/to/rec30.onnx \
  --rec50 path/to/rec50.onnx \
  --rec100 path/to/rec100.onnx \
  --vocab path/to/vocab.txt
```

**Options:**

- `-i, --input <INPUT>` - Input image path (required)
- `--detector <DETECTOR>` - Detector model path
  (default: `../ndlocr-lite/src/model/deim-s-1024x1024.onnx`)
- `--rec30 <REC30>` - Parseq 30 model path
- `--rec50 <REC50>` - Parseq 50 model path
- `--rec100 <REC100>` - Parseq
 100 model path
- `--vocab <VOCAB>` - Vocabulary file path (default: `config/NDLmoji.txt`)

### `tokenize-well-known` - Batch Process JSON Files

Batch processes JSON files in well_known_set format, updating
the words arrays with tokenized vocabulary.

```bash
# Process a single file
utils tokenize-well-known path/to/file.json

# Process all JSON files in a directory
utils tokenize-well-known path/to/directory/
```

**Arguments:**

- `PATH` - Path to directory or JSON file

### `find-missing` - Find Missing Vocabulary

Finds vocabulary words from well-known sets that are missing from
the dictionary and optionally generates translations using an
OpenAI-compatible API.

```bash
# Generate a report of missing vocabulary
utils find-missing

# Generate a report with custom output path
utils find-missing -o custom_report.md

# Auto-generate translations for missing words
utils find-missing --generate

# Generate only Russian translations
utils find-missing --generate --russian-only

# Generate only English translations
utils find-missing --generate --english-only

# With custom API settings
utils find-missing \
  --generate \
  --api-base http://localhost:8000/v1 \
  --api-key your-api-key \
  --workers 16
```

**Options:**

- `-o, --output <OUTPUT>` - Output path for the markdown report
  (default: `missing_vocabulary.md` in project root)
- `-g, --generate` - Auto-generate missing words with translations
- `--api-base <API_BASE>` - OpenAI API base URL
  (default: `http://10.2.11.6:8001/v1`)
- `--api-key <API_KEY>` - OpenAI API key (default: `none`)
- `-w, --workers <WORKERS>` - Number of concurrent translation
  requests (default: `32`)
- `--chunk-size <CHUNK_SIZE>` - Chunk size for processing (default: `512`)
- `--russian-only` - Only translate to Russian
- `--english-only` - Only translate to English

### `generate-grammar` - Generate Grammar Rule Descriptions

Generates markdown descriptions for Japanese grammar rules using
an LLM (OpenAI-compatible API).
Russian and English descriptions are generated in **separate API calls**
to ensure native-quality output for each language.

```bash
# Generate description for a single rule
utils generate-grammar --rule-id 01KJ9AVWBGC2BT0DMFPDYYFEWB

# Dry run: see what rules would be processed
utils generate-grammar --all --dry-run

# Regenerate all N5 rules
utils generate-grammar --all --level N5

# Regenerate all rules
utils generate-grammar --all

# Custom API endpoint
utils generate-grammar --all \
  --api-base http://localhost:8000/v1 \
  --api-key your-api-key

# Custom grammar.json path
utils generate-grammar --all --grammar-path path/to/grammar.json
```

**Options:**

- `rule_id` - Rule ID to generate (omit with `--all` for batch mode)
- `--all` - Generate descriptions for all rules
- `--level <LEVEL>` - Filter by JLPT level (N5, N4, N3) — use with `--all`
- `--api-base <API_BASE>` - OpenAI API base URL
  (default: `http://10.2.11.6:8001/v1`)
- `--api-key <API_KEY>` - OpenAI API key (default: `none`)
- `-w, --workers <WORKERS>` - Number of concurrent workers
  (default: `1`, sequential)
- `--dry-run` - Show what would be done without making changes
- `--grammar-path <PATH>` - Custom path to grammar.json

**Notes:**

- The command saves after **each rule** (crash-safe)
- 1-second delay between rules (rate limiting)
- RU and EN descriptions are generated in separate LLM calls
- Existing `rule_id`, `level`, and `format_map` are preserved

### `validate-dictionary` - Validate Vocabulary Translations

Validates vocabulary dictionary translations by sending each word +
translation to an LLM for Y/N correctness check.

```bash
# Dry run — see how many words would be checked
utils validate-dictionary --api-key YOUR_KEY --dry-run

# Validate all vocabulary with 8 workers
utils validate-dictionary --api-key YOUR_KEY

# Resume interrupted validation
utils validate-dictionary --api-key YOUR_KEY --resume

# Validate with custom model and limit
utils validate-dictionary --api-key YOUR_KEY \
  --model qwen/qwen3.5-flash --limit 100

# Custom output path
utils validate-dictionary --api-key YOUR_KEY -o results/invalid.jsonl
```

**Options:**

- `--api-key <API_KEY>` - OpenRouter API key (required, or set
  `OPENROUTER_API_KEY` env var)
- `--api-base <API_BASE>` - API base URL
  (default: `https://openrouter.ai/api/v1`)
- `--model <MODEL>` - LLM model (default: `qwen/qwen3.5-flash`)
- `-w, --workers <WORKERS>` - Concurrent requests (default: `8`)
- `-o, --output <OUTPUT>` - Output JSONL progress file path
- `--dry-run` - Estimate without API calls
- `--resume` - Resume from previous run
- `--limit <LIMIT>` - Max words to validate

**Output:**

- `.jsonl` file — append-only progress (crash-safe)
- `.json` file — final summary with list of invalid words for re-generation

## Getting Help

To see all available commands:

```bash
utils --help
```

To see help for a specific command:

```bash
utils tokenize --help
utils ndlocr --help
utils tokenize-well-known --help
utils find-missing --help
```

## Dictionary Requirements

The tokenizer commands require the SudachiDict dictionary (lindera dictionary
format, built by `scripts/build_sudachidict.py`; readable by the lindera 6.x
runtime). The CLI resolves it in the
following locations:

1. the directory passed via `--dictionary-dir` (if it exists)
2. `cdn/dictionaries/sudachidict-20260723/` (versioned)
3. `target/cdn-cache/` (downloaded CDN copy)

The dictionary files include:

- `char_def.bin` (compressed)
- `matrix.mtx` (compressed)
- `dict.trie` (compressed)
- `dict.valsidx` (compressed)
- `dict.vals` (compressed)
- `unk.bin` (compressed)
- `dict.wordsidx` (compressed)
- `dict.words` (compressed)
- `metadata.json`

## Examples

### Extract vocabulary from Japanese text

```bash
tokenizer tokenize "私は日本語を勉強しています"
# Output: 勉強 日本語 私
```

### OCR a Japanese document

```bash
tokenizer ndlocr -i document.png > output.txt
```

### Update well-known sets with tokenized vocabulary

```bash
tokenizer tokenize-well-known ../origa_ui/public/well_known/
```

### Find and translate missing vocabulary

```bash
tokenizer find-missing --generate --api-key sk-xxx
```
