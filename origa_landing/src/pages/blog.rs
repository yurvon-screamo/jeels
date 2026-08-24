//! `/blog` index and `/blog/<slug>` article pages.
//!
//! The index lists every article published in the requested locale (strict
//! filter — no EN fallback; an untranslated article on `/ko/blog` would
//! mislead users and waste crawl budget). The article page falls back to
//! the English version when no translation exists, with `robots: noindex,
//! follow` and a canonical URL pointing at the EN version.

use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::use_params_map;

use crate::blog::{self, BlogPost};
use crate::components::seo::{
    PageMeta, SchemaOrg, article_schema, breadcrumb_schema, item_list_schema,
};
use crate::content::Locale;

const BASE_URL: &str = env!("ORIGA_LANDING_BASE_URL");

#[component]
pub fn BlogIndexPage() -> impl IntoView {
    let locale = use_context::<Locale>().expect("Locale context missing");
    let c = locale.content();
    let posts = blog::list_by_locale(locale);

    // Build ItemList entries with owned paths so the slice outlives the
    // schema call. Paths are short-lived locals, not leaked — they drop
    // when `path_storage` goes out of scope at the end of this function.
    let path_storage: Vec<String> = posts
        .iter()
        .map(|post| format!("/blog/{}", post.slug))
        .collect();
    let item_list_items: Vec<(&str, &str)> = posts
        .iter()
        .zip(path_storage.iter())
        .map(|(post, path)| (post.frontmatter.title.as_str(), path.as_str()))
        .collect();
    let item_list_json = item_list_schema(locale, &item_list_items);
    let breadcrumb_json = breadcrumb_schema(
        locale,
        &[(c.breadcrumb_home, "/"), (c.breadcrumb_blog, "/blog")],
    );

    view! {
        <PageMeta
            locale
            path="/blog"
            title=c.blog_index_meta_title
            description=c.blog_index_meta_description
        />
        <SchemaOrg json=item_list_json/>
        <SchemaOrg json=breadcrumb_json/>

        <section class="blog-index">
            <header class="blog-index__header">
                <h1 class="blog-index__title">{c.blog_index_h1}</h1>
                <p class="blog-index__subtitle">{c.blog_index_subtitle}</p>
            </header>
            {if posts.is_empty() {
                view! { <p class="blog-index__empty">{c.blog_index_empty}</p> }.into_any()
            } else {
                view! {
                    <ul class="blog-index__list">
                        {posts.iter().map(|post| {
                            let href = format!("{}/blog/{}", locale.path_prefix(), post.slug);
                            view! {
                                <li class="blog-index__item">
                                    <a href=href class="blog-index__item-link">
                                        <h2 class="blog-index__item-title">
                                            {post.frontmatter.title.clone()}
                                        </h2>
                                        <p class="blog-index__item-meta">
                                            {c.blog_updated_label} " " {post.frontmatter.lastmod.clone()}
                                        </p>
                                        <p class="blog-index__item-description">
                                            {post.frontmatter.meta_description.clone()}
                                        </p>
                                    </a>
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                }.into_any()
            }}
        </section>
    }
}

/// Static descriptor of how a request was resolved. The fallback variant
/// records that the user asked for a non-EN locale but received the EN
/// article — drives the `noindex` + canonical-redirect SEO signals.
enum Resolution {
    Native(&'static BlogPost),
    Fallback(&'static BlogPost),
    NotFound,
}

#[component]
pub fn BlogPostPage() -> impl IntoView {
    let locale = use_context::<Locale>().expect("Locale context missing");
    let params = use_params_map();
    let slug = params.read().get("slug").unwrap_or_default();

    let resolution = resolve(locale, &slug);

    match resolution {
        Resolution::NotFound => crate::components::NotFound.into_any(),
        Resolution::Native(post) => render_article(locale, post, false).into_any(),
        Resolution::Fallback(post) => render_article(locale, post, true).into_any(),
    }
}

fn resolve(locale: Locale, slug: &str) -> Resolution {
    if slug.is_empty() {
        return Resolution::NotFound;
    }
    if let Some(post) = blog::find(locale, slug) {
        return Resolution::Native(post);
    }
    match blog::find(Locale::En, slug) {
        Some(post) => Resolution::Fallback(post),
        None => Resolution::NotFound,
    }
}

/// Computed SEO metadata for one article render. Grouping these into a
/// struct keeps [`render_article`] short and lets unit tests cover URL /
/// hreflang logic without spinning up the full Leptos view.
struct ArticleMeta {
    canonical: String,
    keywords: String,
    og_image: String,
    og_locale: &'static str,
    canonical_locale: Locale,
    x_default_locale: Locale,
}

impl ArticleMeta {
    fn compute(requested_locale: Locale, post: &BlogPost, is_fallback: bool) -> Self {
        let canonical_locale = if is_fallback {
            Locale::En
        } else {
            requested_locale
        };
        let translation_locales = blog::locales_for_slug(post.slug);

        // `x-default` must point at a URL that actually serves content. The
        // EN article is the canonical fallback when it exists; otherwise we
        // reuse the canonical URL of the article being served (e.g. an
        // RU-only post has no EN version to fall back to, so x-default
        // points at itself).
        let x_default_locale = if translation_locales.contains(&Locale::En) {
            Locale::En
        } else {
            canonical_locale
        };

        Self {
            canonical: format!("{BASE_URL}{}", article_url(canonical_locale, post.slug)),
            keywords: post.frontmatter.target_keywords.join(", "),
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
            x_default_locale,
        }
    }
}

fn render_article(
    requested_locale: Locale,
    post: &'static BlogPost,
    is_fallback: bool,
) -> impl IntoView {
    let meta = ArticleMeta::compute(requested_locale, post, is_fallback);
    let article_json = article_schema(meta.canonical_locale, post, &meta.canonical);
    let c = meta.canonical_locale.content();
    let breadcrumb_path = format!("/blog/{}", post.slug);
    let breadcrumb_json = breadcrumb_schema(
        meta.canonical_locale,
        &[
            (c.breadcrumb_home, "/"),
            (c.breadcrumb_blog, "/blog"),
            (post.frontmatter.title.as_str(), &breadcrumb_path),
        ],
    );
    let faq_entries = crate::blog::faq::extract_faq(&post.html);
    let faq_json = if faq_entries.is_empty() {
        None
    } else {
        let pairs: Vec<(&str, &str)> = faq_entries
            .iter()
            .map(|entry| (entry.question.as_str(), entry.answer.as_str()))
            .collect();
        Some(crate::components::seo::faq_schema(
            meta.canonical_locale,
            &pairs,
        ))
    };
    let translation_locales = blog::locales_for_slug(post.slug);

    view! {
        <Title text=post.frontmatter.meta_title.clone()/>
        <Meta name="description" content=post.frontmatter.meta_description.clone()/>
        <Meta name="keywords" content=meta.keywords/>
        {if is_fallback {
            view! { <Meta name="robots" content="noindex, follow"/> }.into_any()
        } else {
            ().into_any()
        }}
        <Meta property="og:title" content=post.frontmatter.meta_title.clone()/>
        <Meta property="og:description" content=post.frontmatter.meta_description.clone()/>
        <Meta property="og:type" content="article"/>
        <Meta property="og:image" content=meta.og_image.clone()/>
        <Meta property="og:url" content=meta.canonical.clone()/>
        <Meta property="og:locale" content=meta.og_locale/>
        <Meta name="twitter:card" content="summary_large_image"/>
        <Meta name="twitter:title" content=post.frontmatter.meta_title.clone()/>
        <Meta name="twitter:description" content=post.frontmatter.meta_description.clone()/>
        <Meta name="twitter:image" content=meta.og_image/>
        <link rel="canonical" href=meta.canonical.clone()/>
        {translation_locales
            .iter()
            .map(|loc| {
                let href = format!("{BASE_URL}{}", article_url(*loc, post.slug));
                view! { <link rel="alternate" hreflang=loc.as_str() href=href.clone()/> }
            })
            .collect_view()}
        <link rel="alternate" hreflang="x-default" href=format!(
            "{BASE_URL}{}",
            article_url(meta.x_default_locale, post.slug),
        )/>

        <SchemaOrg json=article_json/>
        <SchemaOrg json=breadcrumb_json/>
        {match faq_json {
            Some(json) => view! { <SchemaOrg json/> }.into_any(),
            None => ().into_any(),
        }}

        <ArticleBody requested_locale post is_fallback />
    }
}

#[component]
fn ArticleBody(
    requested_locale: Locale,
    post: &'static BlogPost,
    is_fallback: bool,
) -> impl IntoView {
    let c = requested_locale.content();
    let locale_marker = if is_fallback {
        format!(
            "Showing English article · {}",
            requested_locale.display_label()
        )
    } else {
        requested_locale.display_label().to_string()
    };

    view! {
        <article class="blog-post">
            <header class="blog-post__header">
                <p class="blog-post__locale-marker">{locale_marker}</p>
                <p class="blog-post__updated">
                    {c.blog_updated_label} " " {post.frontmatter.lastmod.clone()}
                </p>
            </header>
            // The single `<h1>` comes from the markdown body itself (every
            // article opens with `# Title`). Rendering a second one here from
            // the frontmatter produced two identical H1s per page — a
            // relevance-diluting signal for crawlers. The body heading is
            // styled via `.blog-post__body h1:first-of-type`.
            <div class="blog-post__body" inner_html=post.html.as_str()></div>
        </article>
    }
}

fn article_url(locale: Locale, slug: &str) -> String {
    // `Locale::En.path_prefix()` is `""`, so a single format string covers
    // both the EN (`/blog/<slug>`) and prefixed (`/ru/blog/<slug>`) cases.
    format!("{}/blog/{slug}", locale.path_prefix())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn article_url_en_has_no_locale_prefix() {
        assert_eq!(
            article_url(Locale::En, "anki-alternative-japanese"),
            "/blog/anki-alternative-japanese"
        );
    }

    #[test]
    fn article_url_ru_has_locale_prefix() {
        assert_eq!(
            article_url(Locale::Ru, "luchshee-prilozhenie"),
            "/ru/blog/luchshee-prilozhenie"
        );
    }
}
