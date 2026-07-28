use super::*;
use origa::domain::{Card, NativeLanguage, PhraseCard, RateMode, Rating, User};
use rstest::rstest;
use ulid::Ulid;

fn fixture_knowledge_set() -> KnowledgeSet {
    let mut user = User::new(
        "codec-test@example.com".to_string(),
        NativeLanguage::Russian,
        None,
    );
    let ratings = [Rating::Easy, Rating::Good, Rating::Hard, Rating::Again];
    for i in 0..6 {
        let study_card = user
            .create_card(Card::Phrase(PhraseCard::new(Ulid::new())))
            .expect("create_card");
        let card_id = *study_card.card_id();
        user.rate_card(
            card_id,
            ratings[i % ratings.len()],
            RateMode::StandardLesson,
        )
        .expect("rate_card");
    }
    user.knowledge_set().clone()
}

#[test]
fn encode_then_decode_strict_preserves_knowledge_set() {
    // Arrange
    let original = fixture_knowledge_set();

    // Act
    let wire = encode(&original).expect("encode");
    let restored = decode_strict(&wire).expect("decode_strict");

    // Assert — structural equality: the wire format is lossless.
    assert_eq!(original, restored);
}

#[test]
fn encode_output_carries_deflate_prefix() {
    // Arrange
    let ks = fixture_knowledge_set();

    // Act
    let wire = encode(&ks).expect("encode");

    // Assert — wire-format contract: the self-describing magic prefix is
    // present so the read path can discriminate the format unambiguously.
    assert!(
        wire.starts_with(DEFLATE_PREFIX),
        "wire must carry the deflate prefix"
    );
}

#[test]
fn encode_output_is_smaller_than_raw_json() {
    // Arrange
    let ks = fixture_knowledge_set();
    let raw_json = serde_json::to_string(&ks).expect("json");

    // Act
    let wire = encode(&ks).expect("encode");

    // Assert — compression effectiveness: the deflated wire string is
    // smaller than the uncompressed JSON it replaces.
    assert!(
        wire.len() < raw_json.len(),
        "deflated wire ({}) must be smaller than raw json ({})",
        wire.len(),
        raw_json.len()
    );
}

#[test]
fn decode_legacy_plain_json_returns_knowledge_set() {
    // Arrange — a pre-upgrade remote row stores plain JSON, no prefix.
    let ks = fixture_knowledge_set();
    let legacy_wire = serde_json::to_string(&ks).expect("json");

    // Act
    let restored = decode_strict(&legacy_wire).expect("decode_strict legacy");

    // Assert — back-compat: existing rows are readable by the new client.
    assert_eq!(ks, restored);
}

// The recovering policy (`decode`) is the self-heal contract: anything the
// codec cannot parse resolves to an empty KnowledgeSet, never panics, never
// returns partial data. Every corruption class shares that contract, so they
// collapse into one parameterized test — each `#[case]` documents a distinct
// corruption shape the contract must absorb.
#[rstest]
#[case::corrupt_legacy_json(
    r#"{"study_cards":{"broken":"truncated;#[unterminated"#.to_string()
)]
#[case::corrupt_deflated_payload(format!(
    "{DEFLATE_PREFIX}!!!not-valid-base64-or-deflate!!!"
))]
#[case::unknown_prefix("ZSTD;some-future-format-payload".to_string())]
fn decode_recovering_returns_empty_on_unparseable_input(#[case] corrupt: String) {
    // Act
    let restored = decode(&corrupt);

    // Assert — self-heal: corrupt remote resolves to empty so the subsequent
    // merge is a no-op and local data overwrites remote.
    assert_eq!(restored, KnowledgeSet::default());
}
