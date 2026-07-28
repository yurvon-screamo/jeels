//! Wire-format codec for `KnowledgeSet` on the TrailBase remote path.
//!
//! The remote `user` table stores `knowledge_set` in a single `TEXT`
//! column. Previously this was a plain JSON string; an active user's set
//! grows to multiple megabytes and every checkpoint PUT shipped the full
//! blob uncompressed. This codec compresses it.
//!
//! Wire format (self-describing, discriminated by a magic prefix):
//!
//! - new:   `"DEFLATE;" + base64(deflate(json_string))`
//! - legacy (existing rows, untouched): plain JSON, which always starts
//!   with `{`. The prefix can never be a prefix of a valid JSON object,
//!   so the discriminator is unambiguous and does not depend on a
//!   sibling column staying in sync.
//!
//! base64 is unavoidable: the column is `TEXT`, so raw deflate bytes
//! cannot be stored directly. Its ~33% overhead is more than offset by
//! deflate's compression ratio (measured 4.69x on a representative
//! fixture — see `tests/knowledge_set_format_poc.rs`).
//!
//! Error policy (see ADR for the full rationale):
//!
//! - **Read is tolerant.** `decode` never fails: any corruption
//!   (truncated base64, bad deflate stream, malformed JSON, unknown
//!   prefix) is logged and replaced with an empty `KnowledgeSet`. This
//!   preserves the existing self-heal: a corrupt remote merges as a
//!   no-op, then the next `save_local_and_sync_remote` overwrites the
//!   corrupt row with the device's local data.
//! - **Write is strict.** `encode` returns `Result`. A serialization
//!   failure surfaces as an error instead of silently writing a
//!   corrupt fallback to the remote.
//!
//! Both policies share a single parser: `decode` delegates to
//! `decode_strict`, so bugs in the parse path are caught by the
//! roundtrip tests that exercise the strict variant.

use std::io::Read;
use std::io::Write;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use origa::domain::{KnowledgeSet, OrigaError};

#[cfg(test)]
#[path = "knowledge_set_codec_tests.rs"]
mod tests;

/// Marks a deflated wire string. JSON objects start with `{`, which this
/// prefix cannot collide with, so presence/absence is an unambiguous
/// format discriminator that needs no companion field.
const DEFLATE_PREFIX: &str = "DEFLATE;";

/// Deflate compression level for the encode path. Chosen from the PoC
/// gate (see `tests/knowledge_set_format_poc.rs`): on a representative
/// ~8 MiB fixture, level 6 reaches a 4.69x wire-size reduction at ~197ms
/// encode+decode (native release, ~400ms projected WASM) — well within
/// the sync-checkpoint latency budget. Level 9 buys only +0.06x ratio
/// for a disproportionate encode cost; level 1 loses meaningful ratio.
const DEFLATE_LEVEL: u32 = 6;

pub fn encode(ks: &KnowledgeSet) -> Result<String, OrigaError> {
    let json = serde_json::to_string(ks).map_err(|e| OrigaError::RepositoryError {
        reason: format!("knowledge_set json encode failed: {e}"),
    })?;
    let deflated = deflate(json.as_bytes())?;
    Ok(format!("{DEFLATE_PREFIX}{}", BASE64.encode(&deflated)))
}

pub fn decode(raw: &str) -> KnowledgeSet {
    decode_strict(raw).unwrap_or_else(|e| {
        tracing::warn!(
            error = %e,
            "knowledge_set remote decode failed; self-heal via empty KnowledgeSet"
        );
        KnowledgeSet::default()
    })
}

/// Single parse path. `decode` wraps this with the recovering policy;
/// roundtrip tests call it directly to assert correctness (not recovery).
pub fn decode_strict(raw: &str) -> Result<KnowledgeSet, OrigaError> {
    if let Some(b64) = raw.strip_prefix(DEFLATE_PREFIX) {
        let deflated = BASE64
            .decode(b64)
            .map_err(|e| OrigaError::RepositoryError {
                reason: format!("knowledge_set base64 decode failed: {e}"),
            })?;
        let json_bytes = inflate(&deflated)?;
        serde_json::from_slice(&json_bytes).map_err(|e| OrigaError::RepositoryError {
            reason: format!("knowledge_set json decode failed: {e}"),
        })
    } else {
        serde_json::from_str(raw).map_err(|e| OrigaError::RepositoryError {
            reason: format!("knowledge_set legacy json decode failed: {e}"),
        })
    }
}

fn deflate(input: &[u8]) -> Result<Vec<u8>, OrigaError> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(DEFLATE_LEVEL));
    encoder
        .write_all(input)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("knowledge_set deflate write failed: {e}"),
        })?;
    encoder.finish().map_err(|e| OrigaError::RepositoryError {
        reason: format!("knowledge_set deflate finish failed: {e}"),
    })
}

fn inflate(input: &[u8]) -> Result<Vec<u8>, OrigaError> {
    let mut decoder = DeflateDecoder::new(input);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| OrigaError::RepositoryError {
            reason: format!("knowledge_set inflate failed: {e}"),
        })?;
    Ok(out)
}
