use indicatif::{ProgressBar, ProgressStyle};
use keikaku::application::use_cases::{GenerateVocabularyContentUseCase, VocabularyContent};
use keikaku::domain::value_objects::{JapaneseLevel, PartOfSpeech};
use keikaku::settings::ApplicationEnvironment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Serialize, Deserialize)]
struct VocabularyEntry {
    level: String,
    russian_translation: String,
    english_translation: String,
    russian_examples: Vec<ExamplePhraseJson>,
    english_examples: Vec<ExamplePhraseJson>,
    part_of_speech: String,
    embedding: Vec<f32>,
}

#[derive(Serialize, Deserialize)]
struct ExamplePhraseJson {
    text: String,
    translation: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ApplicationEnvironment::load().await?;
    regenerate_vocabularies().await?;
    Ok(())
}

async fn regenerate_vocabularies() -> Result<(), Box<dyn std::error::Error>> {
    let env = ApplicationEnvironment::get();
    let llm_service = env.get_llm_service().await?;
    let embedding_service = env.get_embedding_service().await?;

    let semaphore = Arc::new(Semaphore::new(20));
    let use_case = Arc::new(GenerateVocabularyContentUseCase::new(
        llm_service,
        embedding_service,
    ));

    let levels = vec![
        ("words/vocabulary_n5.json", JapaneseLevel::N5),
        ("words/vocabulary_n4.json", JapaneseLevel::N4),
        ("words/vocabulary_n3.json", JapaneseLevel::N3),
        ("words/vocabulary_n2.json", JapaneseLevel::N2),
        ("words/vocabulary_n1.json", JapaneseLevel::N1),
    ];

    for (file_path, level) in levels {
        println!("Processing {}...", file_path);

        let content = fs::read_to_string(file_path)?;
        let vocabulary: HashMap<String, VocabularyEntry> = serde_json::from_str(&content)?;

        let total_words = vocabulary.len();
        let words: Vec<String> = vocabulary.keys().cloned().collect();

        let pb = ProgressBar::new(total_words as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(format!("Processing {}", file_path));

        let pb = Arc::new(pb);
        let mut tasks = Vec::new();

        for (idx, word) in words.iter().enumerate() {
            let word = word.clone();
            let use_case = Arc::clone(&use_case);
            let semaphore = Arc::clone(&semaphore);
            let pb = Arc::clone(&pb);
            let level = level.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let result = use_case.generate_content(&word, &level).await;
                println!("Generated content for {}: {}", word, result.is_ok());
                pb.inc(1);
                (idx, word, result)
            });

            tasks.push(task);
        }

        let mut results: Vec<(usize, String, Result<VocabularyContent, _>)> = Vec::new();
        for task in tasks {
            results.push(task.await?);
        }

        pb.finish_with_message(format!("Completed {}", file_path));

        results.sort_by_key(|(idx, _, _)| *idx);

        let mut new_vocabulary: HashMap<String, VocabularyEntry> = HashMap::new();

        for (_idx, word, result) in results {
            match result {
                Ok(content) => {
                    let entry = VocabularyEntry {
                        level: format!("N{}", level.as_number()),
                        russian_translation: content.russian_translation,
                        english_translation: content.english_translation,
                        russian_examples: content
                            .russian_examples
                            .into_iter()
                            .map(|e| ExamplePhraseJson {
                                text: e.text().clone(),
                                translation: e.translation().clone(),
                            })
                            .collect(),
                        english_examples: content
                            .english_examples
                            .into_iter()
                            .map(|e| ExamplePhraseJson {
                                text: e.text().clone(),
                                translation: e.translation().clone(),
                            })
                            .collect(),
                        part_of_speech: part_of_speech_to_string(&content.part_of_speech),
                        embedding: content.embedding,
                    };
                    new_vocabulary.insert(word, entry);
                }
                Err(e) => {
                    eprintln!("Error processing {}: {}", word, e);
                }
            }
        }

        let json_content = serde_json::to_string_pretty(&new_vocabulary)?;
        fs::write(file_path, json_content)?;
        println!(
            "Completed {}: {} words processed",
            file_path,
            new_vocabulary.len()
        );
    }

    println!("All vocabularies regenerated successfully!");
    Ok(())
}

fn part_of_speech_to_string(pos: &PartOfSpeech) -> String {
    match pos {
        PartOfSpeech::Noun => "Noun".to_string(),
        PartOfSpeech::Verb => "Verb".to_string(),
        PartOfSpeech::Adjective => "Adjective".to_string(),
        PartOfSpeech::Adverb => "Adverb".to_string(),
        PartOfSpeech::Pronoun => "Pronoun".to_string(),
        PartOfSpeech::Preposition => "Preposition".to_string(),
        PartOfSpeech::Conjunction => "Conjunction".to_string(),
        PartOfSpeech::Interjection => "Interjection".to_string(),
        PartOfSpeech::Particle => "Particle".to_string(),
        PartOfSpeech::Other => "Other".to_string(),
    }
}
