//! Build-time integrity checks for the generated `public/sitemap.xml`.
//!
//! `build.rs::generate_sitemap` renders `public/sitemap.xml.tmpl`,
//! substituting `{{LASTMOD}}` with the build date. These tests read the
//! generated file (produced at compile time, before tests run) and assert the
//! sitemaps.org 0.9 contract holds:
//!   - one `<lastmod>` per `<url>` (20 URLs: 5 pages × 4 locales),
//!   - each `<lastmod>` is an ISO-8601 date,
//!   - `<lastmod>` follows `<loc>` (the schema-required element order),
//!   - no unresolved `{{...}}` placeholder leaks into the output.
//!
//! The HTTP cache policy for `/sitemap.xml` (`no-cache`) is covered by
//! `cache_headers::sitemap_xml_has_no_cache`; it is not re-asserted here to
//! avoid duplication.

#![cfg(feature = "ssr")]

use std::path::PathBuf;

fn sitemap_contents() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("public/sitemap.xml");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("generated sitemap not found at {}: {e}", path.display()))
}

/// Collect every `<lastmod>VALUE</lastmod>` payload from `xml`, in document
/// order. Used by both the count and the format assertions.
fn lastmod_values(xml: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut rest = xml;
    while let Some(open) = rest.find("<lastmod>") {
        let after_open = &rest[open + "<lastmod>".len()..];
        let close = after_open
            .find("</lastmod>")
            .unwrap_or_else(|| panic!("unterminated <lastmod> in sitemap: {after_open}"));
        values.push(after_open[..close].to_string());
        rest = &after_open[close + "</lastmod>".len()..];
    }
    values
}

/// Validate `YYYY-MM-DD` without pulling in a regex dependency. sitemaps.org
/// expects the full date form (no time portion) for `<lastmod>`.
fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[0..4].iter().all(|b| b.is_ascii_digit())
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[8..10].iter().all(|b| b.is_ascii_digit())
}

#[test]
fn lastmod_appears_once_per_url() {
    // Each <url> element carries exactly one <lastmod>. As of 2026-07-23 the
    // count is: 7 static page groups × 4 locales (28) + 4 blog index URLs +
    // 7 articles × 4 locales (28) + docs (2 index + 9 articles × 2 locales = 20)
    // = 80 <url> entries. The count assertion catches drift in either
    // direction — a missing locale variant or a duplicate entry.
    let values = lastmod_values(&sitemap_contents());
    assert_eq!(values.len(), 80, "expected one <lastmod> per <url>");
}

#[test]
fn every_url_block_has_full_hreflang_alternate_set() {
    // Every URL carries the full hreflang set for its locale coverage:
    // - Blog and static pages: 5 entries (en/ru/ko/vi + x-default) — all 4
    //   locales are translated.
    // - Docs pages: 3 entries (en/ru + x-default) — docs ship in EN and RU
    //   only; KO/VI serve EN content with noindex and are not in the sitemap.
    let xml = sitemap_contents();
    for block in xml.split("<url>").skip(1) {
        let url_block = block.split("</url>").next().unwrap_or(block);
        let alternate_count = url_block.matches("<xhtml:link rel=\"alternate\"").count();
        let expected = if url_block.contains("/docs") { 3 } else { 5 };
        assert_eq!(
            alternate_count, expected,
            "expected {expected} hreflang alternates for this URL; got {alternate_count} in:\n{url_block}"
        );
    }
}

#[test]
fn no_unresolved_template_placeholders() {
    let xml = sitemap_contents();
    assert!(
        !xml.contains("{{"),
        "unresolved template placeholder in sitemap: {}",
        &xml[..xml.len().min(400)]
    );
}

#[test]
fn every_lastmod_is_an_iso_date() {
    let values = lastmod_values(&sitemap_contents());
    assert!(
        !values.is_empty(),
        "precondition: sitemap has <lastmod> entries"
    );
    for value in &values {
        assert!(
            is_iso_date(value),
            "<lastmod> must be YYYY-MM-DD, got {value:?}"
        );
    }
}

#[test]
fn lastmod_follows_loc_in_every_url() {
    // sitemaps.org 0.9 allows child elements of <url> in any order, but the
    // Google/Bing/Yandex crawlers we target all parse <loc> first; placing
    // <lastmod> immediately after <loc> (before hreflang alternates) is the
    // documented convention. This guards against a template edit that swaps
    // the order.
    let xml = sitemap_contents();
    for block in xml.split("<url>").skip(1) {
        let url_block = block.split("</url>").next().unwrap_or(block);
        let loc = url_block.find("<loc>");
        let lastmod = url_block.find("<lastmod>");
        match (loc, lastmod) {
            (Some(loc_idx), Some(lastmod_idx)) => {
                assert!(
                    loc_idx < lastmod_idx,
                    "<loc> must precede <lastmod> in: {url_block}"
                );
            },
            _ => panic!("<url> block missing <loc>/<lastmod>: {url_block}"),
        }
    }
}
