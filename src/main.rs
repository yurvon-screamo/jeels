use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use keikaku::application::LlmService;
use keikaku::domain::error::JeersError;
use keikaku::infrastructure::{CandleLlm, GeminiLlm, LlmServiceInvoker, OpenAiLlm};
use keikaku::settings::{ApplicationEnvironment, LlmSettings};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::Semaphore;
use toml;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExamplePhraseStoredType {
    text: String,
    translation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VocabularyEntryStoredType {
    level: String,
    russian_translation: String,
    english_translation: String,
    russian_examples: Vec<ExamplePhraseStoredType>,
    english_examples: Vec<ExamplePhraseStoredType>,
    #[serde(default)]
    part_of_speech: Option<String>,
    embedding: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct ValidationResponse {
    is_correct: bool,
    issues: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LlmResponse {
    translation: String,
    examples: Vec<ExamplePhraseStoredType>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ApplicationEnvironment::load().await?;

    let vocabulary_files = vec![
        "words/vocabulary_n5.json",
        "words/vocabulary_n4.json",
        "words/vocabulary_n3.json",
        "words/vocabulary_n2.json",
        "words/vocabulary_n1.json",
    ];

    let multi = MultiProgress::new();

    for file_path in vocabulary_files {
        println!("Processing {}...", file_path);
        process_vocabulary_file(file_path, &multi).await?;
    }

    Ok(())
}

async fn create_llm_service() -> Result<LlmServiceInvoker, JeersError> {
    // Read settings from config.toml directly
    let config_path = "config.toml";
    let contents = std::fs::read_to_string(config_path).map_err(|e| JeersError::SettingsError {
        reason: format!("Failed to read config.toml: {}", e),
    })?;

    #[derive(serde::Deserialize)]
    struct Config {
        llm: LlmSettings,
    }

    let config: Config = toml::from_str(&contents).map_err(|e| JeersError::SettingsError {
        reason: format!("Failed to parse config.toml: {}", e),
    })?;

    match &config.llm {
        LlmSettings::Gemini { temperature, model } => Ok(LlmServiceInvoker::Gemini(
            GeminiLlm::new(*temperature, model.clone()).map_err(|e| JeersError::SettingsError {
                reason: e.to_string(),
            })?,
        )),
        LlmSettings::OpenAi {
            temperature,
            model,
            base_url,
            env_var_name,
        } => Ok(LlmServiceInvoker::OpenAi(
            OpenAiLlm::new(
                *temperature,
                model.clone(),
                base_url.clone(),
                env_var_name.clone(),
            )
            .map_err(|e| JeersError::SettingsError {
                reason: e.to_string(),
            })?,
        )),
        LlmSettings::Candle {
            max_sample_len,
            temperature,
            seed,
            model_repo,
            model_filename,
            model_revision,
            tokenizer_repo,
            tokenizer_filename,
        } => Ok(LlmServiceInvoker::Candle(
            CandleLlm::new(
                *max_sample_len,
                *temperature,
                *seed,
                model_repo.clone(),
                model_filename.clone(),
                model_revision.clone(),
                tokenizer_repo.clone(),
                tokenizer_filename.clone(),
            )
            .map_err(|e| JeersError::SettingsError {
                reason: e.to_string(),
            })?,
        )),
    }
}

async fn process_vocabulary_file(
    file_path: &str,
    multi: &MultiProgress,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let vocabulary: HashMap<String, VocabularyEntryStoredType> = serde_json::from_str(&content)?;

    let total_words = vocabulary.len();
    let semaphore = Arc::new(Semaphore::new(20));
    let fixed_words = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    let pb = multi.add(ProgressBar::new(total_words as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(format!("Processing {}", file_path));

    let mut handles = Vec::new();

    let llm_service = create_llm_service().await?;
    let llm_service = Arc::new(llm_service);

    for (word, entry) in vocabulary.iter() {
        let word = word.clone();
        let entry = entry.clone();
        let llm_service = llm_service.clone();
        let semaphore = semaphore.clone();
        let fixed_words = fixed_words.clone();
        let pb = pb.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            let mut current_entry = entry.clone();
            let mut was_fixed = false;

            // Check Russian translation and examples
            match check_and_fix_entry(
                &word,
                &current_entry,
                &keikaku::domain::value_objects::NativeLanguage::Russian,
                &llm_service,
            )
            .await
            {
                Ok(Some(fixed_entry)) => {
                    current_entry = fixed_entry;
                    was_fixed = true;
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("    Error checking Russian for '{}': {}", word, e);
                    // Fallback: generate new content
                    if let Ok(Some(fixed_entry)) = generate_fallback_content(
                        &word,
                        &current_entry,
                        &keikaku::domain::value_objects::NativeLanguage::Russian,
                        &llm_service,
                    )
                    .await
                    {
                        current_entry = fixed_entry;
                        was_fixed = true;
                    }
                }
            }

            // Check English translation and examples
            match check_and_fix_entry(
                &word,
                &current_entry,
                &keikaku::domain::value_objects::NativeLanguage::English,
                &llm_service,
            )
            .await
            {
                Ok(Some(fixed_entry)) => {
                    current_entry = fixed_entry;
                    was_fixed = true;
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("    Error checking English for '{}': {}", word, e);
                    // Fallback: generate new content
                    if let Ok(Some(fixed_entry)) = generate_fallback_content(
                        &word,
                        &current_entry,
                        &keikaku::domain::value_objects::NativeLanguage::English,
                        &llm_service,
                    )
                    .await
                    {
                        current_entry = fixed_entry;
                        was_fixed = true;
                    }
                }
            }

            if was_fixed {
                fixed_words.lock().await.push(word.clone());
            }

            pb.inc(1);
            (word, current_entry)
        });

        handles.push(handle);
    }

    let mut updated_vocabulary = HashMap::new();
    for handle in handles {
        let (word, entry) = handle.await?;
        updated_vocabulary.insert(word, entry);
    }

    pb.finish_with_message("Completed");

    let fixed_list = fixed_words.lock().await;
    let fixed_count = fixed_list.len();

    println!(
        "  Completed: processed {}, fixed {}",
        total_words, fixed_count
    );

    if !fixed_list.is_empty() {
        println!("  Fixed words:");
        for word in fixed_list.iter() {
            println!("    - {}", word);
        }
    }

    // Save updated vocabulary
    let updated_content = serde_json::to_string_pretty(&updated_vocabulary)?;
    fs::write(file_path, updated_content)?;
    println!("  Saved updates to {}", file_path);

    Ok(())
}

async fn generate_fallback_content(
    word: &str,
    entry: &VocabularyEntryStoredType,
    native_language: &keikaku::domain::value_objects::NativeLanguage,
    llm_service: &Arc<keikaku::infrastructure::LlmServiceInvoker>,
) -> Result<Option<VocabularyEntryStoredType>, String> {
    let level = parse_jlpt_level(&entry.level);

    // Generate new content directly without validation (no issues available)
    let generation_prompt = build_generation_prompt(word, &level, native_language, None);
    let generation_response = match llm_service.as_ref().generate_text(&generation_prompt).await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!(
                "    Fallback generation failed for '{}' ({:?}): {}",
                word, native_language, e
            );
            return Ok(None);
        }
    };

    let new_content = match parse_generation_response(&generation_response) {
        Ok(content) => content,
        Err(e) => {
            eprintln!(
                "    Fallback parsing failed for '{}' ({:?}): {}. Response: {}",
                word, native_language, e, generation_response
            );
            return Ok(None);
        }
    };

    // Create updated entry
    let mut updated_entry = entry.clone();
    match native_language {
        keikaku::domain::value_objects::NativeLanguage::Russian => {
            updated_entry.russian_translation = new_content.translation;
            updated_entry.russian_examples = new_content.examples;
        }
        keikaku::domain::value_objects::NativeLanguage::English => {
            updated_entry.english_translation = new_content.translation;
            updated_entry.english_examples = new_content.examples;
        }
    }

    Ok(Some(updated_entry))
}

async fn check_and_fix_entry(
    word: &str,
    entry: &VocabularyEntryStoredType,
    native_language: &keikaku::domain::value_objects::NativeLanguage,
    llm_service: &Arc<keikaku::infrastructure::LlmServiceInvoker>,
) -> Result<Option<VocabularyEntryStoredType>, String> {
    let level = parse_jlpt_level(&entry.level);
    let translation = match native_language {
        keikaku::domain::value_objects::NativeLanguage::Russian => &entry.russian_translation,
        keikaku::domain::value_objects::NativeLanguage::English => &entry.english_translation,
    };
    let examples = match native_language {
        keikaku::domain::value_objects::NativeLanguage::Russian => &entry.russian_examples,
        keikaku::domain::value_objects::NativeLanguage::English => &entry.english_examples,
    };

    // Validate current entry
    let validation_prompt =
        build_validation_prompt(word, translation, examples, &level, native_language);
    let validation_response = match llm_service.as_ref().generate_text(&validation_prompt).await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!(
                "    Error validating '{}' ({:?}): {}, falling back to generation",
                word, native_language, e
            );
            // Fallback to generation
            return generate_fallback_content(word, entry, native_language, llm_service).await;
        }
    };

    let validation_result = match parse_validation_response(&validation_response) {
        Ok(result) => result,
        Err(e) => {
            eprintln!(
                "    Error parsing validation response for '{}' ({:?}): {}, falling back to generation. Response: {}",
                word, native_language, e, validation_response
            );
            // Fallback to generation
            return generate_fallback_content(word, entry, native_language, llm_service).await;
        }
    };

    if validation_result.is_correct {
        return Ok(None);
    }

    println!(
        "    Found issues with '{}' ({:?}): {:?}",
        word, native_language, validation_result.issues
    );

    // Generate new content with validation issues to avoid repeating mistakes
    let generation_prompt = build_generation_prompt(
        word,
        &level,
        native_language,
        Some(&validation_result.issues),
    );
    let generation_response = match llm_service.as_ref().generate_text(&generation_prompt).await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!(
                "    Error generating new content for '{}' ({:?}): {}",
                word, native_language, e
            );
            return Ok(None);
        }
    };

    let new_content = match parse_generation_response(&generation_response) {
        Ok(content) => content,
        Err(e) => {
            eprintln!(
                "    Error parsing generation response for '{}' ({:?}): {}. Response: {}",
                word, native_language, e, generation_response
            );
            return Ok(None);
        }
    };

    // Create updated entry
    let mut updated_entry = entry.clone();
    match native_language {
        keikaku::domain::value_objects::NativeLanguage::Russian => {
            updated_entry.russian_translation = new_content.translation;
            updated_entry.russian_examples = new_content.examples;
        }
        keikaku::domain::value_objects::NativeLanguage::English => {
            updated_entry.english_translation = new_content.translation;
            updated_entry.english_examples = new_content.examples;
        }
    }

    Ok(Some(updated_entry))
}

fn build_validation_prompt(
    word: &str,
    translation: &str,
    examples: &[ExamplePhraseStoredType],
    level: &keikaku::domain::value_objects::JapaneseLevel,
    native_language: &keikaku::domain::value_objects::NativeLanguage,
) -> String {
    let examples_json = serde_json::to_string(examples).unwrap_or_default();

    format!(
        r#"Ты — эксперт по японскому языку.
Проверь правильность перевода и примеров для слова '{word}' уровня {level} для {native_language} говорящего студента.

Текущий перевод: "{translation}"

Текущие примеры:
{examples_json}

Требования к переводу:
1. Перевод должен быть точным и соответствовать уровню {level}.
2. Перевод должен быть одним предложением без повторения самого слова.
3. Не должно быть транскрипции или чтения.

Требования к примерам:
1. Должно быть 2 примера использования слова.
2. Максимально простая грамматика для уровня {level}.
3. Короткие простые предложения.
4. Переводы примеров должны быть точными.

Ответь СТРОГО валидным JSON без markdown разметки:
{{
  "is_correct": true/false,
  "issues": ["список проблем, если есть"]
}}

Если всё правильно, укажи "is_correct": true и пустой массив "issues"."#
    )
}

fn build_generation_prompt(
    word: &str,
    level: &keikaku::domain::value_objects::JapaneseLevel,
    native_language: &keikaku::domain::value_objects::NativeLanguage,
    validation_issues: Option<&[String]>,
) -> String {
    let issues_section = if let Some(issues) = validation_issues {
        if !issues.is_empty() {
            format!(
                "\n\nВАЖНО: Рекомендации по переводу и примерам:\n{}.\n",
                issues
                    .iter()
                    .enumerate()
                    .map(|(i, issue)| format!("{}. {}", i + 1, issue))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    format!(
        r#"Ты — помощник для изучения языков.
Твоя задача: Создай перевод и примеры использования слова: '{word}' для {native_language} говорящего студента уровня {level}.{issues_section}

Требования к переводу:
1. Ответь 1 предложением.
2. Не повторяй слово в ответе, потому что твой ответ будет использоваться как обратная сторона карточки и нужно иметь возможность их переворачивать и прогонять в обратном направлении.
3. Не указывай в ответе чтение или транскрипцию, студент умеет читать.
4. Выдай просто ответ без вводных или объяснений зачем и для кого это.
5. Если слово состоит из 1 кандзи, то объясни его значение как слово, а не как кандзи.

Требования к примерам:
1. Создай 2 простых примера использования слова.
2. Максимально простая грамматика.
3. Короткие простые предложения.
4. Ориентируйся на уровень {level}.

Ответ должен быть СТРОГО валидным JSON, без markdown разметки (без ```json):
{{
  "translation": "перевод слова",
  "examples": [
    {{
      "text": "предложение на японском",
      "translation": "перевод предложения"
    }}
  ]
}}

Например (для уровня N5, русскоязычного студента):
{{
  "translation": "изучать",
  "examples": [
    {{
      "text": "私は日本語を勉強しています。",
      "translation": "Я изучаю японский язык."
    }},
    {{
      "text": "毎日勉強します。",
      "translation": "Я учусь каждый день."
    }}
  ]
}}"#
    )
}

fn parse_validation_response(response: &str) -> Result<ValidationResponse, String> {
    let cleaned = response.trim().replace("```json", "").replace("```", "");
    let parsed: ValidationResponse = serde_json::from_str(&cleaned)
        .map_err(|e| format!("Failed to parse validation response: {}", e))?;
    Ok(parsed)
}

fn parse_generation_response(response: &str) -> Result<LlmResponse, String> {
    let cleaned = response.trim().replace("```json", "").replace("```", "");
    let parsed: LlmResponse = serde_json::from_str(&cleaned)
        .map_err(|e| format!("Failed to parse generation response: {}", e))?;
    Ok(parsed)
}

fn parse_jlpt_level(s: &str) -> keikaku::domain::value_objects::JapaneseLevel {
    match s {
        "N5" => keikaku::domain::value_objects::JapaneseLevel::N5,
        "N4" => keikaku::domain::value_objects::JapaneseLevel::N4,
        "N3" => keikaku::domain::value_objects::JapaneseLevel::N3,
        "N2" => keikaku::domain::value_objects::JapaneseLevel::N2,
        "N1" => keikaku::domain::value_objects::JapaneseLevel::N1,
        _ => keikaku::domain::value_objects::JapaneseLevel::N1,
    }
}
