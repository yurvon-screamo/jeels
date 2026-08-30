use crate::domain::OrigaError;
use crate::traits::UserRepository;
use chrono::{Duration, Utc};
use tracing::{debug, info};
use ulid::Ulid;

/// Закрытие руки знакомства (docs/acquaintance-mode.md §9.2): единственный
/// момент планирования для карт руки — каждой ещё новой карте сидируется
/// состояние памяти с первым ревью назавтра, дневной лимит списывается
/// одной операцией за все карты руки. Тренировочные ответы этот путь
/// не проходят.
pub struct CompleteAcquaintanceHandUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CompleteAcquaintanceHandUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, card_ids: Vec<Ulid>) -> Result<(), OrigaError> {
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        debug!(user_id = %user.id(), cards = card_ids.len(), "Completing acquaintance hand");

        let first_due = Utc::now() + Duration::days(1);
        user.complete_acquaintance_hand(&card_ids, first_due)?;

        self.repository.save(&user).await?;

        info!(
            user_id = %user.id(),
            cards = card_ids.len(),
            "Acquaintance hand completed, first review scheduled for tomorrow"
        );
        Ok(())
    }
}
