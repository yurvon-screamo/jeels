use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "build_config.rs"]
mod build_config;

fn main() {
    handle_i18n();
    handle_lindera_dictionary();
    generate_well_known_meta();
    generate_font_registry();
    stage_fonts_for_bundling();
    println!("cargo:rerun-if-changed=build.rs");

    // Version info from CI environment
    let version = option_env!("ORIGA_VERSION").unwrap_or("dev");
    let commit = option_env!("ORIGA_COMMIT").unwrap_or("unknown");
    let build_date = option_env!("ORIGA_BUILD_DATE").unwrap_or("unknown");
    let public_base_url = option_env!("ORIGA_PUBLIC_BASE_URL").unwrap_or("");
    let cdn_base_url = build_config::resolve_cdn(env::var("ORIGA_CDN_BASE_URL").ok().as_deref());
    // http:// is allowed for local backends; resolve_cdn() returns DEFAULT_CDN
    // (https) for empty/unset, so http:// only arrives via explicit override.
    // See ADR-023 §3.
    assert!(
        cdn_base_url.starts_with("https://") || cdn_base_url.starts_with("http://"),
        "ORIGA_CDN_BASE_URL must be an http(s) URL, got: {cdn_base_url}"
    );
    let cdn_region = option_env!("ORIGA_CDN_REGION").unwrap_or("auto");
    let trailbase = build_config::resolve_trailbase(env::var("TRAILBASE_URL").ok().as_deref());

    println!("cargo:rustc-env=ORIGA_VERSION={}", version);
    println!("cargo:rustc-env=ORIGA_COMMIT={}", commit);
    println!("cargo:rustc-env=ORIGA_BUILD_DATE={}", build_date);
    println!("cargo:rustc-env=ORIGA_PUBLIC_BASE_URL={}", public_base_url);
    println!("cargo:rustc-env=ORIGA_CDN_BASE_URL={}", cdn_base_url);
    println!("cargo:rustc-env=ORIGA_CDN_REGION={}", cdn_region);
    println!("cargo:rustc-env=TRAILBASE_URL={}", trailbase);

    // Sentry env vars for the WASM frontend. Single `SENTRY_DSN` from CI is
    // emitted under the crate-scoped name `SENTRY_DSN_UI` (distinguishes from
    // the tauri-native layer in the same Sentry project via the `layer` tag).
    // Empty/unset = Sentry disabled at runtime. `SENTRY_RELEASE_UI` is derived
    // from `ORIGA_VERSION` to guarantee release-name sync with the native
    // layer (ADR-036 §3). See ADR-024 for the env-var read pattern
    // (`env::var()` + `cargo:rustc-env` + `rerun-if-env-changed` instead of
    // `option_env!`, which captures at build-script compile time).
    let sentry_dsn = env::var("SENTRY_DSN").unwrap_or_default();
    let sentry_environment =
        env::var("SENTRY_ENVIRONMENT").unwrap_or_else(|_| "development".into());
    let sentry_release = env::var("ORIGA_VERSION").unwrap_or_else(|_| "dev".into());

    println!("cargo:rustc-env=SENTRY_DSN_UI={sentry_dsn}");
    println!("cargo:rustc-env=SENTRY_ENVIRONMENT_UI={sentry_environment}");
    println!("cargo:rustc-env=SENTRY_RELEASE_UI={sentry_release}");

    println!("cargo:rerun-if-env-changed=ORIGA_VERSION");
    println!("cargo:rerun-if-env-changed=ORIGA_COMMIT");
    println!("cargo:rerun-if-env-changed=ORIGA_BUILD_DATE");
    println!("cargo:rerun-if-env-changed=ORIGA_PUBLIC_BASE_URL");
    println!("cargo:rerun-if-env-changed=ORIGA_CDN_BASE_URL");
    println!("cargo:rerun-if-env-changed=ORIGA_CDN_REGION");
    println!("cargo:rerun-if-env-changed=TRAILBASE_URL");
    println!("cargo:rerun-if-env-changed=SENTRY_DSN");
    println!("cargo:rerun-if-env-changed=SENTRY_ENVIRONMENT");
    println!("cargo:rerun-if-changed=build_config.rs");
    println!("cargo:rerun-if-changed=../build_defaults.rs");
}

fn handle_i18n() {
    println!("cargo:rerun-if-changed=locales/en.json");
    println!("cargo:rerun-if-changed=locales/ru.json");

    let i18n_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("i18n");

    let cfg = leptos_i18n_build::Config::new("en")
        .expect("Failed to create i18n config")
        .add_locale("ru")
        .expect("Failed to add locale");

    let infos = leptos_i18n_build::TranslationsInfos::parse(cfg)
        .expect("Failed to parse i18n translations");

    infos.emit_diagnostics();
    infos.rerun_if_locales_changed();
    infos
        .generate_i18n_module(i18n_dir)
        .expect("Failed to generate i18n module");
}

fn handle_lindera_dictionary() {
    // The tokenizer dictionary ships as pre-built deflate files in
    // cdn/dictionaries/sudachidict-20260723/ (SudachiDict compiled with the
    // lindera 5.x builder; the lindera 6.x runtime reads the same format —
    // see docs/src/sudachidict.md in the lindera repo).
    // Nothing to build at compile time; verify the files exist so a broken
    // checkout fails fast.
    //
    // The directory name mirrors `origa::domain::tokenizer::SUDACHIDICT_DIR`
    // (a build script cannot import from the crate itself) and the paths in
    // scripts/deploy_cdn.py — bump all three together.
    const SUDACHIDICT_DIR: &str = "sudachidict-20260723";
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let public_dict_dir = Path::new(&manifest_dir)
        .join("..")
        .join("cdn")
        .join("dictionaries")
        .join(SUDACHIDICT_DIR);

    let required = [
        "char_def.bin",
        "matrix.mtx",
        "dict.trie",
        "dict.valsidx",
        "dict.vals",
        "unk.bin",
        "dict.wordsidx",
        "dict.words",
        "metadata.json",
    ];

    for file in required {
        if !public_dict_dir.join(file).exists() {
            println!(
                "cargo:warning=Dictionary file missing: {} (cdn/ is gitignored; restore via scripts/deploy_cdn.py)",
                public_dict_dir.join(file).display()
            );
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[allow(non_snake_case)]
struct WellKnownContent {
    #[serde(default)]
    Russian: Option<LanguageContent>,
    #[serde(default)]
    English: Option<LanguageContent>,
}

#[derive(Debug, Deserialize)]
struct LanguageContent {
    #[serde(default)]
    title: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, Deserialize)]
struct WellKnownSet {
    #[serde(default)]
    content: WellKnownContent,
    #[serde(default)]
    words: Vec<serde_json::Value>,
    #[serde(default)]
    level: String,
}

#[derive(Debug, Serialize)]
struct WellKnownMeta {
    id: String,
    set_type: String,
    level: String,
    title_ru: String,
    title_en: String,
    desc_ru: String,
    desc_en: String,
    word_count: usize,
}

fn extract_meta(
    data: &mut WellKnownSet,
    set_id: &str,
    set_type: &str,
    level: &str,
) -> WellKnownMeta {
    data.level = level.to_string();

    let (title_ru, desc_ru) = data
        .content
        .Russian
        .as_ref()
        .map(|r| (r.title.clone(), r.description.clone()))
        .unwrap_or_default();

    let (title_en, desc_en) = data
        .content
        .English
        .as_ref()
        .map(|e| (e.title.clone(), e.description.clone()))
        .unwrap_or_default();

    WellKnownMeta {
        id: set_id.to_string(),
        set_type: set_type.to_string(),
        level: level.to_string(),
        title_ru,
        title_en,
        desc_ru,
        desc_en,
        word_count: data.words.len(),
    }
}

fn load_json(path: &Path) -> WellKnownSet {
    let content =
        fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|_| panic!("Failed to parse JSON from {}", path.display()))
}

fn generate_well_known_meta() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let base_dir = Path::new(&manifest_dir)
        .join("..")
        .join("cdn")
        .join("well_known_set");
    let output_file = base_dir.join("well_known_sets_meta.json");

    if !base_dir.exists() {
        return;
    }

    // Rerun if any JSON file in well_known_set changes
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("jlpt_n5.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("jlpt_n4.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("jlpt_n3.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("jlpt_n2.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("jlpt_n1.json").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("migii").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("minna_n5").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("minna_n4").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("minna_n3").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("minna_n2").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("spy_family").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("duolingo").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("irodori_nyuumon").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("irodori_shokyuu1").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        base_dir.join("irodori_shokyuu2").display()
    );

    let mut meta_list: Vec<WellKnownMeta> = Vec::new();

    // JLPT files
    let jlpt_files = [
        ("jlpt_n5.json", "jlpt_n5", "N5"),
        ("jlpt_n4.json", "jlpt_n4", "N4"),
        ("jlpt_n3.json", "jlpt_n3", "N3"),
        ("jlpt_n2.json", "jlpt_n2", "N2"),
        ("jlpt_n1.json", "jlpt_n1", "N1"),
    ];

    for (filename, set_id, level) in &jlpt_files {
        let path = base_dir.join(filename);
        if path.exists() {
            let mut data = load_json(&path);
            meta_list.push(extract_meta(&mut data, set_id, "Jlpt", level));
        }
    }

    // Migii files
    let migii_configs = [
        ("n5", 20, "N5"),
        ("n4", 11, "N4"),
        ("n3", 31, "N3"),
        ("n2", 31, "N2"),
        ("n1", 56, "N1"),
    ];

    for (level_id, count, level) in &migii_configs {
        for i in 1..=*count {
            let filename = format!("migii/{}/migii_{}_{}.json", level_id, level_id, i);
            let path = base_dir.join(&filename);
            if path.exists() {
                let mut data = load_json(&path);
                let set_id = format!("migii_{}_{}", level_id, i);
                meta_list.push(extract_meta(&mut data, &set_id, "Migii", level));
            }
        }
    }

    // Minna N5 files
    let minna_n5_dir = base_dir.join("minna_n5");
    if minna_n5_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&minna_n5_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with("minna_n5_"))
                .unwrap_or(false)
            {
                let mut data = load_json(&path);
                let set_id = path.file_stem().unwrap().to_string_lossy().to_string();
                meta_list.push(extract_meta(&mut data, &set_id, "MinnaNoNihongo", "N5"));
            }
        }
    }

    // Minna N4 files
    let minna_n4_dir = base_dir.join("minna_n4");
    if minna_n4_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&minna_n4_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with("minna_n4_"))
                .unwrap_or(false)
            {
                let mut data = load_json(&path);
                let set_id = path.file_stem().unwrap().to_string_lossy().to_string();
                meta_list.push(extract_meta(&mut data, &set_id, "MinnaNoNihongo", "N4"));
            }
        }
    }

    // Minna N3 files
    let minna_n3_dir = base_dir.join("minna_n3");
    if minna_n3_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&minna_n3_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with("minna_n3_"))
                .unwrap_or(false)
            {
                let mut data = load_json(&path);
                let set_id = path.file_stem().unwrap().to_string_lossy().to_string();
                meta_list.push(extract_meta(&mut data, &set_id, "MinnaNoNihongo", "N3"));
            }
        }
    }

    // Minna N2 files
    let minna_n2_dir = base_dir.join("minna_n2");
    if minna_n2_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&minna_n2_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with("minna_n2_"))
                .unwrap_or(false)
            {
                let mut data = load_json(&path);
                let set_id = path.file_stem().unwrap().to_string_lossy().to_string();
                meta_list.push(extract_meta(&mut data, &set_id, "MinnaNoNihongo", "N2"));
            }
        }
    }

    // Irodori Nyuumon files
    let irodori_nyuumon_dir = base_dir.join("irodori_nyuumon");
    if irodori_nyuumon_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&irodori_nyuumon_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with("irodori_nyuumon_"))
                .unwrap_or(false)
            {
                let mut data = load_json(&path);
                let set_id = path.file_stem().unwrap().to_string_lossy().to_string();
                meta_list.push(extract_meta(&mut data, &set_id, "Irodori", "N5"));
            }
        }
    }

    // Irodori Shokyuu1 files
    let irodori_shokyuu1_dir = base_dir.join("irodori_shokyuu1");
    if irodori_shokyuu1_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&irodori_shokyuu1_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with("irodori_shokyuu1_"))
                .unwrap_or(false)
            {
                let mut data = load_json(&path);
                let set_id = path.file_stem().unwrap().to_string_lossy().to_string();
                meta_list.push(extract_meta(&mut data, &set_id, "Irodori", "N4"));
            }
        }
    }

    // Irodori Shokyuu2 files
    let irodori_shokyuu2_dir = base_dir.join("irodori_shokyuu2");
    if irodori_shokyuu2_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&irodori_shokyuu2_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with("irodori_shokyuu2_"))
                .unwrap_or(false)
            {
                let mut data = load_json(&path);
                let set_id = path.file_stem().unwrap().to_string_lossy().to_string();
                meta_list.push(extract_meta(&mut data, &set_id, "Irodori", "N4"));
            }
        }
    }

    // Spy Family files
    let spy_family_dir = base_dir.join("spy_family");
    if spy_family_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&spy_family_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            let mut data = load_json(&path);
            let set_id = path.file_stem().unwrap().to_string_lossy().to_string();
            meta_list.push(extract_meta(&mut data, &set_id, "SpyFamily", "N3"));
        }
    }

    // Duolingo files
    let duolingo_dir = base_dir.join("duolingo");
    if duolingo_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(&duolingo_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for subdir_entry in entries {
            let subdir = subdir_entry.path();
            let mut json_entries: Vec<_> = fs::read_dir(&subdir)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "json")
                        .unwrap_or(false)
                })
                .collect();
            json_entries.sort_by_key(|e| e.file_name());

            for json_entry in json_entries {
                let path = json_entry.path();
                let stem = path.file_stem().unwrap().to_string_lossy();
                let parent_name = path
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy();

                let mut data = load_json(&path);

                // Level comes from the content file's own `level` field
                // (corpus ground truth: Section/Модуль 1-3 → N5, 4 → N4,
                // 5-6 → N3). Title parsing was removed: RU-series English
                // titles carry both tokens ("Module 5 Section 16"), so the
                // heuristic read the sub-unit number — 55 sets were dropped
                // and 66 tagged with their unit's level instead of their
                // module's.
                let level = data.level.trim().to_string();
                if !matches!(level.as_str(), "N5" | "N4" | "N3") {
                    eprintln!(
                        "cargo:warning=Skipping duolingo set: invalid level {:?} for duolingo/{parent_name}/{stem}",
                        level
                    );
                    continue;
                }

                let set_id = format!("duolingo_{}_{}", parent_name, stem);
                let set_type = if stem.contains("_en_") {
                    "DuolingoEn"
                } else {
                    "DuolingoRu"
                };

                meta_list.push(extract_meta(&mut data, &set_id, set_type, &level));
            }
        }
    }

    // Sort by ID
    meta_list.sort_by(|a, b| a.id.cmp(&b.id));

    // Write output
    let output = serde_json::to_string_pretty(&meta_list).expect("Failed to serialize meta list");
    fs::write(&output_file, output).expect("Failed to write output file");

    println!(
        "cargo:warning=Generated {} meta entries to {}",
        meta_list.len(),
        output_file.display()
    );
}

/// Emit `OUT_DIR/fonts.rs` enumerating the subset woff2 files in `cdn/fonts/`.
///
/// Each file is named `<logical>-<sha8>.woff2` (content hash, see
/// `scripts/subset_fonts.py`). The hash suffix is the CDN cache-bust key: a
/// regenerated subset yields a new hash → new URL, so immutable edge caches
/// can never serve a stale font (PR #182 class).
///
/// The generated module exposes `FONT_FILES: &[(&str, &str)]` mapping the
/// logical name (used by `font_face_css`) to the on-CDN filename. When
/// `cdn/fonts/` is absent the array is empty, so the app degrades gracefully
/// (no @font-face injected, system fallback) instead of failing to build.
fn generate_font_registry() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let fonts_dir = Path::new(&manifest_dir)
        .join("..")
        .join("cdn")
        .join("fonts");
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_file = Path::new(&out_dir).join("fonts.rs");

    println!("cargo:rerun-if-changed={}", fonts_dir.display());

    let mut entries: Vec<(String, String)> = Vec::new();
    if fonts_dir.is_dir() {
        let mut names: Vec<_> = fs::read_dir(&fonts_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().to_str().map(str::to_owned))
            .filter(|n| n.ends_with(".woff2"))
            .collect();
        names.sort();
        for filename in names {
            let logical = strip_hash_suffix(&filename);
            entries.push((logical, filename));
        }
    }

    let mut src = String::from("pub static FONT_FILES: &[(&str, &str)] = &[\n");
    for (logical, filename) in &entries {
        src.push_str(&format!("    (\"{logical}\", \"{filename}\"),\n"));
    }
    src.push_str("];\n");
    fs::write(&out_file, src).expect("Failed to write fonts.rs");
    println!(
        "cargo:warning=Registered {} font file(s) to {}",
        entries.len(),
        out_file.display()
    );
}

/// Reduce `<logical>-<sha8>.woff2` to `<logical>` by stripping the trailing
/// 8-hex-char content hash. Falls back to the full stem if the suffix does not
/// match the expected hash shape (defensive: malformed filenames stay visible).
fn strip_hash_suffix(filename: &str) -> String {
    let stem = filename.trim_end_matches(".woff2");
    match stem.rsplit_once('-') {
        Some((head, tail)) if tail.len() == 8 && tail.chars().all(|c| c.is_ascii_hexdigit()) => {
            head.to_owned()
        },
        _ => stem.to_owned(),
    }
}

/// Copy the canonical `cdn/fonts/*.woff2` into `origa_ui/public/fonts/` so the
/// existing trunk `copy-dir href="public"` serves them at `/fonts/` on the
/// local origin (Tauri builds, first-launch offline CJK). trunk does not accept
/// assets outside the `index.html` directory (trunk-rs/trunk#1045), so we stage
/// the canonical files inside `public/` first. `public/fonts/` is gitignored —
/// `cdn/fonts/` remains the single source of truth, mirrored verbatim here.
///
/// Idempotent: re-runs only when `cdn/fonts/` changes (`rerun-if-changed`), and
/// only rewrites a target file when its bytes differ from the source, so
/// incremental trunk rebuilds are not invalidated by no-op copies.
fn stage_fonts_for_bundling() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = Path::new(&manifest_dir)
        .join("..")
        .join("cdn")
        .join("fonts");
    let dest_dir = Path::new(&manifest_dir).join("public").join("fonts");

    println!("cargo:rerun-if-changed={}", src_dir.display());

    if !src_dir.is_dir() {
        // cdn/fonts/ absent (e.g. fresh checkout without CDN assets): font_face
        // degrades to FONT_FILES empty -> system fallback. Do not fail the build.
        return;
    }

    fs::create_dir_all(&dest_dir).expect("Failed to create public/fonts/");

    let mut staged: Vec<String> = Vec::new();
    for entry in fs::read_dir(&src_dir)
        .unwrap_or_else(|_| panic!("Failed to read {}", src_dir.display()))
        .flatten()
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("woff2") {
            continue;
        }
        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_owned(),
            None => continue,
        };
        let dest = dest_dir.join(&filename);

        let copy_needed = match fs::read(&path) {
            Ok(src_bytes) => match fs::read(&dest) {
                Ok(dest_bytes) => src_bytes != dest_bytes,
                Err(_) => true,
            },
            Err(e) => panic!("Failed to read source font {}: {e}", path.display()),
        };
        if copy_needed {
            fs::copy(&path, &dest)
                .unwrap_or_else(|_| panic!("Failed to stage font {filename} into public/fonts/"));
        }
        staged.push(filename);
    }

    // Prune stale woff2 in dest that no longer exist in the canonical source
    // (keeps public/fonts/ an exact mirror, avoids shipping removed subsets).
    if dest_dir.is_dir() {
        for entry in fs::read_dir(&dest_dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("woff2") {
                continue;
            }
            let still_canonical = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| staged.iter().any(|s| s == name));
            if !still_canonical {
                let _ = fs::remove_file(&path);
            }
        }
    }

    println!(
        "cargo:warning=Staged {} bundled font(s) into public/fonts/",
        staged.len()
    );
}
