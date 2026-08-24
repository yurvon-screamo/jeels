//! Static registry of blog articles, populated on first access from
//! compile-time-embedded markdown sources.
//!
//! Each article is a tuple of `(locale, slug, source)`. The registry parses
//! every article on first access via [`OnceLock::get_or_init`], panicking on
//! a malformed frontmatter or on a `status: draft` article. The
//! panic-on-draft invariant is what makes "drafts don't ship" a build-time
//! guarantee: any `cargo test --workspace` run that touches the registry
//! will surface the draft before it reaches production.

use std::sync::OnceLock;

use crate::blog::frontmatter::{self, ArticleStatus, Frontmatter};
use crate::blog::render::markdown_to_html;
use crate::content::Locale;

/// A single rendered blog article. The `html` field is sanitized at
/// construction time and safe to inject via `inner_html`.
#[derive(Debug, Clone)]
pub struct BlogPost {
    pub slug: &'static str,
    pub locale: Locale,
    pub frontmatter: Frontmatter,
    pub html: String,
}

/// Compile-time manifest of articles to ship. Each entry is
/// `(locale, slug, raw_markdown_source)`. Add a new article by appending a
/// tuple here and dropping the `.md` file under `content/blog/<locale>/`.
const ARTICLES: &[(Locale, &str, &str)] = &[
    // Article 1: "Anki alternative for Japanese" — published in all 4 locales
    // under one slug so hreflang alternates resolve to a single article group.
    (
        Locale::En,
        "anki-alternative-japanese",
        include_str!("../../content/blog/en/anki-alternative-japanese.md"),
    ),
    (
        Locale::Ru,
        "anki-alternative-japanese",
        include_str!("../../content/blog/ru/anki-alternative-japanese.md"),
    ),
    (
        Locale::Ko,
        "anki-alternative-japanese",
        include_str!("../../content/blog/ko/anki-alternative-japanese.md"),
    ),
    (
        Locale::Vi,
        "anki-alternative-japanese",
        include_str!("../../content/blog/vi/anki-alternative-japanese.md"),
    ),
    // Article 2: "Best Japanese learning app" — same pattern, slug unified
    // across locales. The original RU slug `luchshee-prilozhenie-izucheniya-
    // yaponskogo` is kept alive as a 308 redirect in `server.rs`.
    (
        Locale::En,
        "best-japanese-learning-app",
        include_str!("../../content/blog/en/best-japanese-learning-app.md"),
    ),
    (
        Locale::Ru,
        "best-japanese-learning-app",
        include_str!("../../content/blog/ru/best-japanese-learning-app.md"),
    ),
    (
        Locale::Ko,
        "best-japanese-learning-app",
        include_str!("../../content/blog/ko/best-japanese-learning-app.md"),
    ),
    (
        Locale::Vi,
        "best-japanese-learning-app",
        include_str!("../../content/blog/vi/best-japanese-learning-app.md"),
    ),
    // Article 3: "How to Learn Japanese from Manga" — published in all 4
    // locales under one slug. Manga-mining workflow with tooling landscape
    // (manga-ocr, YomiNinja, Yomitan, KanjiSnap, Origa).
    (
        Locale::En,
        "learn-japanese-from-manga",
        include_str!("../../content/blog/en/learn-japanese-from-manga.md"),
    ),
    (
        Locale::Ru,
        "learn-japanese-from-manga",
        include_str!("../../content/blog/ru/learn-japanese-from-manga.md"),
    ),
    (
        Locale::Ko,
        "learn-japanese-from-manga",
        include_str!("../../content/blog/ko/learn-japanese-from-manga.md"),
    ),
    (
        Locale::Vi,
        "learn-japanese-from-manga",
        include_str!("../../content/blog/vi/learn-japanese-from-manga.md"),
    ),
    // Article 4: "Japanese OCR Apps" — published in all 4 locales under one
    // slug. Tooling comparison across general-purpose, manga-specific, and
    // learning-integrated OCR categories.
    (
        Locale::En,
        "japanese-ocr-app",
        include_str!("../../content/blog/en/japanese-ocr-app.md"),
    ),
    (
        Locale::Ru,
        "japanese-ocr-app",
        include_str!("../../content/blog/ru/japanese-ocr-app.md"),
    ),
    (
        Locale::Ko,
        "japanese-ocr-app",
        include_str!("../../content/blog/ko/japanese-ocr-app.md"),
    ),
    (
        Locale::Vi,
        "japanese-ocr-app",
        include_str!("../../content/blog/vi/japanese-ocr-app.md"),
    ),
    // Article 5: "Best Japanese Learning Apps That Work Offline" — published
    // in all 4 locales. Four-layer model of what "offline" means across
    // categories.
    (
        Locale::En,
        "best-japanese-learning-app-offline",
        include_str!("../../content/blog/en/best-japanese-learning-app-offline.md"),
    ),
    (
        Locale::Ru,
        "best-japanese-learning-app-offline",
        include_str!("../../content/blog/ru/best-japanese-learning-app-offline.md"),
    ),
    (
        Locale::Ko,
        "best-japanese-learning-app-offline",
        include_str!("../../content/blog/ko/best-japanese-learning-app-offline.md"),
    ),
    (
        Locale::Vi,
        "best-japanese-learning-app-offline",
        include_str!("../../content/blog/vi/best-japanese-learning-app-offline.md"),
    ),
    // Article 6: "Japanese AI Tutors" — published in all 4 locales. Honest
    // breakdown of what AI tutors are good at vs. what they leave out
    // (retention, vocabulary capture).
    (
        Locale::En,
        "japanese-ai-tutor",
        include_str!("../../content/blog/en/japanese-ai-tutor.md"),
    ),
    (
        Locale::Ru,
        "japanese-ai-tutor",
        include_str!("../../content/blog/ru/japanese-ai-tutor.md"),
    ),
    (
        Locale::Ko,
        "japanese-ai-tutor",
        include_str!("../../content/blog/ko/japanese-ai-tutor.md"),
    ),
    (
        Locale::Vi,
        "japanese-ai-tutor",
        include_str!("../../content/blog/vi/japanese-ai-tutor.md"),
    ),
    // Article 7: "Japanese from Zero" — RU-source article, translated to all
    // 4 locales. Self-study roadmap from kana to N4 immersion.
    (
        Locale::En,
        "yaponskiy-s-nulya",
        include_str!("../../content/blog/en/yaponskiy-s-nulya.md"),
    ),
    (
        Locale::Ru,
        "yaponskiy-s-nulya",
        include_str!("../../content/blog/ru/yaponskiy-s-nulya.md"),
    ),
    (
        Locale::Ko,
        "yaponskiy-s-nulya",
        include_str!("../../content/blog/ko/yaponskiy-s-nulya.md"),
    ),
    (
        Locale::Vi,
        "yaponskiy-s-nulya",
        include_str!("../../content/blog/vi/yaponskiy-s-nulya.md"),
    ),
    // Articles 8–12: SEO cluster expansion (2026-08) — EN + RU only. KO/VI
    // translations are not published yet, so `locales_for_slug` returns two
    // locales and hreflang alternates stay at {en, ru, x-default}. When a
    // KO/VI translation ships, append its tuple here and widen the sitemap
    // template's hreflang set for that slug.
    // Article 8: "Learn Hiragana and Katakana".
    (
        Locale::En,
        "learn-hiragana-katakana",
        include_str!("../../content/blog/en/learn-hiragana-katakana.md"),
    ),
    (
        Locale::Ru,
        "learn-hiragana-katakana",
        include_str!("../../content/blog/ru/learn-hiragana-katakana.md"),
    ),
    // Article 9: "JLPT N5 Preparation".
    (
        Locale::En,
        "jlpt-n5-preparation",
        include_str!("../../content/blog/en/jlpt-n5-preparation.md"),
    ),
    (
        Locale::Ru,
        "jlpt-n5-preparation",
        include_str!("../../content/blog/ru/jlpt-n5-preparation.md"),
    ),
    // Article 10: "Japanese Textbooks for Beginners".
    (
        Locale::En,
        "japanese-textbooks-beginners",
        include_str!("../../content/blog/en/japanese-textbooks-beginners.md"),
    ),
    (
        Locale::Ru,
        "japanese-textbooks-beginners",
        include_str!("../../content/blog/ru/japanese-textbooks-beginners.md"),
    ),
    // Article 11: "How Many Kanji Do You Need".
    (
        Locale::En,
        "how-many-kanji-to-learn",
        include_str!("../../content/blog/en/how-many-kanji-to-learn.md"),
    ),
    (
        Locale::Ru,
        "how-many-kanji-to-learn",
        include_str!("../../content/blog/ru/how-many-kanji-to-learn.md"),
    ),
    // Article 12: "Learning Japanese Through Anime".
    (
        Locale::En,
        "learn-japanese-from-anime",
        include_str!("../../content/blog/en/learn-japanese-from-anime.md"),
    ),
    (
        Locale::Ru,
        "learn-japanese-from-anime",
        include_str!("../../content/blog/ru/learn-japanese-from-anime.md"),
    ),
];

static REGISTRY: OnceLock<Vec<BlogPost>> = OnceLock::new();

/// Build the registry vector by parsing every article in `ARTICLES`. Called
/// at most once per process via [`OnceLock::get_or_init`]; any later caller
/// reuses the cached value. Panics on a malformed frontmatter or on a draft
/// article — both are programmer errors, not runtime conditions.
fn build_registry() -> Vec<BlogPost> {
    ARTICLES
        .iter()
        .map(|(locale, slug, src)| {
            let locale_code = locale.as_str();
            let (yaml, body) = frontmatter::split_frontmatter(src)
                .unwrap_or_else(|e| panic!("blog/{locale_code}/{slug}.md: {e}"));
            let fm = frontmatter::parse(yaml)
                .unwrap_or_else(|e| panic!("blog/{locale_code}/{slug}.md: {e}"));
            assert!(
                fm.status == ArticleStatus::Ready,
                "blog/{locale_code}/{slug}.md has status {:?}; only Ready articles may ship",
                fm.status,
            );
            assert_eq!(
                fm.locale, *locale,
                "blog/{locale_code}/{slug}.md frontmatter.locale ({:?}) does not match its directory",
                fm.locale,
            );
            let html = markdown_to_html(body);
            BlogPost { slug, locale: *locale, frontmatter: fm, html }
        })
        .collect()
}

/// Read-only access to the full registry. The first call triggers parsing
/// of every article; subsequent calls are free.
pub fn all() -> &'static [BlogPost] {
    REGISTRY.get_or_init(build_registry)
}

/// Find an article by `(locale, slug)`. Returns `None` if no article exists
/// for that exact pair — callers handling locale fallback (e.g. serving the
/// EN article on `/ko/blog/<slug>`) must do the fallback themselves.
pub fn find(locale: Locale, slug: &str) -> Option<&'static BlogPost> {
    all()
        .iter()
        .find(|post| post.locale == locale && post.slug == slug)
}

/// Locales that have a published translation of `slug`. Used to emit correct
/// `hreflang` alternates: a page should only point at translations that
/// actually exist, never at URLs that would 404 or fall back.
pub fn locales_for_slug(slug: &str) -> Vec<Locale> {
    all()
        .iter()
        .filter(|post| post.slug == slug)
        .map(|post| post.locale)
        .collect()
}

/// All articles published in `locale`, sorted by `lastmod` descending.
/// Used by the blog index page (`/blog`, `/ru/blog`, etc.) to render the
/// article list. Strict filter — no EN fallback. If a future article is
/// published in EN only, it will not appear on `/ko/blog`; that is the
/// intended behaviour (an index page advertising an untranslated article
/// would mislead users and waste crawl budget).
pub fn list_by_locale(locale: Locale) -> Vec<&'static BlogPost> {
    let mut posts: Vec<&'static BlogPost> =
        all().iter().filter(|post| post.locale == locale).collect();
    posts.sort_by(|a, b| b.frontmatter.lastmod.cmp(&a.frontmatter.lastmod));
    posts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_contains_en_article() {
        let posts = all();
        assert!(
            posts
                .iter()
                .any(|p| p.slug == "anki-alternative-japanese" && p.locale == Locale::En),
            "EN article missing from registry: {posts:?}"
        );
    }

    #[test]
    fn registry_size_matches_articles_const() {
        // ARTICLES is the single source of truth for what ships; the parsed
        // registry must agree with it byte-for-byte. Drift here means a
        // frontmatter parse failed silently or ARTICLES was edited without
        // confirming the build.
        assert_eq!(
            all().len(),
            ARTICLES.len(),
            "registry length must match ARTICLES const (expected {}, got {})",
            ARTICLES.len(),
            all().len()
        );
    }

    #[test]
    fn find_returns_seed_article() {
        let post =
            find(Locale::En, "anki-alternative-japanese").expect("seed article must be findable");
        assert_eq!(post.locale, Locale::En);
        assert!(
            !post.html.is_empty(),
            "article body must render to non-empty HTML"
        );
    }

    #[test]
    fn find_returns_none_for_unknown_slug() {
        assert!(find(Locale::En, "does-not-exist").is_none());
    }

    #[test]
    fn locales_for_anki_slug_returns_all_four_locales() {
        let locales = locales_for_slug("anki-alternative-japanese");
        assert!(locales.contains(&Locale::En), "missing EN: {locales:?}");
        assert!(locales.contains(&Locale::Ru), "missing RU: {locales:?}");
        assert!(locales.contains(&Locale::Ko), "missing KO: {locales:?}");
        assert!(locales.contains(&Locale::Vi), "missing VI: {locales:?}");
        assert_eq!(locales.len(), 4, "expected exactly 4 locales: {locales:?}");
    }

    #[test]
    fn locales_for_best_app_slug_returns_all_four_locales() {
        let locales = locales_for_slug("best-japanese-learning-app");
        assert_eq!(locales.len(), 4, "expected 4 locales: {locales:?}");
    }

    #[test]
    fn list_by_locale_returns_all_native_articles() {
        // Strict filter — no cross-locale bleed. The per-locale count equals
        // ARTICLES entries for that locale, whatever that number happens to
        // be at this point in the content roadmap. Critical for the index
        // page so it doesn't advertise untranslated articles via a locale URL.
        for locale in Locale::ALL {
            let posts = list_by_locale(*locale);
            let expected = ARTICLES.iter().filter(|(l, _, _)| *l == *locale).count();
            assert_eq!(
                posts.len(),
                expected,
                "expected {expected} articles in {:?}, got {}; list_by_locale drift",
                locale,
                posts.len()
            );
            for post in &posts {
                assert_eq!(
                    post.locale, *locale,
                    "list_by_locale({:?}) leaked non-native post: {:?}",
                    locale, post.slug
                );
            }
        }
    }

    #[test]
    fn seed_article_html_is_sanitized() {
        let post = find(Locale::En, "anki-alternative-japanese").expect("seed present");
        assert!(
            !post.html.contains("<script"),
            "rendered article must not contain <script>; got: {}",
            &post.html[..post.html.len().min(400)]
        );
    }
}
