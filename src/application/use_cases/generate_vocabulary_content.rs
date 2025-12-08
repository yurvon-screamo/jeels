use crate::application::{EmbeddingService, LlmService};
use crate::domain::error::JeersError;
use crate::domain::value_objects::{ExamplePhrase, JapaneseLevel, PartOfSpeech};
use serde::Deserialize;

const MAX_RETRIES: usize = 4;

#[derive(Clone)]
pub struct GenerateVocabularyContentUseCase<'a, L: LlmService, E: EmbeddingService> {
    llm_service: &'a L,
    embedding_service: &'a E,
}

#[derive(Debug, Deserialize)]
struct LlmResponse {
    russian_translation: String,
    english_translation: String,
    part_of_speech: String,
    russian_examples: Vec<ExamplePhrase>,
    english_examples: Vec<ExamplePhrase>,
}

pub struct VocabularyContent {
    pub russian_translation: String,
    pub english_translation: String,
    pub part_of_speech: PartOfSpeech,
    pub russian_examples: Vec<ExamplePhrase>,
    pub english_examples: Vec<ExamplePhrase>,
    pub embedding: Vec<f32>,
}

impl<'a, L: LlmService, E: EmbeddingService> GenerateVocabularyContentUseCase<'a, L, E> {
    pub fn new(llm_service: &'a L, embedding_service: &'a E) -> Self {
        Self {
            llm_service,
            embedding_service,
        }
    }

    pub async fn generate_content(
        &self,
        word: &str,
        japanese_level: &JapaneseLevel,
    ) -> Result<VocabularyContent, JeersError> {
        let content = self.generate_with_llm(word, japanese_level).await?;
        let embedding = self
            .embedding_service
            .generate_embedding("Represent this Japanese word for find same words", word)
            .await?;

        Ok(VocabularyContent {
            russian_translation: content.russian_translation,
            english_translation: content.english_translation,
            part_of_speech: content.part_of_speech,
            russian_examples: content.russian_examples,
            english_examples: content.english_examples,
            embedding: embedding.0,
        })
    }

    async fn generate_with_llm(
        &self,
        word: &str,
        japanese_level: &JapaneseLevel,
    ) -> Result<VocabularyContentData, JeersError> {
        let prompt = build_prompt(word, japanese_level);
        let mut last_error = None;

        for attempt in 1..=MAX_RETRIES {
            match self.llm_service.generate_text(&prompt).await {
                Ok(response) => match self.process_llm_response(&response, attempt) {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        last_error = Some(e);
                        if attempt < MAX_RETRIES {
                            continue;
                        }
                    }
                },
                Err(e) => {
                    last_error = Some(create_generation_error(attempt, &e));
                    if attempt < MAX_RETRIES {
                        continue;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| JeersError::LlmError {
            reason: "Failed to generate content after all retries".to_string(),
        }))
    }

    fn process_llm_response(
        &self,
        response: &str,
        attempt: usize,
    ) -> Result<VocabularyContentData, JeersError> {
        let response_cleaned = clean_json_response(response);
        let llm_response = parse_json_response(&response_cleaned, attempt)?;
        let part_of_speech = parse_part_of_speech(&llm_response.part_of_speech, attempt)?;
        Ok(VocabularyContentData {
            russian_translation: clean_response_text(&llm_response.russian_translation),
            english_translation: clean_response_text(&llm_response.english_translation),
            part_of_speech,
            russian_examples: llm_response.russian_examples,
            english_examples: llm_response.english_examples,
        })
    }
}

struct VocabularyContentData {
    russian_translation: String,
    english_translation: String,
    part_of_speech: PartOfSpeech,
    russian_examples: Vec<ExamplePhrase>,
    english_examples: Vec<ExamplePhrase>,
}

fn build_prompt(word: &str, japanese_level: &JapaneseLevel) -> String {
    format!(
        r#"Ты — помощник для изучения языков.
Твоя задача: Создай переводы и примеры использования слова: '{word}' для студентов уровня {japanese_level}.

Требования к переводам:
1. Русский перевод: ответь 1 предложением на русском языке.
2. Английский перевод: ответь 1 предложением на английском языке.
3. Не повторяй слово в ответе, потому что твой ответ будет использоваться как обратная сторона карточки и нужно иметь возможность их переворачивать и прогонять в обратном направлении.
4. Не указывай в ответе чтение или транскрипцию, студент умеет читать.
5. Выдай просто ответ без вводных или объяснений зачем и для кого это.
6. Если слово состоит из 1 кандзи, то объясни его значение как слово, а не как кандзи.

Требования к примерам:
1. Создай 2 простых примера использования слова для каждого языка.
2. Максимально простая грамматика.
3. Короткие простые предложения.
4. Ориентируйся на уровень {japanese_level}.

Требования к части речи:
Определи часть речи слова и верни одно из: Noun, Verb, Adjective, Adverb, Pronoun, Preposition, Conjunction, Interjection, Particle, Other.

Ответ должен быть СТРОГО валидным JSON, без markdown разметки (без ```json):
{{
  "russian_translation": "перевод на русском",
  "english_translation": "translation in English",
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
}}

Например (для уровня N5):
{{
  "russian_translation": "изучать",
  "english_translation": "to study",
  "part_of_speech": "Verb",
  "russian_examples": [
    {{
      "text": "私は日本語を勉強しています。",
      "translation": "Я изучаю японский язык."
    }},
    {{
      "text": "毎日勉強します。",
      "translation": "Я учусь каждый день."
    }}
  ],
  "english_examples": [
    {{
      "text": "私は日本語を勉強しています。",
      "translation": "I am studying Japanese."
    }},
    {{
      "text": "毎日勉強します。",
      "translation": "I study every day."
    }}
  ]
}}"#,
        word = word
    )
}

fn clean_response_text(response: &str) -> String {
    response.trim_matches(['\n', '\r', '.', ' ']).to_string()
}

fn parse_json_response(response: &str, attempt: usize) -> Result<LlmResponse, JeersError> {
    serde_json::from_str::<LlmResponse>(response).map_err(|e| JeersError::LlmError {
        reason: format!(
            "Failed to parse JSON (attempt {}/{}): {}. Response: {}",
            attempt, MAX_RETRIES, e, response
        ),
    })
}

fn parse_part_of_speech(pos: &str, attempt: usize) -> Result<PartOfSpeech, JeersError> {
    match pos.trim() {
        "Noun" => Ok(PartOfSpeech::Noun),
        "Verb" => Ok(PartOfSpeech::Verb),
        "Adjective" => Ok(PartOfSpeech::Adjective),
        "Adverb" => Ok(PartOfSpeech::Adverb),
        "Pronoun" => Ok(PartOfSpeech::Pronoun),
        "Preposition" => Ok(PartOfSpeech::Preposition),
        "Conjunction" => Ok(PartOfSpeech::Conjunction),
        "Interjection" => Ok(PartOfSpeech::Interjection),
        "Particle" => Ok(PartOfSpeech::Particle),
        "Other" => Ok(PartOfSpeech::Other),
        _ => Err(JeersError::LlmError {
            reason: format!(
                "Invalid part of speech (attempt {}/{}): '{}'. Expected one of: Noun, Verb, Adjective, Adverb, Pronoun, Preposition, Conjunction, Interjection, Particle, Other",
                attempt, MAX_RETRIES, pos
            ),
        }),
    }
}

fn create_generation_error(attempt: usize, error: &dyn std::fmt::Display) -> JeersError {
    JeersError::LlmError {
        reason: format!(
            "Failed to generate content (attempt {}/{}): {}",
            attempt, MAX_RETRIES, error
        ),
    }
}

fn clean_json_response(response: &str) -> String {
    response.trim().replace("```json", "").replace("```", "")
}
