//! Integration tests for `/blog` index and `/blog/<slug>` article pages.
//!
//! Mirrors the convention in `tests/seo_meta.rs`: render the production
//! router via `tower::ServiceExt::oneshot`, parse the resulting HTML with
//! plain string assertions. Coverage spans all 4 locales (EN/RU/KO/VI) for
//! every published article, plus the index page and the legacy-slug 308
//! redirect.

#![cfg(feature = "ssr")]

use http::StatusCode;

use common::get;

mod common;

// =========================================================================
// Published article slugs
// =========================================================================

/// Canonical list of every published blog slug. Slugs with `en`/`ru` entries
/// only (the 2026-08 cluster) are covered by the same tests as full-coverage
/// slugs except where noted: hreflang expectations differ (see
/// `PARTIAL_COVERAGE_SLUGS`) and script-consistency checks skip EN-fallback
/// renders. Locale-specific variants live in `content/blog/<locale>/<slug>.md`.
/// The const is shared by every script-consistency test and the
/// parameterized JSON-LD/hreflang tests so that adding a new article is a
/// one-line edit (append here + register in `ARTICLES`) instead of a
/// five-file sweep.
const ALL_SLUGS: &[&str] = &[
    "anki-alternative-japanese",
    "best-japanese-learning-app",
    "learn-japanese-from-manga",
    "japanese-ocr-app",
    "best-japanese-learning-app-offline",
    "japanese-ai-tutor",
    "yaponskiy-s-nulya",
    "learn-hiragana-katakana",
    "jlpt-n5-preparation",
    "japanese-textbooks-beginners",
    "how-many-kanji-to-learn",
    "learn-japanese-from-anime",
];

/// Slugs published in EN + RU only (no KO/VI translations yet). Their pages
/// advertise a 3-entry hreflang set {en, ru, x-default} instead of the full
/// 5-entry set, and `/ko|vi/blog/<slug>` serves the EN article as a fallback.
const PARTIAL_COVERAGE_SLUGS: &[&str] = &[
    "learn-hiragana-katakana",
    "jlpt-n5-preparation",
    "japanese-textbooks-beginners",
    "how-many-kanji-to-learn",
    "learn-japanese-from-anime",
];

// =========================================================================
// Article pages — native rendering across all 4 locales
// =========================================================================

#[tokio::test]
async fn en_article_returns_200_with_h1() {
    let (status, body) = get("/blog/anki-alternative-japanese").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains(r#"<article class="blog-post">"#),
        "EN article must render inside .blog-post; got first 600 chars: {}",
        body.chars().take(600).collect::<String>()
    );
    assert!(
        body.contains("Anki Alternative for Japanese"),
        "EN article must contain its title"
    );
}

#[tokio::test]
async fn ru_article_returns_200_with_localized_h1() {
    let (status, body) = get("/ru/blog/best-japanese-learning-app").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains(r#"<article class="blog-post">"#),
        "RU article must render inside .blog-post; got first 600 chars: {}",
        body.chars().take(600).collect::<String>()
    );
    assert!(
        body.contains("лучшее приложение"),
        "RU article must contain its Russian title"
    );
}

#[tokio::test]
async fn ko_article_returns_200_with_localized_h1() {
    let (status, body) = get("/ko/blog/anki-alternative-japanese").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains(r#"<article class="blog-post">"#),
        "KO article must render inside .blog-post; got first 600 chars: {}",
        body.chars().take(600).collect::<String>()
    );
    assert!(
        body.contains("Anki 대안"),
        "KO article must contain its Korean title fragment; got first 800 chars: {}",
        body.chars().take(800).collect::<String>()
    );
}

#[tokio::test]
async fn vi_article_returns_200_with_localized_h1() {
    let (status, body) = get("/vi/blog/best-japanese-learning-app").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains(r#"<article class="blog-post">"#),
        "VI article must render inside .blog-post"
    );
    assert!(
        body.contains("ứng dụng học tiếng Nhật"),
        "VI article must contain its Vietnamese title fragment; got first 800 chars: {}",
        body.chars().take(800).collect::<String>()
    );
}

#[tokio::test]
async fn all_article_urls_return_200() {
    // All translations of every published article must be reachable in their
    // native locale. A failure here usually means a markdown file is missing
    // or the ARTICLES const in registry.rs is out of sync with the files on
    // disk. The slug list is sourced from `ALL_SLUGS` so adding a new article
    // does not require touching this test.
    for slug in ALL_SLUGS {
        for prefix in ["", "/ru", "/ko", "/vi"] {
            let uri = format!("{prefix}/blog/{slug}");
            let (status, _) = get(&uri).await;
            assert_eq!(status, StatusCode::OK, "expected 200 for {uri}");
        }
    }
}

// =========================================================================
// 404 / unknown slug
// =========================================================================

#[tokio::test]
async fn unknown_slug_returns_404() {
    let (status, body) = get("/blog/this-slug-does-not-exist").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        !body.contains(r#"<article class="blog-post">"#),
        "404 must not render a blog article; got: {}",
        body.chars().take(400).collect::<String>()
    );
}

// =========================================================================
// Sanitization
// =========================================================================

#[tokio::test]
async fn seed_article_html_is_sanitized() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    let body_start = body
        .find("<div class=\"blog-post__body\"")
        .expect("article body must be present");
    let body_end = body[body_start..]
        .find("</article>")
        .map(|offset| body_start + offset)
        .unwrap_or(body.len());
    let article_html = &body[body_start..body_end];
    assert!(
        !article_html.contains("<script"),
        "rendered article must not contain <script>; got: {article_html}"
    );
    assert!(
        !article_html.contains("onerror="),
        "rendered article must not contain inline event handlers; got: {article_html}"
    );
}

// =========================================================================
// hreflang — now uniform 5-element set on every article (en/ru/ko/vi + x-default)
// =========================================================================

#[tokio::test]
async fn en_article_hreflang_lists_all_4_locales() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    for locale in ["en", "ru", "ko", "vi"] {
        let needle = format!(r#"hreflang="{locale}" href="https://"#);
        assert!(
            body.contains(&needle),
            "EN article must declare hreflang={locale}"
        );
    }
    assert!(
        body.contains(r#"hreflang="x-default""#),
        "EN article must declare hreflang=x-default"
    );
}

#[tokio::test]
async fn ko_article_hreflang_includes_all_4_locales() {
    // After Slice-2 the KO translation exists, so /ko/blog/<slug> is a
    // native render (no fallback). Every translation must be advertised.
    let (_, body) = get("/ko/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"hreflang="ko" href="https://"#),
        "KO native article must self-declare hreflang=ko"
    );
    assert!(
        body.contains(r#"hreflang="en" href="https://"#),
        "KO article must advertise hreflang=en"
    );
    assert!(
        body.contains(r#"hreflang="ru" href="https://"#),
        "KO article must advertise hreflang=ru"
    );
}

#[tokio::test]
async fn ko_article_canonical_points_at_ko_url() {
    let (_, body) = get("/ko/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"/ko/blog/anki-alternative-japanese"#),
        "KO native canonical must reference its own KO URL; got first 2000 chars: {}",
        body.chars().take(2000).collect::<String>()
    );
    assert!(
        !body.contains(r#"name="robots" content="noindex"#),
        "KO native article must NOT carry noindex (it's a real translation)"
    );
}

// =========================================================================
// Script consistency — guards against LLM translation artifacts
// =========================================================================

#[tokio::test]
async fn ko_articles_do_not_contain_cyrillic() {
    // KO translations must not contain Cyrillic characters (hallucinated
    // script mixing is a common LLM failure mode). Hangul + ASCII + CJK
    // for kanji terms are the expected scripts.
    for slug in ALL_SLUGS {
        let (_, body) = get(&format!("/ko/blog/{slug}")).await;
        let body_start = body
            .find("<div class=\"blog-post__body\"")
            .unwrap_or(body.len());
        let body_end = body[body_start..]
            .find("</article>")
            .map(|offset| body_start + offset)
            .unwrap_or(body.len());
        let article_html = &body[body_start..body_end];
        let cyrillic_count = article_html
            .chars()
            .filter(|c| ('\u{0400}'..='\u{04FF}').contains(c))
            .count();
        assert_eq!(
            cyrillic_count, 0,
            "KO article {slug} must not contain Cyrillic characters; found {cyrillic_count}"
        );
    }
}

#[tokio::test]
async fn vi_articles_do_not_contain_cyrillic() {
    for slug in ALL_SLUGS {
        let (_, body) = get(&format!("/vi/blog/{slug}")).await;
        let body_start = body
            .find("<div class=\"blog-post__body\"")
            .unwrap_or(body.len());
        let body_end = body[body_start..]
            .find("</article>")
            .map(|offset| body_start + offset)
            .unwrap_or(body.len());
        let article_html = &body[body_start..body_end];
        let cyrillic_count = article_html
            .chars()
            .filter(|c| ('\u{0400}'..='\u{04FF}').contains(c))
            .count();
        assert_eq!(
            cyrillic_count, 0,
            "VI article {slug} must not contain Cyrillic characters; found {cyrillic_count}"
        );
    }
}

#[tokio::test]
async fn vi_articles_do_not_contain_korean() {
    // Hangul syllables: U+AC00–U+D7AF. Hallucinated Korean in a VI
    // translation would be a clear LLM error.
    for slug in ALL_SLUGS {
        let (_, body) = get(&format!("/vi/blog/{slug}")).await;
        let body_start = body
            .find("<div class=\"blog-post__body\"")
            .unwrap_or(body.len());
        let body_end = body[body_start..]
            .find("</article>")
            .map(|offset| body_start + offset)
            .unwrap_or(body.len());
        let article_html = &body[body_start..body_end];
        let hangul_count = article_html
            .chars()
            .filter(|c| ('\u{AC00}'..='\u{D7AF}').contains(c))
            .count();
        assert_eq!(
            hangul_count, 0,
            "VI article {slug} must not contain Hangul; found {hangul_count}"
        );
    }
}

#[tokio::test]
async fn vi_articles_do_not_contain_kanji() {
    // VI SEO strategy (marketing/strategies/origa-seo.md §6 Market 3) requires
    // "Hán tự" across the VI locale — "kanji" is the search miss called out as
    // the single highest-impact content fix. This test guards against regressions
    // when new VI articles are added or existing ones edited.
    //
    // EN-fallback renders (`/vi/blog/<slug>` with no VI translation) serve the
    // English article, where "kanji" is the correct term — they are skipped
    // via the `noindex` robots marker only fallback pages carry.
    //
    // The substring "kanji" is matched case-insensitively. Two proper nouns
    // that contain the substring are subtracted from the count because they
    // are product names, not the conceptual term:
    //   - "KanjiSnap" — iOS OCR app
    //   - "WaniKani"  — does NOT contain "kanji" (substring is "kani"); listed
    //     here for documentation only, no subtraction needed.
    // If a future article cites another `<…>Kanji<…>` proper noun, add it to
    // `PROPER_NOUNS_WITH_KANJI_SUBSTRING` rather than weakening the assertion.
    const PROPER_NOUNS_WITH_KANJI_SUBSTRING: &[&str] = &["kanjisnap"];

    for slug in ALL_SLUGS {
        let (_, body) = get(&format!("/vi/blog/{slug}")).await;
        if body.contains(r#"name="robots" content="noindex"#) {
            continue;
        }
        let body_start = body
            .find("<div class=\"blog-post__body\"")
            .unwrap_or(body.len());
        let body_end = body[body_start..]
            .find("</article>")
            .map(|offset| body_start + offset)
            .unwrap_or(body.len());
        let article_html = &body[body_start..body_end];
        let lower = article_html.to_lowercase();
        let raw_count = lower.matches("kanji").count();
        let proper_noun_count = PROPER_NOUNS_WITH_KANJI_SUBSTRING
            .iter()
            .map(|s| lower.matches(s).count())
            .sum::<usize>();
        let kanji_count = raw_count - proper_noun_count;
        assert_eq!(
            kanji_count, 0,
            "VI article {slug} must use 'hán tự' instead of 'kanji' per SEO strategy; \
             found {kanji_count} conceptual occurrence(s) (raw {raw_count} - proper-noun {proper_noun_count})"
        );
    }
}

#[tokio::test]
async fn ru_articles_do_not_contain_korean() {
    for slug in ALL_SLUGS {
        let (_, body) = get(&format!("/ru/blog/{slug}")).await;
        let body_start = body
            .find("<div class=\"blog-post__body\"")
            .unwrap_or(body.len());
        let body_end = body[body_start..]
            .find("</article>")
            .map(|offset| body_start + offset)
            .unwrap_or(body.len());
        let article_html = &body[body_start..body_end];
        let hangul_count = article_html
            .chars()
            .filter(|c| ('\u{AC00}'..='\u{D7AF}').contains(c))
            .count();
        assert_eq!(
            hangul_count, 0,
            "RU article {slug} must not contain Hangul; found {hangul_count}"
        );
    }
}

// =========================================================================
// Breadcrumb — 3 levels after the hub landing was added
// =========================================================================

#[tokio::test]
async fn en_article_breadcrumb_has_3_levels() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    let block_start = body
        .find(r#""@type":"BreadcrumbList""#)
        .expect("BreadcrumbList must be present");
    let window = &body[block_start..];
    let window_end = window.find("</script>").unwrap_or(window.len());
    let block = &window[..window_end];
    let positions = block.matches(r#""position":"#).count();
    assert_eq!(
        positions, 3,
        "article breadcrumb must have 3 positions (Home → Blog → Article); got block: {block}"
    );
}

// =========================================================================
// Canonical, JSON-LD, keywords
// =========================================================================

#[tokio::test]
async fn en_article_has_canonical_url() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"<link rel="canonical" href="https://"#),
        "blog pages must have a canonical URL; got first 1500 chars: {}",
        body.chars().take(1500).collect::<String>()
    );
    assert!(
        body.contains("/blog/anki-alternative-japanese"),
        "canonical must reference the article slug"
    );
}

#[tokio::test]
async fn en_article_has_article_jsonld() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#""@type":"Article""#),
        "blog pages must emit Article JSON-LD; got first 2000 chars: {}",
        body.chars().take(2000).collect::<String>()
    );
}

#[tokio::test]
async fn en_article_jsonld_has_distinct_dates() {
    // The Article JSON-LD must carry distinct `datePublished` (original
    // publication) and `dateModified` (last edit) values once the
    // `published` frontmatter field is populated. Parsing the JSON-LD block
    // (rather than substring-matching) protects the assertion from
    // serde_json formatting drift.
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    let block = find_jsonld_block_by_type(&body, "Article");
    let value: serde_json::Value =
        serde_json::from_str(&block).expect("Article JSON-LD must be valid JSON");
    assert_eq!(
        value.get("datePublished").and_then(|v| v.as_str()),
        Some("2026-07-19"),
        "datePublished must come from the `published` frontmatter field; got block: {block}"
    );
    assert_eq!(
        value.get("dateModified").and_then(|v| v.as_str()),
        Some("2026-07-20"),
        "dateModified must come from the `lastmod` frontmatter field; got block: {block}"
    );
}

/// Find the first JSON-LD block whose `@type` matches `type_name`. Mirrors the
/// helper in `tests/seo_meta.rs`; kept local so each test binary stays
/// self-contained.
fn find_jsonld_block_by_type(html: &str, type_name: &str) -> String {
    let open = r#"<script type="application/ld+json">"#;
    let close = "</script>";
    let needle = format!("\"@type\":\"{type_name}\"");
    let mut rest = html;
    while let Some(start) = rest.find(open) {
        let body_start = start + open.len();
        let body_end = rest[body_start..]
            .find(close)
            .unwrap_or_else(|| panic!("JSON-LD block not closed"))
            + body_start;
        let block = &rest[body_start..body_end];
        if block.contains(&needle) {
            return block.to_owned();
        }
        rest = &rest[body_end + close.len()..];
    }
    panic!(
        "no JSON-LD block with @type={type_name}; body was: {}",
        html.chars().take(500).collect::<String>()
    )
}

#[tokio::test]
async fn en_article_has_breadcrumb_jsonld() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#""@type":"BreadcrumbList""#),
        "blog pages must emit BreadcrumbList JSON-LD"
    );
}

#[tokio::test]
async fn en_article_has_keywords_meta_from_frontmatter() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    assert!(
        body.contains("anki alternative japanese"),
        "blog keywords meta must come from the article frontmatter"
    );
}

#[tokio::test]
async fn ru_article_has_inline_competitor_citation() {
    let (_, body) = get("/ru/blog/best-japanese-learning-app").await;
    assert!(
        body.contains("wanikani.com"),
        "RU article must link to wanikani.com (competitor citation)"
    );
}

#[tokio::test]
async fn en_article_has_internal_link_to_compare() {
    // Articles must not be PageRank dead-ends — each must deep-link to the
    // landing hub pages. The exact pattern `href="/compare"` (closing quote
    // included) avoids false positives from a substring like `/copmare` or a
    // bare mention in code-block text.
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"href="/compare""#),
        "EN article must deep-link to /compare; got first 3000 chars: {}",
        body.chars().take(3000).collect::<String>()
    );
}

#[tokio::test]
async fn internal_links_in_article_do_not_open_new_tab() {
    // The sanitize policy differentiates internal (relative-href) links from
    // external ones — internal links stay in the same tab. This integration
    // test pins the policy on a real rendered page.
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    let internal_link_start = body
        .find(r#"href="/compare""#)
        .expect("internal /compare link must be present for this assertion");
    let tag_end = body[internal_link_start..]
        .find('>')
        .unwrap_or(body.len() - internal_link_start)
        + internal_link_start;
    let internal_anchor = &body[internal_link_start..=tag_end];
    assert!(
        !internal_anchor.contains("target=\"_blank\""),
        "internal link must not carry target=_blank; got: {internal_anchor}"
    );
    assert!(
        !internal_anchor.contains("rel=\"noopener noreferrer\""),
        "internal link must not carry external rel; got: {internal_anchor}"
    );
}

#[tokio::test]
async fn external_links_in_article_keep_safe_attrs() {
    // External competitor-citation links (e.g. wanikani.com) must keep
    // `rel="noopener noreferrer"` and `target="_blank"` after the
    // sanitize-policy change that strips those attributes from internal links.
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    let external_link_start = body
        .find(r#"href="https://www.wanikani.com/""#)
        .expect("WaniKani external link must be present");
    let tag_end = body[external_link_start..]
        .find('>')
        .unwrap_or(body.len() - external_link_start)
        + external_link_start;
    let external_anchor = &body[external_link_start..=tag_end];
    assert!(
        external_anchor.contains("rel=\"noopener noreferrer\""),
        "external link must keep safe rel; got: {external_anchor}"
    );
    assert!(
        external_anchor.contains("target=\"_blank\""),
        "external link must keep target=_blank; got: {external_anchor}"
    );
}

#[tokio::test]
async fn ru_article_does_not_contain_known_typo() {
    let (_, body) = get("/ru/blog/best-japanese-learning-app").await;
    assert!(
        !body.contains("食ьба"),
        "RU article must not contain the legacy typo `食ьба`"
    );
    assert!(
        body.contains("食事"),
        "RU article must contain the corrected `食事`"
    );
}

#[tokio::test]
async fn ru_article_has_ru_og_locale() {
    let (_, body) = get("/ru/blog/best-japanese-learning-app").await;
    assert!(
        body.contains(r#"property="og:locale" content="ru_RU""#),
        "RU article must declare og:locale=ru_RU"
    );
}

#[tokio::test]
async fn en_article_has_en_og_locale() {
    let (_, body) = get("/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"property="og:locale" content="en_US""#),
        "EN article must declare og:locale=en_US"
    );
}

#[tokio::test]
async fn ko_article_has_ko_og_locale() {
    let (_, body) = get("/ko/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"property="og:locale" content="ko_KR""#),
        "KO native article must declare og:locale=ko_KR; got first 1500 chars: {}",
        body.chars().take(1500).collect::<String>()
    );
}

#[tokio::test]
async fn vi_article_has_vi_og_locale() {
    let (_, body) = get("/vi/blog/anki-alternative-japanese").await;
    assert!(
        body.contains(r#"property="og:locale" content="vi_VN""#),
        "VI native article must declare og:locale=vi_VN"
    );
}

// =========================================================================
// Blog index page (/blog) — list of articles
// =========================================================================

#[tokio::test]
async fn en_blog_index_returns_200_with_article_list() {
    let (status, body) = get("/blog").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("blog-index"),
        "EN blog index must render .blog-index; got first 600 chars: {}",
        body.chars().take(600).collect::<String>()
    );
    // EN has both articles; the index must list them.
    assert!(
        body.contains("Anki Alternative for Japanese"),
        "EN blog index must list the Anki article"
    );
    assert!(
        body.contains("Best Japanese Learning App"),
        "EN blog index must list the Best-app article"
    );
}

#[tokio::test]
async fn ru_blog_index_returns_200_with_localized_articles() {
    let (status, body) = get("/ru/blog").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("blog-index"),
        "RU blog index must render .blog-index"
    );
    // Strict filter: RU index shows only RU articles. KO/VI article URLs
    // must NOT appear (would mislead crawlers into thinking /ko/... is RU).
    assert!(
        !body.contains(r#"href="/ko/blog/"#),
        "RU blog index must not link to KO URLs (strict locale filter)"
    );
}

#[tokio::test]
async fn blog_index_has_itemlist_jsonld() {
    let (_, body) = get("/blog").await;
    assert!(
        body.contains(r#""@type":"ItemList""#),
        "blog index must emit ItemList JSON-LD (eligible for Carousel rich result per ADR-018)"
    );
}

#[tokio::test]
async fn blog_index_does_not_shadow_article_route() {
    // Smoke-test on Leptos 0.8 router: static `Route path="blog"` must
    // match `/blog` exactly, and `Route path="blog/:slug"` must still
    // match `/blog/<slug>`. If the static route shadowed the parametric,
    // `/blog/anki-alternative-japanese` would render the index (or 404)
    // instead of the article.
    let (status, body) = get("/blog/anki-alternative-japanese").await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains(r#"<article class="blog-post">"#),
        "article route must still match (router shadow regression)"
    );
    assert!(
        !body.contains("blog-index__list"),
        "article route must not render the blog index page"
    );
}

#[tokio::test]
async fn all_four_blog_index_urls_return_200() {
    for uri in ["/blog", "/ru/blog", "/ko/blog", "/vi/blog"] {
        let (status, body) = get(uri).await;
        assert_eq!(status, StatusCode::OK, "expected 200 for {uri}");
        assert!(
            body.contains("blog-index"),
            "{uri} must render the blog index page"
        );
    }
}

// =========================================================================
// Header navigation — "Blog" link visible across all locales
// =========================================================================

#[tokio::test]
async fn header_includes_blog_link_on_every_locale() {
    let cases = [
        ("/", "/blog", "Blog"),
        ("/ru", "/ru/blog", "Блог"),
        ("/ko", "/ko/blog", "블로그"),
        ("/vi", "/vi/blog", "Blog"),
    ];
    for (uri, expected_href, expected_label) in cases {
        let (_, body) = get(uri).await;
        let expected_attr = format!(r#"href="{expected_href}""#);
        assert!(
            body.contains(&expected_attr),
            "{uri} header must link to {expected_href}"
        );
        assert!(
            body.contains(expected_label),
            "{uri} header must render the localized Blog label `{expected_label}`"
        );
    }
}

// =========================================================================
// Legacy slug 308 redirect
// =========================================================================

#[tokio::test]
async fn old_ru_article_slug_redirects_308() {
    // The RU article was renamed from `luchshee-prilozhenie-...` to the
    // unified `best-japanese-learning-app`. The old URL must 308-redirect
    // to preserve link equity (Google treats 308 as permanent).
    let (status, _body) = get("/ru/blog/luchshee-prilozhenie-izucheniya-yaponskogo").await;
    assert_eq!(
        status.as_u16(),
        308,
        "old RU slug must return 308 Permanent Redirect"
    );
}

// =========================================================================
// Cross-article coverage (rstest-parameterized over ALL_SLUGS)
// =========================================================================
//
// The earlier sections pin specific contracts on a single seed article
// (e.g. `en_article_has_canonical_url`). The parameterized tests below
// extend the same contracts to every published article so adding a new
// slug to `ALL_SLUGS` automatically pulls it into the SEO-coverage net.
//
// Only `article_returns_200_with_h1` is parameterized over ALL_SLUGS × Locale
// (URL reachability is per-locale). The remaining rstest tests parameterize
// over ALL_SLUGS only and assert on the EN URL (`/blog/<slug>`): hreflang
// block, canonical, JSON-LD, keywords meta, internal-link presence — these
// are locale-independent metadata structures on the article entity, so
// checking EN pins the contract for all locales without redundant cases.
//
// rstest is already a workspace dev-dependency; the parameterization style
// (explicit `#[case]` enumeration) keeps each test self-describing in the
// test runner output — failures point at the exact `(slug, locale)` pair.

use rstest::rstest;

/// Locale code as used in URL paths. EN is the empty prefix (canonical URLs
/// live at `/blog/<slug>`); the other locales live under `/<code>/blog/<slug>`.
fn locale_prefix(locale: &str) -> &'static str {
    match locale {
        "en" => "",
        "ru" => "/ru",
        "ko" => "/ko",
        "vi" => "/vi",
        other => panic!("unknown locale code {other:?}"),
    }
}

#[rstest]
#[case("anki-alternative-japanese", "en")]
#[case("anki-alternative-japanese", "ru")]
#[case("anki-alternative-japanese", "ko")]
#[case("anki-alternative-japanese", "vi")]
#[case("best-japanese-learning-app", "en")]
#[case("best-japanese-learning-app", "ru")]
#[case("best-japanese-learning-app", "ko")]
#[case("best-japanese-learning-app", "vi")]
#[case("learn-japanese-from-manga", "en")]
#[case("learn-japanese-from-manga", "ru")]
#[case("learn-japanese-from-manga", "ko")]
#[case("learn-japanese-from-manga", "vi")]
#[case("japanese-ocr-app", "en")]
#[case("japanese-ocr-app", "ru")]
#[case("japanese-ocr-app", "ko")]
#[case("japanese-ocr-app", "vi")]
#[case("best-japanese-learning-app-offline", "en")]
#[case("best-japanese-learning-app-offline", "ru")]
#[case("best-japanese-learning-app-offline", "ko")]
#[case("best-japanese-learning-app-offline", "vi")]
#[case("japanese-ai-tutor", "en")]
#[case("japanese-ai-tutor", "ru")]
#[case("japanese-ai-tutor", "ko")]
#[case("japanese-ai-tutor", "vi")]
#[case("yaponskiy-s-nulya", "en")]
#[case("yaponskiy-s-nulya", "ru")]
#[case("yaponskiy-s-nulya", "ko")]
#[case("yaponskiy-s-nulya", "vi")]
#[tokio::test]
async fn article_returns_200_with_h1(#[case] slug: &str, #[case] locale: &str) {
    let uri = format!("{}/blog/{slug}", locale_prefix(locale));
    let (status, body) = get(&uri).await;
    assert_eq!(status, StatusCode::OK, "expected 200 for {uri}");
    assert!(
        body.contains(r#"<article class="blog-post">"#),
        "{uri}: article must render inside .blog-post; got first 600 chars: {}",
        body.chars().take(600).collect::<String>()
    );
}

#[rstest]
#[case("learn-hiragana-katakana", "en")]
#[case("learn-hiragana-katakana", "ru")]
#[case("jlpt-n5-preparation", "en")]
#[case("jlpt-n5-preparation", "ru")]
#[case("japanese-textbooks-beginners", "en")]
#[case("japanese-textbooks-beginners", "ru")]
#[case("how-many-kanji-to-learn", "en")]
#[case("how-many-kanji-to-learn", "ru")]
#[case("learn-japanese-from-anime", "en")]
#[case("learn-japanese-from-anime", "ru")]
// EN+RU-only cluster: KO/VI prefixes serve the EN fallback (covered by
// `all_article_urls_return_200`), so native-render cases stop at RU.
#[tokio::test]
async fn partial_coverage_article_returns_200_with_h1(#[case] slug: &str, #[case] locale: &str) {
    let uri = format!("{}/blog/{slug}", locale_prefix(locale));
    let (status, body) = get(&uri).await;
    assert_eq!(status, StatusCode::OK, "expected 200 for {uri}");
    assert!(
        body.contains(r#"<article class="blog-post">"#),
        "{uri}: article must render inside .blog-post; got first 600 chars: {}",
        body.chars().take(600).collect::<String>()
    );
}

/// Full-coverage slugs advertise hreflang for all four locales plus
/// x-default; partial (EN+RU) slugs advertise only {en, ru, x-default}.
#[rstest]
#[case("anki-alternative-japanese", &["en", "ru", "ko", "vi"])]
#[case("best-japanese-learning-app", &["en", "ru", "ko", "vi"])]
#[case("learn-japanese-from-manga", &["en", "ru", "ko", "vi"])]
#[case("japanese-ocr-app", &["en", "ru", "ko", "vi"])]
#[case("best-japanese-learning-app-offline", &["en", "ru", "ko", "vi"])]
#[case("japanese-ai-tutor", &["en", "ru", "ko", "vi"])]
#[case("yaponskiy-s-nulya", &["en", "ru", "ko", "vi"])]
#[case("learn-hiragana-katakana", &["en", "ru"])]
#[case("jlpt-n5-preparation", &["en", "ru"])]
#[case("japanese-textbooks-beginners", &["en", "ru"])]
#[case("how-many-kanji-to-learn", &["en", "ru"])]
#[case("learn-japanese-from-anime", &["en", "ru"])]
#[tokio::test]
async fn article_hreflang_lists_exactly_its_translations(
    #[case] slug: &str,
    #[case] locales: &[&str],
) {
    let (_, body) = get(&format!("/blog/{slug}")).await;
    for locale in locales {
        let needle = format!(r#"hreflang="{locale}" href="https://"#);
        assert!(
            body.contains(&needle),
            "EN article {slug} must declare hreflang={locale}"
        );
    }
    assert!(
        body.contains(r#"hreflang="x-default""#),
        "EN article {slug} must declare hreflang=x-default"
    );
    // Alternates must point only at translations that exist: a full-coverage
    // slug emits 4 locale alternates + x-default, a partial one 2 + x-default.
    let expected = locales.len() + 1;
    let actual = body.matches(r#"rel="alternate" hreflang="#).count();
    assert_eq!(
        actual, expected,
        "EN article {slug} must emit exactly {expected} hreflang links"
    );
}

#[rstest]
#[case("anki-alternative-japanese")]
#[case("best-japanese-learning-app")]
#[case("learn-japanese-from-manga")]
#[case("japanese-ocr-app")]
#[case("best-japanese-learning-app-offline")]
#[case("japanese-ai-tutor")]
#[case("yaponskiy-s-nulya")]
#[tokio::test]
async fn article_has_canonical_url(#[case] slug: &str) {
    let (_, body) = get(&format!("/blog/{slug}")).await;
    assert!(
        body.contains(r#"<link rel="canonical" href="https://"#),
        "blog page {slug} must have a canonical URL; got first 1500 chars: {}",
        body.chars().take(1500).collect::<String>()
    );
    assert!(
        body.contains(&format!("/blog/{slug}")),
        "canonical for {slug} must reference the article slug"
    );
}

#[rstest]
#[case("anki-alternative-japanese")]
#[case("best-japanese-learning-app")]
#[case("learn-japanese-from-manga")]
#[case("japanese-ocr-app")]
#[case("best-japanese-learning-app-offline")]
#[case("japanese-ai-tutor")]
#[case("yaponskiy-s-nulya")]
#[tokio::test]
async fn article_has_article_jsonld(#[case] slug: &str) {
    let (_, body) = get(&format!("/blog/{slug}")).await;
    assert!(
        body.contains(r#""@type":"Article""#),
        "blog page {slug} must emit Article JSON-LD; got first 2000 chars: {}",
        body.chars().take(2000).collect::<String>()
    );
}

#[rstest]
#[case("anki-alternative-japanese")]
#[case("best-japanese-learning-app")]
#[case("learn-japanese-from-manga")]
#[case("japanese-ocr-app")]
#[case("best-japanese-learning-app-offline")]
#[case("japanese-ai-tutor")]
#[case("yaponskiy-s-nulya")]
#[tokio::test]
async fn article_has_internal_link_to_compare(#[case] slug: &str) {
    // Articles must not be PageRank dead-ends — each must deep-link to the
    // landing hub pages. The exact pattern `href="/compare"` (closing quote
    // included) avoids false positives from a substring like `/copmare` or a
    // bare mention in code-block text.
    let (_, body) = get(&format!("/blog/{slug}")).await;
    assert!(
        body.contains(r#"href="/compare""#),
        "EN article {slug} must deep-link to /compare; got first 3000 chars: {}",
        body.chars().take(3000).collect::<String>()
    );
}

#[rstest]
#[case("anki-alternative-japanese", "anki alternative japanese")]
#[case("best-japanese-learning-app", "best japanese learning app")]
#[case("learn-japanese-from-manga", "learn japanese from manga")]
#[case("japanese-ocr-app", "japanese ocr app")]
#[case("best-japanese-learning-app-offline", "learn japanese offline")]
#[case("japanese-ai-tutor", "japanese ai tutor")]
#[case("yaponskiy-s-nulya", "japanese from zero")]
#[case("learn-hiragana-katakana", "learn hiragana")]
#[case("jlpt-n5-preparation", "jlpt n5")]
#[case("japanese-textbooks-beginners", "best japanese textbook")]
#[case("how-many-kanji-to-learn", "how many kanji to learn")]
#[case("learn-japanese-from-anime", "learn japanese from anime")]
#[tokio::test]
async fn article_has_keywords_meta_from_frontmatter(#[case] slug: &str, #[case] keyword: &str) {
    // Every article's first target_keyword must surface in the keywords meta
    // tag — if frontmatter parsing silently drops the field, this test catches
    // it for that specific slug.
    let (_, body) = get(&format!("/blog/{slug}")).await;
    assert!(
        body.contains(keyword),
        "EN article {slug} must expose target_keyword `{keyword}` in meta"
    );
}
