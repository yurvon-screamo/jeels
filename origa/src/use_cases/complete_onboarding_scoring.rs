use tracing::{info, warn};

use crate::domain::OrigaError;
use crate::traits::UserRepository;
use crate::use_cases::SeedReadyPhrasesUseCase;

/// Atomically finishes onboarding scoring: clears the per-click "don't know"
/// records collected during onboarding scoring, marks the user's onboarding
/// as completed (so routing no longer redirects to `/onboarding`), persists
/// both via a single `save_sync`, and then seeds ready-to-learn phrase cards
/// for the now-known vocabulary.
///
/// Encapsulated as a domain use case (rather than living in a Leptos callback)
/// so the side-effect chain is unit-testable with `InMemoryUserRepository`
/// inside the `origa` crate and so that the UI layer stays a thin shell.
///
/// Failure of [`SeedReadyPhrasesUseCase`] is treated as non-fatal: phrases are
/// a derivative payload, and the user must still be allowed to proceed to
/// `/home`. The error is logged and the use case returns `Ok(0)`.
#[derive(Clone)]
pub struct CompleteOnboardingScoringUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> CompleteOnboardingScoringUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    /// Returns the number of phrase cards created (0 when phrases are not
    /// loaded yet or when no known vocabulary exists).
    pub async fn execute(&self) -> Result<usize, OrigaError> {
        let mut user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        user.clear_onboarding_scoring_skipped();
        user.mark_onboarding_completed();

        // Persist the cleared skipped set and the completed marker together
        // before any derivative work reads the user, so a crash between the
        // two writes cannot leave the user in a half-finished state.
        self.repository.save_sync(&user).await?;

        let phrase_count = match SeedReadyPhrasesUseCase::new(self.repository)
            .execute()
            .await
        {
            Ok(n) => n,
            Err(e) => {
                warn!(error = ?e, "SeedReadyPhrases failed during onboarding completion");
                0
            },
        };

        info!(
            user_id = %user.id(),
            seeded_phrases = phrase_count,
            "Onboarding scoring completed"
        );

        Ok(phrase_count)
    }
}
