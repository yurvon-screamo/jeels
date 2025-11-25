use crate::application::{EmbeddingService, LlmService, UserRepository};
use crate::domain::Card;
use crate::domain::error::JeersError;
use crate::domain::value_objects::{Answer, Question};
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

    pub(crate) async fn generate_translation_and_embedding(
        &self,
        question_text: &str,
    ) -> Result<(Question, Answer), JeersError> {
        let question_text_string = question_text.to_string();
        let embedding = self
            .embedding_service
            .generate_embedding(&question_text_string)
            .await?;

        let question = Question::new(question_text_string, embedding)?;

        let answer_text = self
            .llm_service
            .generate_text(&format!(
                r#"Объясни значение этого слова для рускоговорящего студента: '{}'. 
Ответь 1 предложением. Не повторяй вопрос в ответе. 
Не указывай в ответе чтение или транскрипцию, студент умеет читать. 
Выдай просто ответ без вводных или объяснений зачем и для кого это.
Если слово состоит из 1 кандзи, то объясни его значение как слово, а не как кандзи."#,
                question_text
            ))
            .await?
            .trim_matches(&['\n', '\r', '.', ' '])
            .to_string();

        let answer = Answer::new(answer_text)?;

        Ok((question, answer))
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        question_text: String,
        answer_text: Option<String>,
    ) -> Result<Card, JeersError> {
        let (question, answer) = if let Some(answer_text) = answer_text {
            let embedding = self
                .embedding_service
                .generate_embedding(&question_text)
                .await?;

            (
                Question::new(question_text, embedding)?,
                Answer::new(answer_text)?,
            )
        } else {
            self.generate_translation_and_embedding(&question_text)
                .await?
        };

        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let card = user.create_card(question, answer)?;
        self.repository.save(&user).await?;

        Ok(card)
    }
}
