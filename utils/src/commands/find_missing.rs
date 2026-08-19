use crate::api::VocabularyEntry;
use crate::api::translate_word_with_model;
use crate::dictionary::load_dictionary;
use crate::utils::{collect_json_files, get_base_path};
use origa::domain::OrigaError;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

/// Loads vocabulary dictionary from chunk files
fn load_vocabulary_dictionary(base_path: &Path) -> Result<HashSet<String>, OrigaError> {
    let mut words = HashSet::new();
    let vocab_path = base_path.join("cdn").join("dictionary");

    if !vocab_path.exists() {
        tracing::warn!("Vocabulary directory not found: {}", vocab_path.display());
        return Ok(words);
    }

    let files = collect_json_files(&vocab_path)?;

    for file in files {
        let content = fs::read_to_string(&file).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to read {}: {}", file.display(), e),
        })?;

        let json: Value =
            serde_json::from_str(&content).map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to parse {}: {}", file.display(), e),
            })?;

        if let Some(obj) = json.as_object() {
            for key in obj.keys() {
                words.insert(key.clone());
            }
        }
    }

    tracing::info!("Loaded {} words from vocabulary dictionary", words.len());
    Ok(words)
}

/// Loads well-known sets from JSON files
fn load_well_known_sets(base_path: &Path) -> Result<HashMap<String, HashSet<String>>, OrigaError> {
    let mut sets = HashMap::new();
    let well_known_path = base_path.join("cdn").join("well_known_set");

    let files = collect_json_files(&well_known_path)?;

    for file in files {
        let content = fs::read_to_string(&file).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to read {}: {}", file.display(), e),
        })?;

        let json: Value =
            serde_json::from_str(&content).map_err(|e| OrigaError::TokenizerError {
                reason: format!("Failed to parse {}: {}", file.display(), e),
            })?;

        // Use filename stem as set name (e.g., "duolingo_en_n3_1")
        let name = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        if let Some(words) = json.get("words").and_then(|w| w.as_array()) {
            let word_set: HashSet<String> = words
                .iter()
                .filter_map(|w| w.as_str().map(String::from))
                .collect();
            sets.insert(name, word_set);
        }
    }

    Ok(sets)
}

/// Finds words that are missing from the dictionary
fn find_missing_words(
    sets: &HashMap<String, HashSet<String>>,
    dictionary_words: &HashSet<String>,
) -> HashMap<String, Vec<String>> {
    let mut missing = HashMap::new();

    for (set_name, words) in sets {
        let missing_in_set: Vec<String> = words
            .iter()
            .filter(|w| !dictionary_words.contains(*w))
            .cloned()
            .collect();

        if !missing_in_set.is_empty() {
            missing.insert(set_name.clone(), missing_in_set);
        }
    }

    missing
}

/// Generates a markdown report of missing words
fn generate_report(
    missing: &HashMap<String, Vec<String>>,
    output_path: &Path,
) -> Result<(), OrigaError> {
    let mut report = String::new();
    report.push_str("# Missing Vocabulary Report\n\n");

    let mut total_missing = 0;
    for (set_name, words) in missing {
        total_missing += words.len();
        report.push_str(&format!("## {}\n\n", set_name));
        for word in words {
            report.push_str(&format!("- {}\n", word));
        }
        report.push('\n');
    }

    report.insert_str(0, &format!("Total missing words: {}\n\n", total_missing));

    fs::write(output_path, report).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to write report: {}", e),
    })?;

    tracing::info!("Report saved to: {}", output_path.display());
    Ok(())
}

/// Collects all unique words from missing sets
fn collect_unique_words(missing: &HashMap<String, Vec<String>>) -> Vec<String> {
    let all_words: HashSet<String> = missing.values().flatten().cloned().collect();
    all_words.into_iter().collect()
}

/// Translates a chunk of words concurrently
async fn translate_words_chunk(
    chunk: &[String],
    api_base: &str,
    api_key: &str,
    model: &str,
    to_russian: bool,
    to_english: bool,
    results: Arc<Mutex<HashMap<String, VocabularyEntry>>>,
) {
    let mut handles = vec![];

    for word in chunk {
        let word = word.clone();
        let results = Arc::clone(&results);
        let api_base = api_base.to_string();
        let api_key = api_key.to_string();
        let model = model.to_string();

        let handle = tokio::spawn(async move {
            let entry = translate_word_with_model(
                &word, &api_base, &api_key, &model, to_russian, to_english,
            )
            .await;
            match entry {
                Ok(entry) => {
                    if entry.russian_translation.is_none() && entry.english_translation.is_none() {
                        tracing::warn!("Translation returned empty for word: {}", word);
                    } else {
                        let mut results = results.lock().await;
                        results.insert(word, entry);
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to translate word '{}': {}", word, e);
                },
            }
            sleep(Duration::from_millis(100)).await;
        });

        handles.push(handle);
    }

    futures::future::join_all(handles).await;
}

/// Generates translations for missing vocabulary words
async fn generate_missing_vocabulary(
    missing: &HashMap<String, Vec<String>>,
    api_base: &str,
    api_key: &str,
    model: &str,
    workers: usize,
    russian_only: bool,
    english_only: bool,
) -> Result<HashMap<String, VocabularyEntry>, OrigaError> {
    let to_russian = !english_only;
    let to_english = !russian_only;

    let all_words = collect_unique_words(missing);
    tracing::info!("Translating {} unique words...", all_words.len());

    let results = Arc::new(Mutex::new(HashMap::new()));

    for chunk in all_words.chunks(workers) {
        translate_words_chunk(
            chunk,
            api_base,
            api_key,
            model,
            to_russian,
            to_english,
            Arc::clone(&results),
        )
        .await;
    }

    let results = Arc::try_unwrap(results).unwrap().into_inner();
    Ok(results)
}

/// Saves vocabulary entries to the dictionary file
fn save_dictionary(
    entries: &HashMap<String, VocabularyEntry>,
    base_path: &Path,
) -> Result<(), OrigaError> {
    let dict_path = base_path
        .join("cdn")
        .join("dictionaries")
        .join("missing_vocabulary.json");

    let mut existing: HashMap<String, VocabularyEntry> = if dict_path.exists() {
        let content = fs::read_to_string(&dict_path).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to read existing dictionary: {}", e),
        })?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        HashMap::new()
    };

    for (word, entry) in entries {
        existing.insert(word.clone(), entry.clone());
    }

    let json = serde_json::to_string_pretty(&existing).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to serialize dictionary: {}", e),
    })?;

    // Create dictionaries directory if it doesn't exist
    if let Some(parent) = dict_path.parent() {
        fs::create_dir_all(parent).map_err(|e| OrigaError::TokenizerError {
            reason: format!("Failed to create dictionaries directory: {}", e),
        })?;
    }

    fs::write(&dict_path, json).map_err(|e| OrigaError::TokenizerError {
        reason: format!("Failed to write dictionary: {}", e),
    })?;

    tracing::info!("Dictionary saved to: {}", dict_path.display());
    Ok(())
}

/// Main function for find_missing command
#[allow(clippy::too_many_arguments)]
pub async fn run_find_missing(
    output: Option<std::path::PathBuf>,
    generate: bool,
    api_base: String,
    api_key: String,
    model: String,
    workers: usize,
    _chunk_size: usize,
    russian_only: bool,
    english_only: bool,
) -> Result<(), OrigaError> {
    let base_path = get_base_path();

    load_dictionary()?;

    let sets = load_well_known_sets(&base_path)?;
    tracing::info!("Loaded {} well-known sets", sets.len());

    let dictionary_words = load_vocabulary_dictionary(&base_path)?;
    let missing = find_missing_words(&sets, &dictionary_words);

    if missing.is_empty() {
        tracing::info!("No missing words found");
        return Ok(());
    }

    let output_path = output.unwrap_or_else(|| base_path.join("missing_vocabulary.md"));
    generate_report(&missing, &output_path)?;

    if generate {
        let translations = generate_missing_vocabulary(
            &missing,
            &api_base,
            &api_key,
            &model,
            workers,
            russian_only,
            english_only,
        )
        .await?;

        save_dictionary(&translations, &base_path)?;
    }

    Ok(())
}
