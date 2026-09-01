use std::time::Instant;

use crate::domain::{JapaneseLevel, NativeLanguage, User};
use crate::traits::UserRepository;
use crate::use_cases::ImportOnboardingSetsUseCase;
use crate::use_cases::tests::fixtures::{
    InMemoryUserRepository, MockCdnProvider, get_cdn_dir, init_real_dictionaries,
};

/// Runs the full onboarding import for the given well-known sets exactly as
/// the onboarding UI queues them (real production JSONs from `cdn/`), on a
/// fresh user, returning `(created card count, elapsed time)`.
async fn import_sets(
    set_ids: &[&str],
    target_level: JapaneseLevel,
) -> (usize, std::time::Duration) {
    init_real_dictionaries();
    let mut cdn = MockCdnProvider::new();
    for id in set_ids {
        let body = std::fs::read_to_string(
            get_cdn_dir()
                .join("well_known_set")
                .join(format!("{id}.json")),
        )
        .unwrap_or_else(|e| panic!("{id}.json must exist in cdn/: {e}"));
        cdn = cdn.with_text(&crate::domain::resolve_set_path(id), &body);
    }
    let repo = InMemoryUserRepository::with_user(User::new(
        "n2-import@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    ));
    let use_case = ImportOnboardingSetsUseCase::new(&repo, &cdn);
    let start = Instant::now();
    let result = use_case
        .execute(
            repo.get_current_user().await.unwrap().unwrap(),
            set_ids.iter().map(|s| s.to_string()).collect(),
            target_level,
        )
        .await
        .unwrap();
    let created = result.created_vocabulary + result.created_kanji + result.created_grammar;
    (created, start.elapsed())
}

/// Best-of-3 import time per scenario: the minimum is robust against
/// asymmetric CPU contention from parallel tests in the shared test pool,
/// which would otherwise skew a single-shot ratio measurement.
async fn fastest_import(
    set_ids: &[&str],
    target_level: JapaneseLevel,
) -> (usize, std::time::Duration) {
    let mut best = std::time::Duration::MAX;
    let mut cards = 0;
    for _ in 0..3 {
        let (created, elapsed) = import_sets(set_ids, target_level).await;
        cards = created;
        best = best.min(elapsed);
    }
    (cards, best)
}

/// Regression guard for the "N2 onboarding import is unusably slow" bug.
/// Every `create_card` used to run full scans over all existing cards
/// (uniqueness validation + daily-stats recalculation), so import time grew
/// quadratically with the card count: per-card cost for an N2 import
/// (~6300 cards) measured ~5x the per-card cost of an N5 import (~900
/// cards). With linear per-card cost the ratio stays near 1; the 2x ceiling
/// is generous for CI noise but fails loudly on quadratic behaviour.
#[tokio::test]
async fn import_cost_grows_linearly_with_card_count() {
    // Arrange — picking N5 queues only jlpt_n5; picking N2 cumulatively queues
    // n5+n4+n3+n2 (see `OnboardingState::set_jlpt_level`).
    let (n5_cards, n5_elapsed) = fastest_import(&["jlpt_n5"], JapaneseLevel::N5).await;
    let (n2_cards, n2_elapsed) = fastest_import(
        &["jlpt_n5", "jlpt_n4", "jlpt_n3", "jlpt_n2"],
        JapaneseLevel::N2,
    )
    .await;

    // Sanity: the N2 run must be the bigger import, otherwise the ratio
    // asserted below would pass vacuously.
    assert!(
        n2_cards > 5 * n5_cards,
        "N2 onboarding must create several times more cards than N5 \
         (n5={n5_cards}, n2={n2_cards})",
    );

    // Act — per-card cost of each scenario's best run
    let per_card_n5 = n5_elapsed.as_nanos() / n5_cards.max(1) as u128;
    let per_card_n2 = n2_elapsed.as_nanos() / n2_cards.max(1) as u128;

    // Assert
    assert!(
        per_card_n2 < per_card_n5 * 2,
        "per-card import cost must stay flat as the card count grows — \
         quadratic card creation is the N2 slow-import bug \
         (n5={per_card_n5} ns/card over {n5_cards} cards, \
         n2={per_card_n2} ns/card over {n2_cards} cards, \
         ratio={:.2})",
        per_card_n2 as f64 / per_card_n5 as f64,
    );
}
