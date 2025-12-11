use crate::application::user_repository::UserRepository;
use crate::domain::daily_history::DailyHistoryItem;
use crate::domain::error::JeersError;
use crate::domain::value_objects::{JapaneseLevel, NativeLanguage};
use ulid::Ulid;

#[derive(Clone, Debug)]
pub struct UserProfile {
    pub id: Ulid,
    pub username: String,
    pub current_japanese_level: JapaneseLevel,
    pub native_language: NativeLanguage,
    pub lesson_history: Vec<DailyHistoryItem>,
}

#[derive(Clone)]
pub struct GetUserInfoUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> GetUserInfoUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Ulid) -> Result<UserProfile, JeersError> {
        let user = self
            .repository
            .find_by_id(user_id)
            .await?
            .ok_or(JeersError::UserNotFound { user_id })?;

        Ok(UserProfile {
            id: user.id(),
            username: user.username().to_string(),
            current_japanese_level: user.current_japanese_level().clone(),
            native_language: user.native_language().clone(),
            lesson_history: user.lesson_history().to_vec(),
        })
    }
}
