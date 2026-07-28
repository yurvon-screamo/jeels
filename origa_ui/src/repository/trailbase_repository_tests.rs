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
