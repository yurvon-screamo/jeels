//! Content fingerprint of a remote `domain_user` wire row.
//!
//! The sync short-circuit (ADR-045) needs a cheap way to tell "the remote
//! row is byte-for-byte the same content we last synchronized" without
//! materializing the row into a `User` (an 8 MB JSON inflate+parse for a
//! large knowledge set). The fingerprint is computed from the raw wire
//! [`serde_json::Value`] before deserialization:
//!
//! - **Structural**: every field the server sends participates, including
//!   columns the `UserRow` struct does not (yet) declare — a forgotten
//!   struct field cannot silently drop out of the fingerprint.
//! - **Volatile-field exclusion**: `updated_at` changes on every PATCH even
//!   when the content is identical, so it is skipped; every other column
//!   only changes when actual content changes.
//! - **Canonical**: `serde_json` maps iterate in `BTreeMap` order (the
//!   workspace does not enable `preserve_order`), so key order in the
//!   server response cannot perturb the hash.
//! - **Streaming**: values are serialized straight into the hasher — no
//!   intermediate string materialized.

use serde_json::Value;
use sha2::Digest;
use sha2::Sha256;

/// Field set by the server on every write; excluded from the fingerprint
/// (see the module documentation).
const VOLATILE_FIELD: &str = "updated_at";

/// Computes the content fingerprint of a wire row.
pub(crate) fn wire_row_fingerprint(row: &Value) -> String {
    let mut hasher = Sha256::new();

    match row {
        Value::Object(map) => {
            for (key, value) in map {
                if key == VOLATILE_FIELD {
                    continue;
                }
                // Key and value are separated by a NUL byte, which cannot
                // occur in a JSON map key, so adjacent fields can never
                // collide (`a="x",b="y"` vs `a="x", ...` rearrangements).
                hasher.update(key.as_bytes());
                hasher.update([0]);
                // The hasher writer is infallible, so serialization cannot
                // fail; a non-object row is hashed as a whole below.
                let _ = serde_json::to_writer(HashWriter(&mut hasher), value);
            }
        },
        other => {
            let _ = serde_json::to_writer(HashWriter(&mut hasher), other);
        },
    }

    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

/// Feeds serialized bytes straight into the hasher without materializing
/// them. `io::Write` cannot fail by construction, making the serde
/// serialization above infallible.
struct HashWriter<'a>(&'a mut Sha256);

impl std::io::Write for HashWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn base_row() -> Value {
        json!({
            "id": 7,
            "trailbase_id": "018f3f21-7f9f-7bbb-ade0-8d4d9e16c7e1",
            "username": "yuko",
            "email": "yuko@example.com",
            "native_language": 1,
            "jlpt_progress": "{\"n5\":10}",
            "current_japanese_level": 2,
            "telegram_user_id": null,
            "knowledge_set": "DEFLATE;eJyrVspLzE1VslJQcnTHAgoBw6UWqw==",
            "imported_sets": "[\"jlpt_n5\"]",
            "daily_load": 3,
            "known_vocab_hash": 91,
            "updated_at": "2026-09-02T18:00:00Z"
        })
    }

    #[test]
    fn updated_at_change_keeps_fingerprint() {
        let mut row = base_row();
        let fp_before = wire_row_fingerprint(&row);
        row["updated_at"] = json!("2027-01-01T00:00:00Z");
        assert_eq!(wire_row_fingerprint(&row), fp_before);
    }

    #[test]
    fn key_order_does_not_affect_fingerprint() {
        let row = base_row();
        // Same object with a different key insertion order: serde_json maps
        // iterate in BTreeMap order, so the fingerprint is order-stable.
        let reordered = json!({
            "updated_at": "2026-09-02T18:00:00Z",
            "known_vocab_hash": 91,
            "daily_load": 3,
            "imported_sets": "[\"jlpt_n5\"]",
            "knowledge_set": "DEFLATE;eJyrVspLzE1VslJQcnTHAgoBw6UWqw==",
            "telegram_user_id": null,
            "current_japanese_level": 2,
            "jlpt_progress": "{\"n5\":10}",
            "native_language": 1,
            "email": "yuko@example.com",
            "username": "yuko",
            "trailbase_id": "018f3f21-7f9f-7bbb-ade0-8d4d9e16c7e1",
            "id": 7
        });
        assert_eq!(wire_row_fingerprint(&row), wire_row_fingerprint(&reordered));
    }

    #[rstest::rstest]
    #[case::trailbase_id("trailbase_id", json!("018f3f21-0000-0000-0000-000000000000"))]
    #[case::username("username", json!("yuko2"))]
    #[case::email("email", json!("other@example.com"))]
    #[case::native_language("native_language", json!(0))]
    #[case::jlpt_progress("jlpt_progress", json!("{\"n4\":5}"))]
    #[case::current_japanese_level("current_japanese_level", json!(4))]
    #[case::telegram_user_id("telegram_user_id", json!(12345))]
    #[case::knowledge_set("knowledge_set", json!("DEFLATE;eJyrVOTHER=="))]
    #[case::imported_sets("imported_sets", json!("[]"))]
    #[case::daily_load("daily_load", json!(5))]
    #[case::known_vocab_hash("known_vocab_hash", json!(92))]
    #[case::record_id("id", json!(8))]
    fn content_field_change_alters_fingerprint(#[case] field: &str, #[case] value: Value) {
        let mut row = base_row();
        row[field] = value;
        assert_ne!(
            wire_row_fingerprint(&row),
            wire_row_fingerprint(&base_row()),
            "changing `{field}` must alter the fingerprint"
        );
    }

    #[test]
    fn unknown_future_column_alters_fingerprint() {
        // The structural guarantee: a column the deserialization struct
        // does not declare still participates in the fingerprint.
        let mut row = base_row();
        row["new_column_v2"] = json!("value");
        assert_ne!(
            wire_row_fingerprint(&row),
            wire_row_fingerprint(&base_row())
        );
    }

    #[test]
    fn non_object_row_is_deterministic() {
        let scalar = json!("not-a-row");
        assert_eq!(wire_row_fingerprint(&scalar), wire_row_fingerprint(&scalar));
    }
}
