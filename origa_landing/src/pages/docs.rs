//! `/docs` index and `/docs/<slug>` article pages.
//!
//! The index renders the `index.md` content for the requested locale. Article
//! pages fall back to the English version when no translation exists (KO/VI),
//! with `robots: noindex, follow` and a canonical URL pointing at the EN
//! version — mirroring the blog fallback strategy. Both layouts share a
//! two-column structure: a fixed sidebar listing every doc page and a content
//! column rendering the markdown body.

use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::components::seo::{SchemaOrg, breadcrumb_schema, item_list_schema, tech_article_schema};
use crate::content::Locale;
use crate::docs::{self, DocPage};

const BASE_URL: &str = env!("ORIGA_LANDING_BASE_URL");

#[component]
pub fn DocsIndexPage() -> impl IntoView {
    let locale = use_context::<Locale>().expect("Locale context missing");
    let page = docs::index_page(locale);
    let is_fallback = page.locale != locale;

    render_doc_page(locale, page, is_fallback)
}

#[component]
pub fn DocsArticlePage() -> impl IntoView {
    let locale = use_context::<Locale>().expect("Locale context missing");
    let params = use_params_map();
    let slug = params.read().get("slug").unwrap_or_default().to_string();

    let resolution = resolve(locale, &slug);

    match resolution {
        Resolution::NotFound => crate::components::NotFound.into_any(),
        Resolution::Native(page) => render_doc_page(locale, page, false).into_any(),
        Resolution::Fallback(page) => render_doc_page(locale, page, true).into_any(),
    }
}

/// Static descriptor of how a request was resolved. The fallback variant
/// records that the user asked for a non-EN locale but received the EN page —
/// drives the `noindex` + canonical-redirect SEO signals.
enum Resolution {
    Native(&'static DocPage),
    Fallback(&'static DocPage),
    NotFound,
}

fn resolve(locale: Locale, slug: &str) -> Resolution {
    if slug.is_empty() || slug == "index" {
        return Resolution::NotFound;
    }
    if let Some(page) = docs::find(locale, slug) {
        return Resolution::Native(page);
    }
    match docs::find(Locale::En, slug) {
        Some(page) => Resolution::Fallback(page),
        None => Resolution::NotFound,
    }
}

/// Computed SEO metadata for one doc page render.
struct DocMeta {
    canonical: String,
    keywords: String,
    og_image: String,
    og_locale: &'static str,
    canonical_locale: Locale,
}

impl DocMeta {
    fn compute(requested_locale: Locale, doc: &DocPage, is_fallback: bool) -> Self {
        let canonical_locale = if is_fallback {
            Locale::En
        } else {
            requested_locale
        };

        Self {
            canonical: format!("{BASE_URL}{}", doc_url(canonical_locale, doc.slug)),
            keywords: doc.frontmatter.target_keywords.join(", "),
            og_image: format!(
                "{BASE_URL}/images/{}.hero.png",
                requested_locale.image_prefix()
            ),
            og_locale: if is_fallback {
                Locale::En.og_locale()
            } else {
                requested_locale.og_locale()
            },
            canonical_locale,
        }
    }
}

fn render_doc_page(
    requested_locale: Locale,
    page: &'static DocPage,
    is_fallback: bool,
) -> impl IntoView {
    let meta = DocMeta::compute(requested_locale, page, is_fallback);
    let c = meta.canonical_locale.content();
    let tech_article_json = tech_article_schema(meta.canonical_locale, page, &meta.canonical);
    let breadcrumb_path = format!("/docs/{}", page.slug);
    let breadcrumb_json = if page.slug == "index" {
        breadcrumb_schema(
            meta.canonical_locale,
            &[(c.breadcrumb_home, "/"), (c.breadcrumb_docs, "/docs")],
        )
    } else {
        breadcrumb_schema(
            meta.canonical_locale,
            &[
                (c.breadcrumb_home, "/"),
                (c.breadcrumb_docs, "/docs"),
                (page.frontmatter.title.as_str(), &breadcrumb_path),
            ],
        )
    };
    let item_list_items = build_item_list_items(meta.canonical_locale);
    let item_list_refs: Vec<(&str, &str)> = item_list_items
        .iter()
        .map(|(title, path)| (title.as_str(), path.as_str()))
        .collect();
    let item_list_json = item_list_schema(meta.canonical_locale, &item_list_refs);
    let translation_locales = docs::locales_for_slug(page.slug);

    let x_default_url = if translation_locales.contains(&Locale::En) {
        doc_url(Locale::En, page.slug)
    } else {
        doc_url(meta.canonical_locale, page.slug)
    };

    view! {
        <Title text=page.frontmatter.meta_title.clone()/>
        <Meta name="description" content=page.frontmatter.meta_description.clone()/>
        <Meta name="keywords" content=meta.keywords.clone()/>
        {if is_fallback {
            view! { <Meta name="robots" content="noindex, follow"/> }.into_any()
        } else {
            ().into_any()
        }}
        <Meta property="og:title" content=page.frontmatter.meta_title.clone()/>
        <Meta property="og:description" content=page.frontmatter.meta_description.clone()/>
        <Meta property="og:type" content="article"/>
        <Meta property="og:image" content=meta.og_image.clone()/>
        <Meta property="og:url" content=meta.canonical.clone()/>
        <Meta property="og:locale" content=meta.og_locale/>
        <Meta name="twitter:card" content="summary_large_image"/>
        <Meta name="twitter:title" content=page.frontmatter.meta_title.clone()/>
        <Meta name="twitter:description" content=page.frontmatter.meta_description.clone()/>
        <Meta name="twitter:image" content=meta.og_image/>
        <link rel="canonical" href=meta.canonical.clone()/>
        {translation_locales
            .iter()
            .map(|loc| {
                let href = format!("{BASE_URL}{}", doc_url(*loc, page.slug));
                view! { <link rel="alternate" hreflang=loc.as_str() href=href.clone()/> }
            })
            .collect_view()}
        <link rel="alternate" hreflang="x-default" href=format!("{BASE_URL}{}", x_default_url)/>

        <SchemaOrg json=tech_article_json/>
        <SchemaOrg json=breadcrumb_json/>
        {if page.slug == "index" {
            view! { <SchemaOrg json=item_list_json/> }.into_any()
        } else {
            ().into_any()
        }}

        <DocsLayout
            requested_locale
            canonical_locale=meta.canonical_locale
            page
            is_fallback
        />
    }
}

fn build_item_list_items(locale: Locale) -> Vec<(String, String)> {
    docs::SIDEBAR_SLUGS
        .iter()
        .filter_map(|slug| {
            let page = docs::find(locale, slug)?;
            Some((page.frontmatter.title.clone(), format!("/docs/{slug}")))
        })
        .collect()
}

#[component]
fn DocsLayout(
    requested_locale: Locale,
    canonical_locale: Locale,
    page: &'static DocPage,
    is_fallback: bool,
) -> impl IntoView {
    let c = canonical_locale.content();
    let locale_marker = if is_fallback {
        format!(
            "Showing English page · {}",
            requested_locale.display_label()
        )
    } else {
        requested_locale.display_label().to_string()
    };

    view! {
        <div class="docs-layout">
            <DocsSidebar locale=requested_locale/>
            <main class="docs-main">
                <article class="docs-article">
                    <header class="docs-article__header">
                        <p class="docs-article__locale-marker">{locale_marker}</p>
                        <h1 class="docs-article__title">{page.frontmatter.title.clone()}</h1>
                        <p class="docs-article__updated">
                            {c.docs_updated_label} " " {page.frontmatter.lastmod.clone()}
                        </p>
                    </header>
                    <div class="docs-article__body" inner_html=page.html.as_str()></div>
                </article>
            </main>
        </div>
    }
}

#[component]
fn DocsSidebar(locale: Locale) -> impl IntoView {
    let entries = docs::sidebar_entries(locale);
    let c = locale.content();
    let docs_home_href = format!("{}/docs", locale.path_prefix());

    view! {
        <aside class="docs-sidebar">
            <a href=docs_home_href class="docs-sidebar__heading-link">
                <span class="docs-sidebar__heading">{c.breadcrumb_docs}</span>
            </a>
            <nav class="docs-sidebar__nav" aria-label="Documentation">
                <ul class="docs-sidebar__list">
                    {entries.into_iter().map(|(slug, title, _)| {
                        let href = format!("{}/docs/{slug}", locale.path_prefix());
                        view! {
                            <li class="docs-sidebar__item">
                                <A href=href attr:class="docs-sidebar__link">
                                    {title}
                                </A>
                            </li>
                        }
                    }).collect_view()}
                </ul>
            </nav>
        </aside>
    }
}

/// Build the locale-prefixed URL for a doc page. EN has no prefix
/// (`/docs/<slug>`); other locales get `/<code>/docs/<slug>`. The `index`
/// slug maps to the bare `/docs` path (the docs landing), not `/docs/index`.
fn doc_url(locale: Locale, slug: &str) -> String {
    if slug == "index" {
        format!("{}/docs", locale.path_prefix())
    } else {
        format!("{}/docs/{slug}", locale.path_prefix())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_url_en_has_no_locale_prefix() {
        assert_eq!(
            doc_url(Locale::En, "getting-started"),
            "/docs/getting-started"
        );
    }

    #[test]
    fn doc_url_ru_has_locale_prefix() {
        assert_eq!(
            doc_url(Locale::Ru, "getting-started"),
            "/ru/docs/getting-started"
        );
    }

    #[test]
    fn doc_url_index_en_is_bare_docs_path() {
        assert_eq!(doc_url(Locale::En, "index"), "/docs");
    }

    #[test]
    fn doc_url_index_ru_has_locale_prefix() {
        assert_eq!(doc_url(Locale::Ru, "index"), "/ru/docs");
    }

    #[test]
    fn resolve_returns_not_found_for_index_slug() {
        assert!(matches!(resolve(Locale::En, "index"), Resolution::NotFound));
    }

    #[test]
    fn resolve_returns_not_found_for_empty_slug() {
        assert!(matches!(resolve(Locale::En, ""), Resolution::NotFound));
    }

    #[test]
    fn resolve_returns_native_for_en_slug() {
        assert!(matches!(
            resolve(Locale::En, "getting-started"),
            Resolution::Native(_)
        ));
    }

    #[test]
    fn resolve_returns_fallback_for_ko_slug() {
        assert!(matches!(
            resolve(Locale::Ko, "getting-started"),
            Resolution::Fallback(_)
        ));
    }

    #[test]
    fn resolve_returns_not_found_for_unknown_slug() {
        assert!(matches!(
            resolve(Locale::En, "does-not-exist"),
            Resolution::NotFound
        ));
    }
}
