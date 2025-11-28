use std::collections::HashMap;

use crate::application::FuriganaService;
use crate::application::user_repository::UserRepository;
use crate::domain::error::JeersError;
use crate::domain::value_objects::StudySessionItem;
use ulid::Ulid;

#[derive(Clone)]
pub struct StartStudySessionUseCase<'a, R: UserRepository, F: FuriganaService> {
    repository: &'a R,
    furigana_service: &'a F,
}

impl<'a, R: UserRepository, F: FuriganaService> StartStudySessionUseCase<'a, R, F> {
    pub fn new(repository: &'a R, furigana_service: &'a F) -> Self {
        Self {
            repository,
            furigana_service,
        }
    }

    pub async fn execute(
        &self,
        user_id: Ulid,
        force_new_cards: bool,
    ) -> Result<Vec<StudySessionItem>, JeersError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        let mut study_session_items = user.start_study_session(force_new_cards);

        for item in &mut study_session_items {
            let furigana = self.furigana_service.get_furigana(item.question());
            item.set_furigana(furigana);

            let mut map = HashMap::new();
            for similarity in item.similarity() {
                map.insert(
                    similarity.card_id(),
                    self.furigana_service.get_furigana(similarity.question()),
                );
            }

            for (card_id, furigana) in map {
                item.set_similarity_furigana(card_id, furigana);
            }
        }

        Ok(study_session_items)
    }
}
