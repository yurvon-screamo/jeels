use crate::application::LlmService;
use crate::domain::error::JeersError;
use crate::domain::value_objects::{
    Answer, CardContent, ExamplePhrase, JapaneseLevel, NativeLanguage,
};
use crate::domain::vocabulary::VOCABULARY_DB;
use serde::Deserialize;

const MAX_RETRIES: usize = 3;

#[derive(Clone)]
pub struct GenerateCardContentUseCase<'a, L: LlmService> {
    llm_service: &'a L,
}

#[derive(Debug, Deserialize)]
struct LlmResponse {
    translation: String,
    examples: Vec<ExamplePhrase>,
}

impl<'a, L: LlmService> GenerateCardContentUseCase<'a, L> {
    pub fn new(llm_service: &'a L) -> Self {
        Self { llm_service }
    }

    pub async fn generate_content(
        &self,
        question_text: &str,
        native_language: &NativeLanguage,
        japanese_level: &JapaneseLevel,
    ) -> Result<CardContent, JeersError> {
        if let Some(result) = self.try_get_from_dictionary(question_text, native_language)? {
            return Ok(result);
        }

        self.generate_with_llm(question_text, native_language, japanese_level)
            .await
    }

    fn try_get_from_dictionary(
        &self,
        question_text: &str,
        native_language: &NativeLanguage,
    ) -> Result<Option<CardContent>, JeersError> {
        if let Some(translation) = VOCABULARY_DB.get_translation(question_text, native_language) {
            let answer = Answer::new(translation)?;
            if let Some(examples) = VOCABULARY_DB.get_examples(question_text, native_language)
                && !examples.is_empty()
            {
                return Ok(Some(CardContent::new(answer, examples)));
            }
        }

        Ok(None)
    }

    async fn generate_with_llm(
        &self,
        question_text: &str,
        native_language: &NativeLanguage,
        japanese_level: &JapaneseLevel,
    ) -> Result<CardContent, JeersError> {
        let prompt = build_prompt(question_text, native_language, japanese_level);
        let mut last_error = None;

        for attempt in 1..=MAX_RETRIES {
            match self.llm_service.generate_text(&prompt).await {
                Ok(response) => match self.process_llm_response(&response, attempt) {
                    Ok(result) => {
                        return Ok(result);
                    }
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
    ) -> Result<CardContent, JeersError> {
        let response_cleaned = clean_json_response(response);
        let llm_response = parse_json_response(&response_cleaned, attempt)?;
        let answer = validate_and_create_answer(&llm_response.translation, attempt)?;
        Ok(CardContent::new(answer, llm_response.examples))
    }
}

fn build_prompt(
    question_text: &str,
    native_language: &NativeLanguage,
    japanese_level: &JapaneseLevel,
) -> String {
    format!(
        r#"Ты — помощник для изучения языков.
Твоя задача: Создай перевод и примеры использования слова: '{word}' для {native_language} говорящего студента уровня {japanese_level}.

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
4. Ориентируйся на уровень {japanese_level}.

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
}}"#,
        word = question_text
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

fn validate_and_create_answer(translation: &str, attempt: usize) -> Result<Answer, JeersError> {
    let answer_text = clean_response_text(translation);
    Answer::new(answer_text).map_err(|e| JeersError::LlmError {
        reason: format!(
            "Invalid answer format (attempt {}/{}): {}",
            attempt, MAX_RETRIES, e
        ),
    })
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
