//! Offline builder for pre-parsed rkyv blobs deployed to the CDN.
//!
//! Produces `dictionaries/JmdictFurigana.rkyv` and `dictionary/vocabulary.rkyv`
//! from the editable sources (`JmdictFurigana.txt`, `chunk_01..11.json`).
//! Clients load these blobs directly instead of parsing text/JSON on every
//! cold start; the sources stay on the CDN as fallback and source-of-truth.
//!
//! Blobs embed `source_sha256` and a `manifest_guard` (derived from the same
//! per-file SHA-256 hex hashes the deploy script puts into the CDN manifest),
//! so clients can verify a blob against the manifest without downloading the
//! sources.

use std::fs;
use std::path::{Path, PathBuf};

use origa::dictionary::cdn_blob::{self, BlobHeader, SCHEMA_VERSION, build_blob, split_blob};
use origa::dictionary::furigana_dict::{
    build_furigana_dict_from_text, serialize_furigana_dict_to_rkyv,
};
use origa::dictionary::vocabulary::{
    VocabularyChunkData, build_vocabulary_database_from_chunks, serialize_vocabulary_blob_to_rkyv,
};
use origa::domain::OrigaError;
use sha2::{Digest, Sha256};

const FURIGANA_SOURCE: &str = "dictionaries/JmdictFurigana.txt";
const FURIGANA_BLOB: &str = "dictionaries/JmdictFurigana.rkyv";
const VOCABULARY_BLOB: &str = "dictionary/vocabulary.rkyv";
const CHUNK_COUNT: usize = 11;

fn sha256_hex(data: &[u8]) -> String {
    let digest: [u8; 32] = Sha256::digest(data).into();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn sha256_raw(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

fn chunk_path(index: usize) -> String {
    format!("dictionary/chunk_{index:02}.json")
}

fn read(cdn_dir: &Path, relative: &str) -> Result<Vec<u8>, String> {
    let path = cdn_dir.join(relative);
    fs::read(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))
}

fn write(cdn_dir: &Path, relative: &str, bytes: &[u8]) -> Result<(), String> {
    let path = cdn_dir.join(relative);
    fs::write(&path, bytes).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

/// Existing blob is skipped only when its header is fresh: same schema
/// version and same source hash as the sources on disk.
fn blob_is_fresh(existing: Option<&[u8]>, schema_version: u32, source_sha256: &[u8; 32]) -> bool {
    match existing.map(split_blob) {
        Some(Ok((header, _))) => {
            header.schema_version == schema_version && header.source_sha256 == *source_sha256
        },
        _ => false,
    }
}

fn deploy_blob(
    cdn_dir: &Path,
    blob_path: &str,
    source_sha256: [u8; 32],
    manifest_guard: [u8; 32],
    payload: Vec<u8>,
) -> Result<bool, String> {
    let existing = read(cdn_dir, blob_path).ok();
    if blob_is_fresh(existing.as_deref(), SCHEMA_VERSION, &source_sha256) {
        tracing::info!("{} is fresh, skipping regeneration", blob_path);
        return Ok(false);
    }

    let header = BlobHeader {
        schema_version: SCHEMA_VERSION,
        source_sha256,
        manifest_guard,
    };
    write(cdn_dir, blob_path, &build_blob(&header, &payload))?;
    tracing::info!("{} written ({} bytes payload)", blob_path, payload.len());
    Ok(true)
}

fn build_furigana(cdn_dir: &Path) -> Result<bool, String> {
    let source = read(cdn_dir, FURIGANA_SOURCE)?;
    let source_hex = sha256_hex(&source);

    // Freshness check BEFORE the expensive parse+serialize: a fresh blob
    // costs one file read, not a full 12 MB text re-parse.
    if blob_is_fresh(
        read(cdn_dir, FURIGANA_BLOB).ok().as_deref(),
        SCHEMA_VERSION,
        &sha256_raw(&source),
    ) {
        tracing::info!("{} is fresh, skipping regeneration", FURIGANA_BLOB);
        return Ok(false);
    }

    let text = String::from_utf8(source.clone())
        .map_err(|e| format!("{FURIGANA_SOURCE} is not valid UTF-8: {e}"))?;
    let dict = build_furigana_dict_from_text(&text)
        .map_err(|e| format!("failed to build furigana dictionary: {e}"))?;
    let payload = serialize_furigana_dict_to_rkyv(&dict)
        .map_err(|e| format!("failed to serialize furigana dictionary: {e}"))?;

    let guard = cdn_blob::manifest_guard_from_hex_hashes(&[&source_hex]);
    deploy_blob(cdn_dir, FURIGANA_BLOB, sha256_raw(&source), guard, payload)
}

fn build_vocabulary(cdn_dir: &Path) -> Result<bool, String> {
    let mut chunks: Vec<(String, Vec<u8>)> = Vec::with_capacity(CHUNK_COUNT);
    let mut concatenated = Vec::new();
    for index in 1..=CHUNK_COUNT {
        let path = chunk_path(index);
        let bytes = read(cdn_dir, &path)?;
        concatenated.extend_from_slice(&bytes);
        chunks.push((path, bytes));
    }

    // Freshness check BEFORE the expensive JSON parse + rkyv serialize:
    // chunk reads are cheap, the parse is not.
    if blob_is_fresh(
        read(cdn_dir, VOCABULARY_BLOB).ok().as_deref(),
        SCHEMA_VERSION,
        &sha256_raw(&concatenated),
    ) {
        tracing::info!("{} is fresh, skipping regeneration", VOCABULARY_BLOB);
        return Ok(false);
    }

    let chunk_data = assemble_chunk_data(&chunks)?;

    let db = build_vocabulary_database_from_chunks(chunk_data)
        .map_err(|e| format!("failed to build vocabulary database: {e}"))?;
    let payload = serialize_vocabulary_blob_to_rkyv(&db)
        .map_err(|e| format!("failed to serialize vocabulary blob: {e}"))?;

    let hex_hashes: Vec<String> = chunks.iter().map(|(_, b)| sha256_hex(b)).collect();
    let hex_refs: Vec<&str> = hex_hashes.iter().map(String::as_str).collect();
    let guard = cdn_blob::manifest_guard_from_hex_hashes(&hex_refs);

    deploy_blob(
        cdn_dir,
        VOCABULARY_BLOB,
        sha256_raw(&concatenated),
        guard,
        payload,
    )
}

fn assemble_chunk_data(chunks: &[(String, Vec<u8>)]) -> Result<VocabularyChunkData, String> {
    let parse = |index: usize| -> Result<String, String> {
        let bytes = &chunks[index - 1].1;
        String::from_utf8(bytes.clone())
            .map_err(|e| format!("{} is not valid UTF-8: {e}", chunk_path(index)))
    };

    Ok(VocabularyChunkData {
        chunk_01: parse(1)?,
        chunk_02: parse(2)?,
        chunk_03: parse(3)?,
        chunk_04: parse(4)?,
        chunk_05: parse(5)?,
        chunk_06: parse(6)?,
        chunk_07: parse(7)?,
        chunk_08: parse(8)?,
        chunk_09: parse(9)?,
        chunk_10: parse(10)?,
        chunk_11: parse(11)?,
    })
}

fn default_cdn_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir has a parent")
        .join("cdn")
}

pub fn run_build_cdn_rkyv(cdn_dir: Option<PathBuf>) -> Result<(), OrigaError> {
    let cdn_dir = cdn_dir.unwrap_or_else(default_cdn_dir);

    let furigana_rebuilt =
        build_furigana(&cdn_dir).map_err(|reason| OrigaError::RepositoryError { reason })?;
    let vocabulary_rebuilt =
        build_vocabulary(&cdn_dir).map_err(|reason| OrigaError::RepositoryError { reason })?;

    if !furigana_rebuilt && !vocabulary_rebuilt {
        println!("All CDN rkyv blobs are fresh, nothing to do");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_cdn(contents: &[(String, Vec<u8>)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("origa-rkyv-test-{}", ulid::Ulid::new()));
        fs::create_dir_all(dir.join("dictionaries")).unwrap();
        fs::create_dir_all(dir.join("dictionary")).unwrap();
        for (path, bytes) in contents {
            fs::write(dir.join(path), bytes).unwrap();
        }
        dir
    }

    fn furigana_source() -> Vec<u8> {
        b"\xe6\x8c\x87|\xe3\x82\x86\xe3\x81\xb3|0:\xe3\x82\x86\xe3\x81\xb3\n".to_vec()
    }

    fn chunk_source() -> Vec<u8> {
        br#"{"\u732b": {"russian_translation": "\u043a\u043e\u0442"}}"#.to_vec()
    }

    fn all_sources() -> Vec<(String, Vec<u8>)> {
        let mut files = vec![(FURIGANA_SOURCE.to_string(), furigana_source())];
        for index in 1..=CHUNK_COUNT {
            files.push((chunk_path(index), chunk_source()));
        }
        files
    }

    #[test]
    fn build_writes_blobs_and_second_run_skips() {
        // Arrange
        let dir = temp_cdn(&all_sources());

        // Act
        let first_furigana = build_furigana(&dir).unwrap();
        let first_vocabulary = build_vocabulary(&dir).unwrap();
        let second_furigana = build_furigana(&dir).unwrap();
        let second_vocabulary = build_vocabulary(&dir).unwrap();

        // Assert
        assert!(first_furigana, "first run rebuilds");
        assert!(first_vocabulary, "first run rebuilds");
        assert!(!second_furigana, "fresh blob is skipped");
        assert!(!second_vocabulary, "fresh blob is skipped");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn changed_source_invalidates_freshness() {
        // Arrange
        let dir = temp_cdn(&all_sources());
        build_furigana(&dir).unwrap();

        // Act: edit the source
        fs::write(dir.join(FURIGANA_SOURCE), b"\n").unwrap();
        let rebuilt = build_furigana(&dir).unwrap();

        // Assert
        assert!(rebuilt, "stale blob is regenerated");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn blobs_are_deterministic_for_identical_sources() {
        // Arrange
        let first_dir = temp_cdn(&all_sources());
        let second_dir = temp_cdn(&all_sources());

        // Act
        build_vocabulary(&first_dir).unwrap();
        build_vocabulary(&second_dir).unwrap();

        // Assert
        let first = fs::read(first_dir.join(VOCABULARY_BLOB)).unwrap();
        let second = fs::read(second_dir.join(VOCABULARY_BLOB)).unwrap();
        assert_eq!(
            first, second,
            "identical sources must produce identical blobs"
        );

        fs::remove_dir_all(&first_dir).ok();
        fs::remove_dir_all(&second_dir).ok();
    }
}
