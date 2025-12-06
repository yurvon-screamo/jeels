use crate::application::{EmbeddingService, LlmService, UserRepository};
use crate::domain::VocabularyCard;
use crate::domain::error::JeersError;
use crate::domain::value_objects::{
    Answer, ExamplePhrase, JapaneseLevel, NativeLanguage, Question,
};
use ulid::Ulid;

#[derive(Clone)]
pub struct CreateCardUseCase<'a, R: UserRepository, E: EmbeddingService, L: LlmService> {
    repository: &'a R,
    embedding_service: &'a E,
    llm_service: &'a L,
}

impl<'a, R: UserRepository, E: EmbeddingService, L: LlmService> CreateCardUseCase<'a, R, E, L> {
    pub fn new(repository: &'a R, embedding_service: &'a E, llm_service: &'a L) -> Self {
        Self {
            repository,
            embedding_service,
            llm_service,
        }
    }

    pub(crate) async fn generate_translation(
        &self,
        question_text: &str,
        native_language: &NativeLanguage,
    ) -> Result<Answer, JeersError> {
        let answer_text = self
            .llm_service
            .generate_text(&format!(
                r#"Объясни значение этого слова для {native_language} говорящего студента: '{}'. Ответь 1 предложением.
Не повторяй слово в ответе, потому что твой ответ будет использоваться как обратная сторона карточки и нужно иметь возможность их переворачивать и прогонять в обратом направлении.
Не указывай в ответе чтение или транскрипцию, студент умеет читать.
Выдай просто ответ без вводных или объяснений зачем и для кого это.
Если слово состоит из 1 кандзи, то объясни его значение как слово, а не как кандзи."#,
                question_text
            ))
            .await?
            .trim_matches(['\n', '\r', '.', ' '])
            .to_string();

        let answer = Answer::new(answer_text)?;

        Ok(answer)
    }

    pub(crate) async fn generate_example_phrases(
        &self,
        question_text: &str,
        native_language: &NativeLanguage,
        japanese_level: &JapaneseLevel,
    ) -> Result<Vec<ExamplePhrase>, JeersError> {
        let prompt = format!(
            r#"Ты — помощник для изучения языков.
    Твоя задача: Создай 2 простых примера использования слова: '{word}'.
    Требования:
    1. Максимально простая грамматика.
    2. Короткие простыепредложения.
    3. Ответ должен быть СТРОГО валидным JSON, без markdown разметки (без ```json).
    4. Ориентируйся на уровень {japanese_level}.

    Используй следующую JSON Schema для ответа:
    {{
      "type": "array",
      "items": {{
        "type": "object",
        "properties": {{
          "text": {{
            "type": "string",
            "description": "Предложение на японском языке"
          }},
          "translation": {{
            "type": "string",
            "description": "Перевод на {native_language} язык"
          }}
        }},
        "required": ["text", "translation"]
      }}
    }}
    "#,
            word = question_text
        );

        let example_phrases = self
            .llm_service
            .generate_text(&prompt)
            .await?
            .trim()
            .replace("```json", "")
            .replace("```", "");

        let example_phrases = serde_json::from_str::<Vec<ExamplePhrase>>(&example_phrases)
            .map_err(|e| JeersError::LlmError {
                reason: format!("Failed to parse JSON: {}. Response: {}", e, example_phrases),
            })?;

        Ok(example_phrases)
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        question_text: String,
        answer_text: Option<String>,
        example_phrases: Option<Vec<ExamplePhrase>>,
    ) -> Result<VocabularyCard, JeersError> {
        let answer_text = if let Some(answer_srt) = &answer_text
            && answer_srt.trim().is_empty()
        {
            None
        } else {
            answer_text
        };

        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        if user
            .cards()
            .values()
            .any(|card| card.question().text() == question_text)
        {
            return Err(JeersError::DuplicateCard {
                question: question_text,
            });
        }

        let embedding = self
            .embedding_service
            .generate_embedding(&question_text)
            .await?;

        let answer = if let Some(answer_text) = answer_text {
            Answer::new(answer_text)?
        } else {
            self.generate_translation(question_text.as_str(), user.native_language())
                .await?
        };

        let example_phrases = if let Some(example_phrases) = example_phrases {
            example_phrases
        } else {
            self.generate_example_phrases(
                question_text.as_str(),
                user.native_language(),
                user.current_japanese_level(),
            )
            .await?
        };

        let question = Question::new(question_text, embedding)?;

        let card = user.create_card(question, answer, example_phrases)?;
        self.repository.save(&user).await?;

        Ok(card)
    }
}
