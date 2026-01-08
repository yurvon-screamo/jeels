use super::generate_card_content::GenerateCardContentUseCase;
use crate::application::UserRepository;
use crate::domain::error::KeikakuError;
use crate::domain::knowledge::{Card, StudyCard, VocabularyCard};
use crate::domain::tokenizer::Tokenizer;
use crate::domain::value_objects::Question;
use tracing::error;
use ulid::Ulid;

#[derive(Clone)]
pub struct CreateVocabularyCardUseCase<'a, R: UserRepository, L: crate::application::LlmService> {
    repository: &'a R,
    generate_content_use_case: GenerateCardContentUseCase<'a, L>,
}

impl<'a, R: UserRepository, L: crate::application::LlmService>
    CreateVocabularyCardUseCase<'a, R, L>
{
    pub fn new(repository: &'a R, llm_service: &'a L) -> Self {
        Self {
            repository,
            generate_content_use_case: GenerateCardContentUseCase::new(llm_service),
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        question_text: String,
    ) -> Result<Vec<StudyCard>, KeikakuError> {
        let mut user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(KeikakuError::UserNotFound { user_id })?;

        let cards = self.create(&mut user, question_text).await?;

        self.repository.save(&user).await?;

        Ok(cards)
    }

    async fn create(
        &self,
        user: &mut crate::domain::User,
        question_text: String,
    ) -> Result<Vec<StudyCard>, KeikakuError> {
        let tokenizer = Tokenizer::new()?;
        let tokens = tokenizer.tokenize(question_text.as_str())?;
        let mut cards = Vec::new();

        for token in tokens {
            if !token.part_of_speech().is_vocabulary_word() {
                continue;
            }

            let question_text = token.orthographic_base_form();
            let question = Question::new(question_text.to_string())?;
            let content = self
                .generate_content_use_case
                .generate_content(
                    question_text,
                    user.native_language(),
                    user.current_japanese_level(),
                )
                .await?;

            let vocabulary_card = VocabularyCard::new(question, content.answer, content.examples);
            let card = Card::Vocabulary(vocabulary_card);

            let card_result = user.create_card(card);

            if let Ok(card) = card_result {
                cards.push(card);
            } else {
                error!("Failed to create card: {}", card_result.err().unwrap());
            }
        }

        Ok(cards)
    }
}
