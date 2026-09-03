//! Header contract for pre-parsed rkyv blobs deployed on the CDN.
//!
//! Layout: `MAGIC (4B) | schema_version (u32 LE) | source_sha256 (32B) |
//! manifest_guard (32B)` followed by the raw rkyv payload. The same module is
//! used by the offline builder (`utils build-cdn-rkyv`) and by the client
//! loaders, so the wire format cannot drift between producer and consumer.
//!
//! `manifest_guard` binds a blob to the manifest hashes of its source files:
//! `sha256(concat(hex_hash_1 .. hex_hash_N))` where the hashes are the ASCII
//! hex strings exactly as they appear in the CDN manifest, concatenated in a
//! fixed path order with no separators. A client that has fetched the remote
//! manifest can recompute the guard from manifest entries alone (no source
//! download) and detect a blob that is stale relative to its sources.

use sha2::{Digest, Sha256};

/// Format magic. Blobs are validated against it before any parsing.
pub const MAGIC: &[u8; 4] = b"ORFG";

/// Version of the header/payload schema. Bump when the payload type changes
/// in a way old clients cannot deserialize; they will fall back to the
/// original text/JSON sources.
pub const SCHEMA_VERSION: u32 = 1;

pub const HEADER_LEN: usize = MAGIC.len() + 4 + 32 + 32;

#[derive(Debug, thiserror::Error)]
pub enum BlobFormatError {
    #[error("blob shorter than the fixed header ({len} bytes)")]
    TooShort { len: usize },
    #[error("blob magic mismatch: expected {expected:?}, got {actual:?}")]
    BadMagic { expected: [u8; 4], actual: [u8; 4] },
    #[error("blob schema version {blob} != supported {supported}")]
    UnsupportedSchema { blob: u32, supported: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobHeader {
    pub schema_version: u32,
    pub source_sha256: [u8; 32],
    pub manifest_guard: [u8; 32],
}

/// What the client can assert about `manifest_guard` for a downloaded blob.
///
/// `Unavailable` means the remote manifest was not fetched at all (offline,
/// network failure): the blob is trusted, matching the trust level of every
/// other Cache API entry today. `Mismatch` means the manifest is present but
/// at least one source path is missing from it — an anomalous state that is
/// conservatively treated as a guard failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardExpectation {
    Unavailable,
    Expect([u8; 32]),
    Mismatch,
}

pub fn sha256_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Assemble a deployable blob: header + rkyv payload.
pub fn build_blob(header: &BlobHeader, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER_LEN + payload.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&header.schema_version.to_le_bytes());
    out.extend_from_slice(&header.source_sha256);
    out.extend_from_slice(&header.manifest_guard);
    out.extend_from_slice(payload);
    out
}

/// Validate the header and split the blob into `(header, payload)`.
pub fn split_blob(blob: &[u8]) -> Result<(BlobHeader, &[u8]), BlobFormatError> {
    if blob.len() < HEADER_LEN {
        return Err(BlobFormatError::TooShort { len: blob.len() });
    }
    let mut magic = [0u8; 4];
    magic.copy_from_slice(&blob[..4]);
    if &magic != MAGIC {
        return Err(BlobFormatError::BadMagic {
            expected: *MAGIC,
            actual: magic,
        });
    }
    let schema_version = u32::from_le_bytes([blob[4], blob[5], blob[6], blob[7]]);
    if schema_version != SCHEMA_VERSION {
        return Err(BlobFormatError::UnsupportedSchema {
            blob: schema_version,
            supported: SCHEMA_VERSION,
        });
    }
    let mut source_sha256 = [0u8; 32];
    source_sha256.copy_from_slice(&blob[8..40]);
    let mut manifest_guard = [0u8; 32];
    manifest_guard.copy_from_slice(&blob[40..72]);

    Ok((
        BlobHeader {
            schema_version,
            source_sha256,
            manifest_guard,
        },
        &blob[HEADER_LEN..],
    ))
}

/// Guard derivation shared by builder and client: SHA-256 over the ASCII hex
/// hashes concatenated in fixed path order, no separators.
pub fn manifest_guard_from_hex_hashes(hex_hashes: &[&str]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for hash in hex_hashes {
        hasher.update(hash.as_bytes());
    }
    hasher.finalize().into()
}

/// Derive the client-side expectation from per-path manifest hashes.
///
/// `None` for the whole slice input means the manifest was not fetched. A
/// `Some` slice with any `None` entry (path absent from the manifest) yields
/// `Mismatch`.
pub fn guard_expectation(hex_hashes_by_path: Option<&[Option<String>]>) -> GuardExpectation {
    let Some(hex_hashes_by_path) = hex_hashes_by_path else {
        return GuardExpectation::Unavailable;
    };
    let Some(hashes) = hex_hashes_by_path
        .iter()
        .map(|entry| entry.as_deref())
        .collect::<Option<Vec<&str>>>()
    else {
        return GuardExpectation::Mismatch;
    };
    GuardExpectation::Expect(manifest_guard_from_hex_hashes(&hashes))
}

/// A blob passes the guard check when the expectation is unavailable
/// (offline) or matches the header value exactly.
pub fn guard_matches(header: &BlobHeader, expectation: &GuardExpectation) -> bool {
    match expectation {
        GuardExpectation::Unavailable => true,
        GuardExpectation::Expect(guard) => header.manifest_guard == *guard,
        GuardExpectation::Mismatch => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_header() -> BlobHeader {
        BlobHeader {
            schema_version: SCHEMA_VERSION,
            source_sha256: sha256_bytes(b"source"),
            manifest_guard: manifest_guard_from_hex_hashes(&["aa", "bb"]),
        }
    }

    #[test]
    fn blob_round_trip_preserves_header_and_payload() {
        let header = sample_header();

        let blob = build_blob(&header, b"payload bytes");
        let (parsed, payload) = split_blob(&blob).expect("valid blob");

        assert_eq!(parsed, header);
        assert_eq!(payload, b"payload bytes");
    }

    #[test]
    fn truncated_blob_is_rejected_as_too_short() {
        let error = split_blob(&[0u8; 16]).expect_err("short blob");

        assert!(matches!(error, BlobFormatError::TooShort { len: 16 }));
    }

    #[test]
    fn foreign_magic_is_rejected() {
        let mut blob = build_blob(&sample_header(), b"payload");
        blob[0] = b'X';

        let error = split_blob(&blob).expect_err("bad magic");

        assert!(matches!(error, BlobFormatError::BadMagic { .. }));
    }

    #[test]
    fn future_schema_version_is_rejected() {
        let mut header = sample_header();
        header.schema_version = SCHEMA_VERSION + 1;
        let blob = build_blob(&header, b"payload");

        let error = split_blob(&blob).expect_err("future schema");

        assert!(matches!(error, BlobFormatError::UnsupportedSchema { .. }));
    }

    #[test]
    fn guard_expectation_is_unavailable_without_manifest() {
        assert_eq!(guard_expectation(None), GuardExpectation::Unavailable);
    }

    #[test]
    fn guard_expectation_is_mismatch_when_any_path_missing() {
        let hashes = [Some("aa".to_string()), None, Some("cc".to_string())];

        assert_eq!(guard_expectation(Some(&hashes)), GuardExpectation::Mismatch);
    }

    #[test]
    fn guard_matches_only_exact_expectation() {
        let header = sample_header();
        let matching = manifest_guard_from_hex_hashes(&["aa", "bb"]);
        let diverging = manifest_guard_from_hex_hashes(&["aa", "cc"]);

        assert!(guard_matches(&header, &GuardExpectation::Expect(matching)));
        assert!(!guard_matches(
            &header,
            &GuardExpectation::Expect(diverging)
        ));
        assert!(!guard_matches(&header, &GuardExpectation::Mismatch));
        assert!(guard_matches(&header, &GuardExpectation::Unavailable));
    }
}
