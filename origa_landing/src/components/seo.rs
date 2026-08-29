use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::content::Locale;

const BASE_URL: &str = env!("ORIGA_LANDING_BASE_URL");

#[component]
pub fn SchemaOrg(json: String) -> impl IntoView {
    // serde_json::to_string does not escape `<`, so a string field containing
    // `</script>` would let an attacker break out of the inline JSON-LD
    // <script> block. Escaping `<` as \u003c neutralises the only character
    // that can terminate the tag; the JSON parser still reads it as `<`.
    let json = json.replace('<', "\\u003c");
    view! {
        <script type="application/ld+json" inner_html=json></script>
    }
}

pub fn software_application_schema(locale: Locale) -> String {
    let c = locale.content();
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "SoftwareApplication",
        "name": "Origa",
        "applicationCategory": "EducationalApplication",
        "operatingSystem": "Windows, Linux, macOS, Android, Web",
        "description": c.home_meta_description,
        "featureList": c.home_schema_feature_list,
        "inLanguage": locale.as_str()
    })
    .to_string()
}

pub fn organization_schema() -> String {
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "Organization",
        "name": "Origa",
        "url": BASE_URL,
        "logo": format!("{BASE_URL}/favicon.png"),
        "sameAs": ["https://github.com/yurvon-screamo/origa"]
    })
    .to_string()
}

pub fn breadcrumb_schema(locale: Locale, items: &[(&str, &str)]) -> String {
    let prefix = locale.path_prefix();
    // Each `(name, path)` pair becomes one BreadcrumbList entry. `path` is
    // locale-independent (`/features`, `/blog/<slug>`); the locale prefix
    // (`/ru`, `/ko`, `/vi`) is applied here so callers stay prefix-agnostic.
    // The root path `"/"` is special-cased: it must carry a trailing slash
    // for EN (the canonical root URL) and no trailing slash for locale roots
    // (ADR-011). Every other path is appended verbatim after the prefix.
    let items_json: Vec<_> = items
        .iter()
        .enumerate()
        .map(|(i, (name, path))| {
            let url = if *path == "/" {
                if prefix.is_empty() {
                    format!("{BASE_URL}/")
                } else {
                    format!("{BASE_URL}{prefix}")
                }
            } else {
                format!("{BASE_URL}{prefix}{path}")
            };
            serde_json::json!({
                "@type": "ListItem",
                "position": i + 1,
                "name": name,
                "item": url
            })
        })
        .collect();
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "BreadcrumbList",
        "itemListElement": items_json
    })
    .to_string()
}

/// Schema.org `ItemList` JSON-LD for a listing page (e.g. `/blog` index).
/// Per ADR-018, `ItemList` is preferred over the semantic-but-rich-result-
/// less `Blog` type, because `ItemList` is eligible for Google's Carousel
/// rich result. Each item is a `(name, path)` pair where `path` is locale-
/// independent; the locale prefix is applied uniformly here.
pub fn item_list_schema(locale: Locale, items: &[(&str, &str)]) -> String {
    let prefix = locale.path_prefix();
    let items_json: Vec<_> = items
        .iter()
        .enumerate()
        .map(|(i, (name, path))| {
            serde_json::json!({
                "@type": "ListItem",
                "position": i + 1,
                "name": name,
                "url": format!("{BASE_URL}{prefix}{path}")
            })
        })
        .collect();
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "ItemList",
        "itemListElement": items_json
    })
    .to_string()
}

/// Schema.org `Article` JSON-LD for a blog post. `datePublished` is sourced
/// from the optional `published` frontmatter field (falling back to `lastmod`
/// when absent for backward compatibility with articles that predate the
/// field). `dateModified` is always the `lastmod` frontmatter value, so the
/// two diverge only when an editor records the original publication date
/// separately from the most-recent edit.
pub fn article_schema(locale: Locale, post: &crate::blog::BlogPost, canonical_url: &str) -> String {
    let date_published = post
        .frontmatter
        .published
        .as_deref()
        .unwrap_or(&post.frontmatter.lastmod);
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": post.frontmatter.title,
        "description": post.frontmatter.meta_description,
        "inLanguage": locale.as_str(),
        "datePublished": date_published,
        "dateModified": post.frontmatter.lastmod,
        "mainEntityOfPage": canonical_url,
        "image": format!("{BASE_URL}/og-image.png"),
        "author": {
            "@type": "Organization",
            "name": "Origa",
            "url": BASE_URL
        },
        "publisher": {
            "@type": "Organization",
            "name": "Origa",
            "logo": format!("{BASE_URL}/favicon.png")
        }
    })
    .to_string()
}

/// Schema.org `TechArticle` JSON-LD for a documentation page. Mirrors
/// [`article_schema`] but uses the `TechArticle` type, which is more accurate
/// for reference/how-to documentation than the news-oriented `Article`.
/// `datePublished` falls back to `lastmod` when the optional `published`
/// frontmatter field is absent (same logic as `article_schema`).
pub fn tech_article_schema(
    locale: Locale,
    doc: &crate::docs::DocPage,
    canonical_url: &str,
) -> String {
    let date_published = doc
        .frontmatter
        .published
        .as_deref()
        .unwrap_or(&doc.frontmatter.lastmod);
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "TechArticle",
        "headline": doc.frontmatter.title,
        "description": doc.frontmatter.meta_description,
        "inLanguage": locale.as_str(),
        "datePublished": date_published,
        "dateModified": doc.frontmatter.lastmod,
        "mainEntityOfPage": canonical_url,
        "image": format!("{BASE_URL}/og-image.png"),
        "author": {
            "@type": "Organization",
            "name": "Origa",
            "url": BASE_URL
        },
        "publisher": {
            "@type": "Organization",
            "name": "Origa",
            "logo": format!("{BASE_URL}/favicon.png")
        }
    })
    .to_string()
}

pub fn learning_resource_schema(locale: Locale) -> String {
    let c = locale.content();
    // `teaches` is localised per locale (schema.org treats it as free text, so
    // a localised value improves per-locale SEO). `educationalLevel` (JLPT
    // N5–N1) stays canonical English: those are the international level names,
    // and `learningResourceType` is an enum-like value consumers match on.
    let teaches = [
        c.learning_resource_teaches_vocab,
        c.learning_resource_teaches_kanji,
        c.learning_resource_teaches_grammar,
        c.learning_resource_teaches_listening,
    ];
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "LearningResource",
        "name": "Origa",
        "description": c.home_meta_description,
        "inLanguage": locale.as_str(),
        "learningResourceType": "Interactive Application",
        "educationalLevel": ["JLPT N5", "JLPT N4", "JLPT N3", "JLPT N2", "JLPT N1"],
        "audience": {
            "@type": "EducationalAudience",
            "EducationalRole": "student"
        },
        "isAccessibleForFree": true,
        "teaches": teaches
    })
    .to_string()
}

pub fn faq_schema(locale: Locale, qas: &[(&str, &str)]) -> String {
    let entities: Vec<_> = qas
        .iter()
        .map(|(question, answer)| {
            serde_json::json!({
                "@type": "Question",
                "name": question,
                "acceptedAnswer": {
                    "@type": "Answer",
                    "text": answer
                }
            })
        })
        .collect();
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "FAQPage",
        "inLanguage": locale.as_str(),
        "mainEntity": entities
    })
    .to_string()
}

pub fn how_to_schema(locale: Locale, name: &str, steps: &[&str]) -> String {
    let steps_json: Vec<_> = steps
        .iter()
        .enumerate()
        .map(|(i, step)| {
            serde_json::json!({
                "@type": "HowToStep",
                "position": i + 1,
                "text": step
            })
        })
        .collect();

    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "HowTo",
        "name": name,
        "inLanguage": locale.as_str(),
        "step": steps_json
    })
    .to_string()
}

#[component]
pub fn PageMeta(
    locale: Locale,
    #[prop(default = "")] path: &'static str,
    title: &'static str,
    description: &'static str,
) -> impl IntoView {
    let img_prefix = locale.image_prefix();
    let og_image = format!("{BASE_URL}/images/{img_prefix}.hero.png");
    let canonical = format!("{BASE_URL}{}{path}", locale.path_prefix());
    let c = locale.content();

    view! {
        <Title text=title/>
        <Meta name="description" content=description/>
        <Meta name="keywords" content=c.keywords/>
        <Meta property="og:title" content=title/>
        <Meta property="og:description" content=description/>
        <Meta property="og:image" content=og_image.clone()/>
        <Meta property="og:url" content=canonical.clone()/>
        <Meta property="og:type" content="website"/>
        <Meta property="og:locale" content=locale.og_locale()/>
        {Locale::ALL
            .iter()
            .filter(|loc| **loc != locale)
            .map(|loc| {
                view! { <Meta property="og:locale:alternate" content=loc.og_locale()/> }
            })
            .collect_view()}
        <Meta name="twitter:card" content="summary_large_image"/>
        <Meta name="twitter:title" content=title/>
        <Meta name="twitter:description" content=description/>
        <Meta name="twitter:image" content=og_image/>
        <link rel="canonical" href=canonical/>
        {Locale::ALL.iter().map(|loc| {
            let href = format!("{BASE_URL}{}{path}", loc.path_prefix());
            let hreflang = loc.as_str();
            view! { <link rel="alternate" hreflang=hreflang href=href/> }
        }).collect_view()}
        <link rel="alternate" hreflang="x-default" href=format!("{BASE_URL}{path}")/>
    }
}
