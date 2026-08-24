#[path = "build_config.rs"]
mod build_config;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let landing_base_url = build_config::resolve_env(
        std::env::var("ORIGA_LANDING_BASE_URL").ok().as_deref(),
        build_config::DEFAULT_LANDING,
    );
    println!("cargo:rustc-env=ORIGA_LANDING_BASE_URL={landing_base_url}");
    println!("cargo:rerun-if-env-changed=ORIGA_LANDING_BASE_URL");
    println!("cargo:rerun-if-changed=build_config.rs");

    let app_base_url = std::env::var("ORIGA_APP_BASE_URL")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| {
            let base_uri = std::env::var("ORIGA_BASE_URI").ok();
            let app_prefix = std::env::var("ORIGA_APP_URI_PREFIX").ok();
            match (app_prefix, base_uri) {
                (Some(prefix), Some(base)) => format!("https://{prefix}.{base}"),
                _ => panic!(
                    "ORIGA_APP_BASE_URL or (ORIGA_APP_URI_PREFIX + ORIGA_BASE_URI) env vars must be set"
                ),
            }
        });
    println!("cargo:rustc-env=ORIGA_APP_BASE_URL={app_base_url}");

    // Optional Yandex.Metrika counter ID (behavioral analytics for the RU
    // market). Empty/absent = the counter is not emitted at all — mirroring
    // the SENTRY_DSN gating contract (ADR-036): no ID, zero footprint.
    let metrika_id = std::env::var("ORIGA_YANDEX_METRIKA_ID").unwrap_or_default();
    println!("cargo:rustc-env=ORIGA_YANDEX_METRIKA_ID={metrika_id}");
    println!("cargo:rerun-if-env-changed=ORIGA_YANDEX_METRIKA_ID");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    build_css();
    generate_sitemap(&manifest_dir, &landing_base_url);
}

fn build_css() {
    println!("cargo:rerun-if-changed=style/landing.css");
    println!("cargo:rerun-if-changed=style/input.css");
    println!("cargo:rerun-if-changed=tailwind.config.js");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let base = std::path::Path::new(&manifest_dir);
    let input = base.join("style/landing.css");
    let input_extra = base.join("style/input.css");
    let output = base.join("style/landing.processed.css");

    let output_mtime = output.metadata().ok().and_then(|m| m.modified().ok());

    let skip = output.exists()
        && output_mtime.is_some_and(|out_time| {
            let main_fresh = input
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .is_some_and(|t| out_time >= t);

            let extra_fresh = input_extra
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .is_some_and(|t| out_time >= t);

            main_fresh && extra_fresh
        });

    if skip {
        return;
    }

    let result = if cfg!(windows) {
        std::process::Command::new("cmd")
            .args([
                "/C",
                "npx",
                "tailwindcss",
                "--input",
                input.to_str().unwrap(),
                "--output",
                output.to_str().unwrap(),
                "--minify",
            ])
            .current_dir(&manifest_dir)
            .status()
    } else {
        std::process::Command::new("npx")
            .args([
                "tailwindcss",
                "--input",
                input.to_str().unwrap(),
                "--output",
                output.to_str().unwrap(),
                "--minify",
            ])
            .current_dir(&manifest_dir)
            .status()
    };

    match result {
        Ok(s) if s.success() => {},
        Ok(s) => panic!("CSS build failed: npx tailwindcss exited with {s}"),
        Err(e) => {
            println!("cargo:warning=npx not available, skipping CSS rebuild: {e}");
        },
    }
}

/// Render `public/sitemap.xml` from `public/sitemap.xml.tmpl`.
///
/// Per-URL freshness contract (2026-08): **article URLs take their `<lastmod>`
/// from the article's own frontmatter**, so a re-deploy no longer stamps every
/// URL as "changed yesterday" — a uniform fresh lastmod teaches crawlers to
/// distrust the field entirely. Static pages and locale index pages keep the
/// release-date fallback chain below, because they have no per-page source of
/// truth available inside the Docker builder (the `.git` directory is too
/// large to copy into the image just for dates).
///
/// Fallback precedence for non-article URLs: `ORIGA_BUILD_DATE` (set by CI,
/// e.g. `docker.yml`) → last commit date of the template (`git log`) →
/// `1970-01-01` sentinel with a warning. The sentinel keeps local builds green
/// in environments without git history; CI always supplies `ORIGA_BUILD_DATE`.
fn generate_sitemap(manifest_dir: &str, landing_base_url: &str) {
    println!("cargo:rerun-if-changed=public/sitemap.xml.tmpl");
    println!("cargo:rerun-if-env-changed=ORIGA_BUILD_DATE");
    println!("cargo:rerun-if-changed=content");

    let base = std::path::Path::new(manifest_dir);
    let tmpl_path = base.join("public/sitemap.xml.tmpl");
    let out_path = base.join("public/sitemap.xml");

    let tmpl = match std::fs::read_to_string(&tmpl_path) {
        Ok(s) => s,
        Err(e) => panic!("failed to read {}: {e}", tmpl_path.display()),
    };

    let default_lastmod = std::env::var("ORIGA_BUILD_DATE")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| git_lastmod(manifest_dir))
        .unwrap_or_else(|| {
            println!(
                "cargo:warning=sitemap lastmod: no ORIGA_BUILD_DATE and no git, using 1970-01-01"
            );
            "1970-01-01".to_string()
        });

    let article_dates = collect_article_lastmods(manifest_dir);
    let rendered = tmpl.replace("{{LASTMOD}}", "{{LASTMOD_PENDING}}");
    let final_xml = apply_per_url_lastmod(
        &rendered,
        landing_base_url,
        &default_lastmod,
        &article_dates,
    );

    std::fs::write(&out_path, final_xml)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", out_path.display()));
}

/// Map every article's sitemap URL path to its frontmatter `lastmod` date.
///
/// Scans `content/blog/<locale>/<slug>.md` and `content/docs/<locale>/<slug>.md`
/// (plus each docs `index.md`) for a `lastmod:` line inside the YAML header.
/// Files without a parsable `lastmod:` are silently skipped — they fall back
/// to the default date rather than failing the build over a content nit.
fn collect_article_lastmods(manifest_dir: &str) -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    let base = std::path::Path::new(manifest_dir).join("content");
    let locale_prefixes: &[(&str, &str)] =
        &[("en", ""), ("ru", "/ru"), ("ko", "/ko"), ("vi", "/vi")];
    for dir in ["blog", "docs"] {
        for (code, prefix) in locale_prefixes {
            let dir_path = base.join(dir).join(code);
            let entries = match std::fs::read_dir(&dir_path) {
                Ok(e) => e,
                Err(_) => continue,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }
                let stem = match path.file_stem().and_then(|s| s.to_str()) {
                    Some(s) => s,
                    None => continue,
                };
                let Some(date) = frontmatter_lastmod(&path) else {
                    continue;
                };
                let slug_path = if stem == "index" && dir == "docs" {
                    format!("{prefix}/{dir}")
                } else {
                    format!("{prefix}/{dir}/{stem}")
                };
                map.insert(slug_path, date);
            }
        }
    }
    map
}

/// Read the `lastmod:` value from a markdown file's frontmatter block. Manual
/// line scan instead of a YAML crate: one fixed field, zero dependencies.
fn frontmatter_lastmod(path: &std::path::Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut closed = false;
    for line in text.lines() {
        let trimmed = line.trim_end();
        if trimmed == "---" {
            if closed {
                return None;
            }
            closed = true;
            continue;
        }
        if !closed {
            continue;
        }
        if trimmed == "---" {
            return None;
        }
        if let Some(value) = trimmed.strip_prefix("lastmod:") {
            let date = value.trim().trim_matches('"');
            if date.len() == 10 && date.as_bytes()[4] == b'-' {
                return Some(date.to_string());
            }
            return None;
        }
    }
    None
}

/// Replace each pending `<lastmod>{{LASTMOD_PENDING}}</lastmod>` that follows a
/// `<loc>` with the date mapped from that URL. URLs absent from the article
/// map keep `default_date`. Panics on structural template damage (unclosed
/// `<loc>`/`<lastmod>`) — a broken sitemap must fail the build loudly.
fn apply_per_url_lastmod(
    rendered: &str,
    landing_base_url: &str,
    default_date: &str,
    article_dates: &std::collections::HashMap<String, String>,
) -> String {
    const LOC_OPEN: &str = "<loc>";
    const LOC_CLOSE: &str = "</loc>";
    const LM_OPEN: &str = "<lastmod>";
    const LM_CLOSE: &str = "</lastmod>";
    const PENDING: &str = "<lastmod>{{LASTMOD_PENDING}}</lastmod>";

    let mut out = String::with_capacity(rendered.len());
    let mut rest = rendered;
    while let Some(loc_idx) = rest.find(LOC_OPEN) {
        let after_loc = &rest[loc_idx + LOC_OPEN.len()..];
        let url_end = after_loc
            .find(LOC_CLOSE)
            .unwrap_or_else(|| panic!("sitemap template: unclosed <loc> near {after_loc}"));
        let url = &after_loc[..url_end];

        let tail = &after_loc[url_end + LOC_CLOSE.len()..];
        let lm_idx = tail
            .find(PENDING)
            .unwrap_or_else(|| panic!("sitemap template: <loc> without pending <lastmod>: {url}"));

        out.push_str(&rest[..loc_idx + LOC_OPEN.len()]);
        out.push_str(url);
        out.push_str(LOC_CLOSE);
        out.push_str(&tail[..lm_idx]);

        let path = url.strip_prefix(landing_base_url).unwrap_or(url);
        let date = article_dates.get(path).map_or(default_date, |d| d);
        out.push_str(LM_OPEN);
        out.push_str(date);
        out.push_str(LM_CLOSE);

        rest = &tail[lm_idx + PENDING.len()..];
    }
    out.push_str(rest);
    out
}

/// Best-effort last-modified date of the sitemap template, as an ISO-8601
/// `YYYY-MM-DD` string. Returns `None` when git is unavailable (no `.git`,
/// git not on PATH) so the caller can fall back to the sentinel.
///
/// `--follow` spans the `sitemap.xml -> sitemap.xml.tmpl` rename so a fresh
/// checkout still resolves a date from the template's pre-rename history.
fn git_lastmod(manifest_dir: &str) -> Option<String> {
    let output = std::process::Command::new("git")
        .args([
            "log",
            "-1",
            "--format=%cd",
            "--date=short",
            "--follow",
            "--",
            "public/sitemap.xml.tmpl",
        ])
        .current_dir(manifest_dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let date = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if date.is_empty() { None } else { Some(date) }
}
