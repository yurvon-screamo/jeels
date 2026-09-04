//! Well-known sets JLPT level audit (#178 S-3, hardened for the 2026-09
//! level-distribution fix).
//!
//! The Duolingo level source of truth is the `level` field carried by every
//! content file under `cdn/well_known_set/duolingo/` (corpus ground truth:
//! Section/Модуль 1-3 → N5, 4 → N4, 5-6 → N3). `origa_ui/build.rs` copies it
//! verbatim into `well_known_sets_meta.json`.
//!
//! The earlier heuristic parsed the Section number from the set title. It
//! misread RU-series English titles ("Module 5 Section 16": the parser took
//! the unit number), which silently dropped 55 sets and mistagged 66 others
//! with the level of their sub-unit instead of their module. This audit now
//! guards both invariants directly:
//!
//! - completeness: every Duolingo content file is present in the meta;
//! - fidelity: the meta level equals the content file's own `level` field
//!   and is one of N5/N4/N3.
//!
//! Spy x Family content files all carry `level: "N3"` in their own metadata.
//!
//! The `cdn/` directory is gitignored. On a fresh clone without the content
//! files the tests **gracefully skip** (pass with a stderr note) rather than
//! panic, so `cargo test --workspace` stays green in CI environments that do
//! not have the CDN artifacts. CI seeds `cdn/` from production S3 and
//! `cargo build -p origa_ui` regenerates the meta deterministically from the
//! seeded content before the audit runs.
//!
//! Run: `cargo test -p origa --test well_known_sets_audit`
//! (after `cargo build -p origa_ui` to regenerate a fresh local meta).

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

fn cdn_dir() -> PathBuf {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by cargo");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("workspace root is parent of CARGO_MANIFEST_DIR")
        .join("cdn")
}

#[derive(Deserialize)]
struct SetMeta {
    id: String,
    set_type: String,
    level: String,
}

fn load_meta() -> Option<Vec<SetMeta>> {
    let path = cdn_dir()
        .join("well_known_set")
        .join("well_known_sets_meta.json");
    if !path.exists() {
        eprintln!(
            "[skip] well_known_sets_audit: {} is absent (cdn/ gitignored on fresh clones)",
            path.display()
        );
        return None;
    }
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let parsed: Vec<SetMeta> = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));
    Some(parsed)
}

/// Every Duolingo content file on disk, mirroring the `duolingo_<dir>_<stem>`
/// id scheme that `origa_ui/build.rs` generates. Returns `None` when the
/// content directory is absent (fresh clone without CDN seeding).
fn duolingo_content_levels() -> Option<Vec<(String, String)>> {
    let root = cdn_dir().join("well_known_set").join("duolingo");
    if !root.exists() {
        eprintln!(
            "[skip] well_known_sets_audit: {} is absent (cdn/ gitignored on fresh clones)",
            root.display()
        );
        return None;
    }
    let mut records = Vec::new();
    let mut subdirs: Vec<_> = fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", root.display()))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    subdirs.sort();
    for subdir in subdirs {
        let parent_name = subdir
            .file_name()
            .expect("subdir has a file name")
            .to_string_lossy()
            .to_string();
        let mut files: Vec<_> = fs::read_dir(&subdir)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", subdir.display()))
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .is_some_and(|ext| ext == "json")
            })
            .map(|e| e.path())
            .collect();
        files.sort();
        for path in files {
            let stem = path
                .file_stem()
                .expect("json file has a stem")
                .to_string_lossy()
                .to_string();
            let set_id = format!("duolingo_{}_{}", parent_name, stem);
            let raw = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
            let parsed: serde_json::Value = serde_json::from_str(&raw)
                .unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()));
            let level = parsed
                .get("level")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            records.push((set_id, level));
        }
    }
    Some(records)
}

const VALID_LEVELS: [&str; 3] = ["N5", "N4", "N3"];

#[test]
fn duolingo_sets_match_content_level_and_are_complete() {
    let (Some(records), Some(content)) = (load_meta(), duolingo_content_levels()) else {
        return;
    };
    let by_id: std::collections::HashMap<&str, &SetMeta> = records
        .iter()
        .filter(|r| r.set_type == "DuolingoEn" || r.set_type == "DuolingoRu")
        .map(|r| (r.id.as_str(), r))
        .collect();

    assert!(
        !by_id.is_empty(),
        "no Duolingo sets in well_known_sets_meta.json — fixture path or set_type values drifted"
    );

    let mut problems: Vec<String> = Vec::new();
    for (set_id, content_level) in &content {
        let Some(meta) = by_id.get(set_id.as_str()) else {
            problems.push(format!("[{set_id}] content file missing from meta (build.rs skip?)"));
            continue;
        };
        if !VALID_LEVELS.contains(&content_level.as_str()) {
            problems.push(format!(
                "[{set_id}] content level {content_level:?} is not one of {VALID_LEVELS:?}"
            ));
        }
        if meta.level != *content_level {
            problems.push(format!(
                "[{set_id}] meta level {:?} != content level {content_level:?}",
                meta.level
            ));
        }
    }

    assert!(
        !problems.is_empty() || !content.is_empty(),
        "audit checked no Duolingo content files — fixture path drifted"
    );
    assert!(
        problems.is_empty(),
        "Duolingo level audit failed ({} problems out of {} content files):\n{}",
        problems.len(),
        content.len(),
        problems.join("\n")
    );
}

#[test]
fn spy_family_sets_are_n3() {
    let Some(records) = load_meta() else {
        return;
    };
    let spy_records: Vec<&SetMeta> = records
        .iter()
        .filter(|r| r.set_type == "SpyFamily")
        .collect();

    assert!(
        !spy_records.is_empty(),
        "no SpyFamily sets found in well_known_sets_meta.json — fixture path drifted"
    );

    let wrong: Vec<&SetMeta> = spy_records
        .iter()
        .filter(|r| r.level != "N3")
        .copied()
        .collect();
    assert!(
        wrong.is_empty(),
        "Spy x Family sets must all be tagged N3 (matches content file level); these are wrong:\n{}",
        wrong
            .iter()
            .map(|r| format!("  [{}] level={}", r.id, r.level))
            .collect::<Vec<_>>()
            .join("\n")
    );
}
