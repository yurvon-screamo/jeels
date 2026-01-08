use crate::application::user_repository::UserRepository;
use crate::domain::error::KeikakuError;
use crate::domain::knowledge::StudyCard;
use ulid::Ulid;

#[derive(Clone)]
pub struct KnowledgeSetCardsUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> KnowledgeSetCardsUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Ulid) -> Result<Vec<StudyCard>, KeikakuError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(KeikakuError::UserNotFound { user_id })?;

        Ok(user
            .knowledge_set()
            .study_cards()
            .values()
            .cloned()
            .collect())
    }
}
