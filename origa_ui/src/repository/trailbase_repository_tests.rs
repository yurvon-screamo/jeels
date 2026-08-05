use super::*;
use origa::domain::{Card, PhraseCard, RateMode, Rating};
use ulid::Ulid;

fn fixture_user() -> User {
    let mut user = User::new(
        "row-test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let study_card = user
        .create_card(Card::Phrase(PhraseCard::new(Ulid::new())))
        .expect("create_card");
    user.rate_card(
        *study_card.card_id(),
        Rating::Good,
        RateMode::StandardLesson,
    )
    .expect("rate_card");
    user
}

#[test]
fn user_to_json_then_userrow_roundtrip_preserves_knowledge_set() {
    // Arrange
    let user = fixture_user();

    // Act — encode to the wire body, then deserialize back as a UserRow
    // (the shape TrailBase returns), and rebuild the User exactly as the
    // production read path does.
    let body = user_to_json(&user, "00000000-0000-0000-0000-000000000001").expect("user_to_json");
    let row: UserRow = serde_json::from_value(body).expect("UserRow deserialize from wire body");
    let restored = row.to_user();

    // Assert — the deflated wire format is lossless end-to-end: what the
    // write path produces, the read path reconstructs.
    assert_eq!(
        user.knowledge_set(),
        restored.knowledge_set(),
        "knowledge_set must survive the full encode -> wire -> decode roundtrip"
    );
    assert_eq!(user.email(), restored.email());
    assert_eq!(user.username(), restored.username());
}

#[test]
fn userrow_to_user_self_heals_on_corrupt_knowledge_set() {
    // Arrange — a valid wire body, but the knowledge_set field is replaced
    // with an unparseable deflated payload. This models a corrupt remote
    // row (truncated write, bit flip in the BLOB, a partial column write).
    let user = fixture_user();
    let mut body =
        user_to_json(&user, "00000000-0000-0000-0000-000000000002").expect("user_to_json");
    body["knowledge_set"] = serde_json::Value::String("DEFLATE;!!!corrupt-base64!!!".to_string());

    // Act
    let row: UserRow = serde_json::from_value(body).expect("UserRow deserialize from wire body");
    let restored = row.to_user();

    // Assert — self-heal: the read path resolves a corrupt knowledge_set
    // to empty (not panic, not Err, not partial garbage). The subsequent
    // merge_current_user + save_local_and_sync_remote then overwrites the
    // corrupt remote with the device's local data — the device never
    // loses its own progress.
    assert_eq!(
        restored.knowledge_set(),
        &KnowledgeSet::default(),
        "corrupt knowledge_set must self-heal to empty, never panic"
    );
}

/// Regression guard for the cross-device onboarding-repeat bug: the
/// `__onboarding_skipped__` / `__onboarding_completed__` sentinel keys
/// live inside `imported_sets`, so they MUST survive the
/// `user_to_json` → wire → `UserRow::to_user` round-trip. If they are
/// dropped, a user who skips onboarding on device A is shown onboarding
/// again on device B because `is_onboarding_completed()` returns false.
#[test]
fn user_to_json_then_userrow_roundtrip_preserves_onboarding_sentinels() {
    use origa::domain::{ONBOARDING_COMPLETED_KEY, ONBOARDING_SKIPPED_KEY};

    // Arrange — a user who skipped onboarding (skip path writes
    // ONBOARDING_SKIPPED_KEY via mark_set_as_imported).
    let mut user = User::new(
        "sentinel-test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    user.mark_set_as_imported(ONBOARDING_SKIPPED_KEY.to_string());
    assert!(
        user.is_onboarding_completed(),
        "fixture precondition: user must be onboarding-completed before round-trip"
    );

    // Act — encode to the wire body, deserialize back as a UserRow, rebuild.
    let body = user_to_json(&user, "00000000-0000-0000-0000-000000000003").expect("user_to_json");
    let row: UserRow = serde_json::from_value(body).expect("UserRow deserialize from wire body");
    let restored = row.to_user();

    // Assert — both sentinel keys survive, and the routing guard still
    // reports onboarding as completed after the round-trip.
    assert!(
        restored.is_set_imported(ONBOARDING_SKIPPED_KEY),
        "ONBOARDING_SKIPPED_KEY must survive the wire round-trip"
    );
    assert!(
        restored.is_onboarding_completed(),
        "is_onboarding_completed() must be true after the round-trip — \
         a false here is the cross-device onboarding-repeat regression"
    );

    // Also cover the completed-marker path (written by the finish path via
    // mark_onboarding_completed, which inserts ONBOARDING_COMPLETED_KEY).
    let mut finished = User::new(
        "completed-test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    finished.mark_onboarding_completed();
    let body =
        user_to_json(&finished, "00000000-0000-0000-0000-000000000004").expect("user_to_json");
    let row: UserRow = serde_json::from_value(body).expect("UserRow deserialize from wire body");
    let restored = row.to_user();
    assert!(
        restored.is_set_imported(ONBOARDING_COMPLETED_KEY),
        "ONBOARDING_COMPLETED_KEY must survive the wire round-trip"
    );
    assert!(restored.is_onboarding_completed());
}
