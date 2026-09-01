use crate::domain::{DailyLoad, NativeLanguage, OrigaError};
use crate::traits::UserRepository;
use tracing::{debug, info};

/// Cap for the display name persisted via [`UpdateUserProfileUseCase`].
/// Keeps the free-form name bounded without rejecting the user's input.
pub const USERNAME_MAX_CHARS: usize = 40;

#[derive(Clone)]
pub struct UpdateUserProfileUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> UpdateUserProfileUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    /// Updates profile fields. `username` is a free-form display name: `None`
    /// keeps the current value; `Some` trims it and caps it at
    /// [`USERNAME_MAX_CHARS`]. An empty name is allowed — the UI displays the
    /// account email in its place.
    pub async fn execute(
        &self,
        native_language: NativeLanguage,
        daily_load: DailyLoad,
        telegram_user_id: Option<u64>,
        username: Option<&str>,
    ) -> Result<(), OrigaError> {
        debug!("Updating user profile");

        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        user.set_native_language(native_language);
        user.set_daily_load(daily_load);
        user.set_telegram_user_id(telegram_user_id);

        if let Some(username) = username {
            let trimmed = username.trim();
            let capped: String = trimmed.chars().take(USERNAME_MAX_CHARS).collect();
            user.set_username(capped);
        }

        self.repository.save_sync(&user).await?;

        info!("User profile updated");
        Ok(())
    }
}
