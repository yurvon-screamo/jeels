use crate::domain::{Card, CardType, JlptContent, OrigaError};
use crate::traits::UserRepository;
use tracing::debug;
use ulid::Ulid;

/// Замена карты руки знакомства при «Уже знаю»: верхушка пула новых карт
/// по JLPT-приоритету (N5 первым), исключая карты, уже занятые рукой.
/// Пропорции `CARD_TYPE_WEIGHTS` здесь не применяются — замещается один
/// слот, а не собирается рука. Дневной лимит не проверяется: замена не
/// добавляет сидируемых карт — лимит спишется при закрытии руки за
/// фактически сидированные.
#[derive(Clone)]
pub struct TakeAcquaintanceReplacementUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> TakeAcquaintanceReplacementUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        jlpt_content: &JlptContent,
        exclude: &[Ulid],
    ) -> Result<Option<(Ulid, CardType)>, OrigaError> {
        let user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        let excluded: std::collections::HashSet<Ulid> = exclude.iter().copied().collect();
        let candidate = user
            .knowledge_set()
            .study_cards()
            .iter()
            .filter(|(card_id, study_card)| {
                study_card.memory().is_new()
                    && !matches!(study_card.card(), Card::Phrase(_))
                    && !excluded.contains(*card_id)
            })
            .map(|(card_id, study_card)| {
                let card_type = CardType::from(study_card.card());
                let priority = crate::domain::jlpt_sort_key(study_card.card(), jlpt_content);
                (priority, *card_id, card_type)
            })
            .max_by(|left, right| left.0.cmp(&right.0).then(right.1.cmp(&left.1)));

        match candidate {
            Some((_priority, card_id, card_type)) => {
                debug!(user_id = %user.id(), replacement = %card_id, "Acquaintance replacement taken");
                Ok(Some((card_id, card_type)))
            },
            None => {
                debug!(user_id = %user.id(), "Acquaintance replacement pool empty");
                Ok(None)
            },
        }
    }
}
