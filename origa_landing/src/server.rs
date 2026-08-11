//! Axum router construction for `origa_landing` SSR.
//!
//! Kept in a library module (rather than inline in `main.rs`) so integration
//! tests in `tests/` can build the exact same router the binary serves.

use std::str::FromStr;

use axum::Router;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Redirect, Response};
use http::header::{
    ACCEPT_LANGUAGE, CACHE_CONTROL, COOKIE, HeaderName, HeaderValue, SET_COOKIE, VARY,
};
use http::{HeaderMap, Method, Request, Uri};
use leptos::config::LeptosOptions;
use leptos_axum::{ErrorHandler, LeptosRoutes};
use tower_http::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

use crate::app::{App, shell};
use crate::content::{LOCALE_COOKIE, LOCALE_COOKIE_MAX_AGE_SECS, Locale};

/// HTML pages: short edge cache so users pick up content/nav changes within
/// 5 minutes, while still letting Cloudflare absorb most origin traffic.
const HTML_CACHE: &str = "public, max-age=300";

/// Hashed/static assets that never change between releases (favicon, OG image,
/// prebuilt CSS, `/images/*`). A year of immutable caching is the standard max
/// for HTTP/1.1 caches.
const IMMUTABLE_CACHE: &str = "public, max-age=31536000, immutable";

/// Crawl-control files (robots.txt, sitemap.xml): search engines must always
/// see the latest copy. Also the forced value on 4xx/5xx error responses so
/// that CDNs never pin a "not found" or server-error page as cacheable —
/// otherwise a later-added file would not be served until the cache expired.
const NO_CACHE: &str = "no-cache";

/// Permanent 308 redirects from `strip_trailing_slash`. A 24h edge cache lets
/// Cloudflare answer the redirect without re-hitting origin on every crawl.
/// Google's SEO guidance for permanent redirects recommends
/// `max-age >= 86400` so link equity transfer is not delayed by re-redirects.
const REDIRECT_CACHE: &str = "public, max-age=86400";

/// `Vary` value forcing edge caches to keep separate entries per combination
/// of the two request headers that drive locale negotiation on "/". Without
/// it a cached English "/" is served to a non-English user before the
/// `negotiate_locale` redirect can fire — the exact CDN edge-poisoning
/// regression class fixed in ADR-011 / PR #182.
const VARY_LOCALE: &str = "Cookie, Accept-Language";

/// Build the production Axum router.
///
/// Layering summary (request flows top → bottom; the LAST `.layer()` call is
/// the outermost, i.e. closest to the client):
///
/// 1. `security_headers` — outermost. Stamps `X-Content-Type-Options`,
///    `X-Frame-Options`, `Referrer-Policy` and `Permissions-Policy` on EVERY
///    response, including the 308 short-circuit from `strip_trailing_slash`
///    and the 404 from the fallback chain. Being outermost is what makes the
///    headers reach redirect/error responses — the inner middlewares
///    short-circuit without calling `next`, so a layer below them would never
///    see the response. See ADR-015.
/// 2. `strip_trailing_slash` — normalises `/ru/`, `/images/`, `/favicon.png/`
///    to their canonical slash-less form. 308 responses produced here carry
///    `REDIRECT_CACHE` directly and short-circuit the inner stack.
/// 3. `negotiate_locale` — only fires on `GET`/`HEAD "/"`: resolves the
///    visitor's preferred locale (saved `origa_locale` cookie, else
///    `Accept-Language`) and, when it is not English, emits a `307` to the
///    localised path with `Cache-Control: no-cache` + `Vary: Cookie,
///    Accept-Language`. English and unresolvable requests fall through.
/// 4. `enforce_cache_policy` — applies the cache-control contract to every
///    non-redirect response produced by the inner stack:
///    - 2xx without an explicit `Cache-Control` → stamped with `HTML_CACHE`
///      (the default for leptos HTML routes, which set no header themselves).
///    - 2xx with `Cache-Control` → left intact (static assets keep
///      `IMMUTABLE_CACHE`; robots/sitemap keep `NO_CACHE`).
///    - 2xx on "/" → additionally stamped with `Vary: Cookie, Accept-Language`,
///      because "/" is content-negotiated by `negotiate_locale` (see above);
///      without it a cached English "/" would be served to non-English
///      visitors before the redirect can fire (ADR-011 / PR #182).
///    - 4xx/5xx → overridden to `NO_CACHE`, regardless of what the inner
///      service set. Without this, `ServeDir`/`ServeFile` would stamp
///      `IMMUTABLE_CACHE` on a 404 via `insert_response_header_if_not_present`
///      (which fires on *all* statuses), and Cloudflare would cache the "not
///      found" for a year — a later-added file would not be served until the
///      cache expired.
///    - 3xx → passed through unchanged. The only 3xx from inner services is
///      `304 Not Modified` on conditional requests, which must keep the same
///      `Cache-Control` the corresponding 200 would have. `308`s from
///      `strip_trailing_slash` and `307`s from `negotiate_locale` never reach
///      this middleware (they are layers further out and short-circuit the
///      inner stack).
///
/// Fallback chain: when neither explicit static routes nor leptos' registered
/// routes match a path, the request hits `ServeDir(public/)`; if no file is
/// found there either, control falls through to `ErrorHandler`, which renders
/// the App via `leptos_router` (triggering its `<Routes fallback=NotFound>`
/// branch) and stamps HTTP 404 on the response if the App did not override
/// the status itself. This is what makes `/random` return a real 404 with the
/// visible "404" body instead of an empty shell.
///
/// Note on `308 Permanent Redirect`: `Redirect::permanent` in axum 0.8 emits
/// status 308, not 301. Both are treated as permanent by Googlebot/Yandex/Bing
/// and pass full link equity.
pub fn build_router(leptos_options: LeptosOptions) -> Router {
    let pkg_dir = env!("CARGO_MANIFEST_DIR");
    let routes = leptos_axum::generate_route_list(App);

    let public_dir = format!("{pkg_dir}/public");
    let error_handler = ErrorHandler::new(shell, leptos_options.clone());

    Router::new()
        .nest_service(
            "/images",
            ServeDir::new(format!("{public_dir}/images")).insert_response_header_if_not_present(
                CACHE_CONTROL,
                HeaderValue::from_static(IMMUTABLE_CACHE),
            ),
        )
        .route_service(
            "/landing.processed.css",
            ServeFile::new(format!("{pkg_dir}/style/landing.processed.css"))
                .insert_response_header_if_not_present(
                    CACHE_CONTROL,
                    HeaderValue::from_static(IMMUTABLE_CACHE),
                ),
        )
        .route_service(
            "/favicon.png",
            ServeFile::new(format!("{public_dir}/favicon.png"))
                .insert_response_header_if_not_present(
                    CACHE_CONTROL,
                    HeaderValue::from_static(IMMUTABLE_CACHE),
                ),
        )
        .route_service(
            "/favicon.ico",
            ServeFile::new(format!("{public_dir}/favicon.ico"))
                .insert_response_header_if_not_present(
                    CACHE_CONTROL,
                    HeaderValue::from_static(IMMUTABLE_CACHE),
                ),
        )
        .route_service(
            "/apple-touch-icon.png",
            ServeFile::new(format!("{public_dir}/apple-touch-icon.png"))
                .insert_response_header_if_not_present(
                    CACHE_CONTROL,
                    HeaderValue::from_static(IMMUTABLE_CACHE),
                ),
        )
        .route_service(
            "/browserconfig.xml",
            ServeFile::new(format!("{public_dir}/browserconfig.xml"))
                .insert_response_header_if_not_present(
                    CACHE_CONTROL,
                    HeaderValue::from_static(IMMUTABLE_CACHE),
                ),
        )
        .route_service(
            "/robots.txt",
            ServeFile::new(format!("{public_dir}/robots.txt"))
                .insert_response_header_if_not_present(
                    CACHE_CONTROL,
                    HeaderValue::from_static(NO_CACHE),
                ),
        )
        .route_service(
            "/sitemap.xml",
            ServeFile::new(format!("{public_dir}/sitemap.xml"))
                .insert_response_header_if_not_present(
                    CACHE_CONTROL,
                    HeaderValue::from_static(NO_CACHE),
                ),
        )
        .route_service(
            "/llms.txt",
            ServeFile::new(format!("{public_dir}/llms.txt")).insert_response_header_if_not_present(
                CACHE_CONTROL,
                HeaderValue::from_static(NO_CACHE),
            ),
        )
        // IndexNow key file: proves domain ownership to IndexNow-participating
        // search engines (Yandex, Bing, Naver, Seznam). See ADR-038.
        .route_service(
            "/e7825074-6888-4e03-a9ad-91459e4c9940.txt",
            ServeFile::new(format!(
                "{public_dir}/e7825074-6888-4e03-a9ad-91459e4c9940.txt"
            ))
            .insert_response_header_if_not_present(
                CACHE_CONTROL,
                HeaderValue::from_static(NO_CACHE),
            ),
        )
        .route(
            "/ru/blog/luchshee-prilozhenie-izucheniya-yaponskogo",
            axum::routing::get(redirect_old_ru_article_slug),
        )
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback_service(
            ServeDir::new(&public_dir)
                .append_index_html_on_directories(false)
                .fallback(error_handler),
        )
        // Inner cache-policy layer. Replaces the previous
        // `SetResponseHeaderLayer::if_not_present(HTML_CACHE)` with a function
        // that additionally overrides `Cache-Control` to `NO_CACHE` on 4xx/5xx.
        // See `enforce_cache_policy` below for the full rationale.
        .layer(middleware::from_fn(enforce_cache_policy))
        .layer(middleware::from_fn(negotiate_locale))
        .layer(middleware::from_fn(strip_trailing_slash))
        .layer(middleware::from_fn(security_headers))
        .with_state(leptos_options)
}

/// Cache-policy middleware applied to every non-redirect response.
///
/// Replaces the old outermost `SetResponseHeaderLayer::if_not_present(HTML)`
/// with status-aware logic. The per-service
/// `insert_response_header_if_not_present` calls on `ServeDir`/`ServeFile`
/// remain in place — they stamp `IMMUTABLE_CACHE`/`NO_CACHE` on successful
/// responses — but they fire on *all* statuses including 404, so this
/// middleware post-processes the result:
///
/// - 4xx/5xx → forced to `NO_CACHE`. This is the fix for the SEO issue where
///   `/images/nonexistent.png` returned 404 with `IMMUTABLE_CACHE`, causing
///   Cloudflare to cache "not found" for a year and block any later-added
///   file from being served until the cache expired.
/// - 2xx without `Cache-Control` → stamped with `HTML_CACHE` (the default for
///   leptos HTML routes, which do not set their own header).
/// - 2xx/3xx with `Cache-Control` already present → left intact, so static
///   assets keep `IMMUTABLE_CACHE`, robots/sitemap keep `NO_CACHE`, and
///   `304 Not Modified` preserves the same `Cache-Control` as the 200 would
///   have (required by HTTP conditional-cache semantics).
///
/// `308 Permanent Redirect` responses from `strip_trailing_slash` never reach
/// this middleware: they are produced by a layer *outside* this one, which
/// short-circuits the inner stack.
async fn enforce_cache_policy(request: Request<axum::body::Body>, next: Next) -> Response {
    let is_root = request.uri().path() == "/";
    let mut response = next.run(request).await;
    let status = response.status();

    if status.is_client_error() || status.is_server_error() {
        response
            .headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static(NO_CACHE));
    } else if status.is_success() {
        if !response.headers().contains_key(CACHE_CONTROL) {
            response
                .headers_mut()
                .insert(CACHE_CONTROL, HeaderValue::from_static(HTML_CACHE));
        }
        // The site root "/" is content-negotiated by `negotiate_locale`: the
        // same URL yields a 200 (English) or a 307 redirect depending on the
        // request's `Cookie`/`Accept-Language`. An edge cache MUST vary on
        // those headers, otherwise it serves a cached English 200 to a
        // non-English user instead of letting the redirect fire — the CDN
        // edge-poisoning regression class fixed in ADR-011 / PR #182. Every
        // other path is locale-determined by its URL prefix, so it needs no
        // `Vary`.
        if is_root {
            response
                .headers_mut()
                .insert(VARY, HeaderValue::from_static(VARY_LOCALE));
        }
    }

    response
}

/// Middleware: locale negotiation on the site root.
///
/// The root path "/" is the canonical English URL (no locale prefix). Rather
/// than force every visitor to find the language switcher, this middleware
/// resolves a preferred locale for a `GET`/`HEAD` to "/" and, when it is not
/// English, emits a temporary redirect to the matching localised path
/// (`/ru`, `/ko`, `/vi`). Resolution order: a saved `origa_locale` cookie
/// wins, otherwise the `Accept-Language` header. English, or requests where
/// no preference can be resolved, fall through to a normal English "/".
///
/// Redirects are `307` (temporary, method-preserving) and carry `Cache-Control: no-cache` plus
/// `Vary: Cookie, Accept-Language`, because the target depends on those
/// headers: a permanent or edge-cached redirect would pin one user's locale
/// for everyone (the ADR-011 / PR #182 edge-poisoning class). The cookie is
/// written client-side by the language switcher on every switch — including
/// to English — so a user on `/ko` who clicks "EN" lands on "/" and is not
/// bounced back.
///
/// `?lang=XX` query parameter: an explicit server-side locale override that
/// lets a visitor break out of an Accept-Language-driven redirect loop on the
/// very first visit (before any client-side cookie has been set). When a valid
/// locale is supplied, the server stamps `Set-Cookie` so subsequent visits are
/// served directly, then either falls through (English = root) or redirects to
/// the localised path (non-English). An invalid or absent value falls through
/// to the normal cookie / Accept-Language negotiation.
async fn negotiate_locale(
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    if (method != Method::GET && method != Method::HEAD) || uri.path() != "/" {
        return next.run(request).await;
    }

    // ?lang=XX: explicit server-side locale override. Lets a first-time
    // visitor with a non-English OS break out of the redirect loop by
    // stamping the locale cookie on the response before negotiation runs.
    if let Some(lang_locale) = query_locale_param(&uri) {
        let cookie = locale_cookie_value(lang_locale);
        if lang_locale == Locale::En {
            // English is the root URL — set the cookie and render directly.
            // The response carries `no-cache` because the served page depends
            // on the `?lang` query parameter: an edge cache must not pin a
            // `?lang=en` response and serve it to a visitor without the param.
            let mut response = next.run(request).await;
            response
                .headers_mut()
                .insert(CACHE_CONTROL, HeaderValue::from_static(NO_CACHE));
            response
                .headers_mut()
                .insert(VARY, HeaderValue::from_static(VARY_LOCALE));
            if let Ok(value) = HeaderValue::from_str(&cookie) {
                response.headers_mut().insert(SET_COOKIE, value);
            }
            return response;
        }
        // Non-English: redirect to the localised path and stamp the cookie so
        // the choice persists. The `?lang` param is not carried into the
        // redirect target — the cookie now records the preference.
        let mut response = Redirect::temporary(lang_locale.path_prefix()).into_response();
        response
            .headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static(NO_CACHE));
        response
            .headers_mut()
            .insert(VARY, HeaderValue::from_static(VARY_LOCALE));
        if let Ok(value) = HeaderValue::from_str(&cookie) {
            response.headers_mut().insert(SET_COOKIE, value);
        }
        return response;
    }

    let Some(locale) = preferred_locale(&headers) else {
        return next.run(request).await;
    };

    let location = match uri.query() {
        Some(query) if !query.is_empty() => format!("{}?{query}", locale.path_prefix()),
        _ => locale.path_prefix().to_string(),
    };
    let mut response = Redirect::temporary(&location).into_response();
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static(NO_CACHE));
    response
        .headers_mut()
        .insert(VARY, HeaderValue::from_static(VARY_LOCALE));
    response
}

/// Extract a valid locale from the `?lang=XX` query parameter on a request to
/// "/". Returns `None` when the parameter is absent or its value is not one of
/// the supported locales (en/ru/ko/vi), so the caller can fall through to
/// normal cookie / Accept-Language negotiation.
fn query_locale_param(uri: &Uri) -> Option<Locale> {
    let query = uri.query()?;
    for pair in query.split('&') {
        let Some(value) = pair.strip_prefix("lang=") else {
            continue;
        };
        return Locale::from_str(value).ok();
    }
    None
}

/// Build the `Set-Cookie` header value for persisting a locale choice. The
/// resulting string is always ASCII-safe (constant name + known locale code +
/// numeric max-age), so `HeaderValue::from_str` on it cannot fail in practice.
fn locale_cookie_value(locale: Locale) -> String {
    format!(
        "{}={}; path=/; max-age={}; SameSite=Lax",
        LOCALE_COOKIE,
        locale.as_str(),
        LOCALE_COOKIE_MAX_AGE_SECS,
    )
}

/// Resolve the preferred non-English locale for a request to "/": an explicit
/// `origa_locale` cookie takes precedence over `Accept-Language`; if no cookie
/// is present, `Accept-Language` is consulted. Returns `None` for English (the
/// default root) so the caller can fall through to a normal 200.
///
/// Crucially, an explicit English cookie must win over a non-English
/// `Accept-Language`: a visitor who switched to English on a localised path
/// must land on "/" and not be bounced back by their browser's language.
/// That requires distinguishing "no cookie" (fall back to `Accept-Language`)
/// from "cookie = en" (honour the choice, no redirect).
fn preferred_locale(headers: &HeaderMap) -> Option<Locale> {
    if let Some(locale) = cookie_locale(headers) {
        // An explicit cookie wins entirely, including English (no redirect).
        return (locale != Locale::En).then_some(locale);
    }
    let accept = headers.get(ACCEPT_LANGUAGE)?.to_str().ok()?;
    locale_from_accept_language(accept)
}

/// Read the explicit locale from the `origa_locale` cookie, if present and
/// valid. Returns the locale **including English** — `None` means the cookie
/// is absent or malformed, signalling the caller to fall back to
/// `Accept-Language`. Conflating "absent" with "English" would let a
/// non-English browser override an explicit English choice.
fn cookie_locale(headers: &HeaderMap) -> Option<Locale> {
    let cookie = headers.get(COOKIE)?.to_str().ok()?;
    for pair in cookie.split(';') {
        let pair = pair.trim();
        let Some(value) = pair
            .strip_prefix(LOCALE_COOKIE)
            .and_then(|rest| rest.strip_prefix('='))
        else {
            continue;
        };
        if let Ok(locale) = Locale::from_str(value) {
            return Some(locale);
        }
    }
    None
}

fn locale_from_accept_language(header: &str) -> Option<Locale> {
    // Parse Accept-Language per RFC 7231 §5.3.5, e.g.
    // "en-US,en;q=0.9,ru;q=0.8". Each entry becomes (primary subtag, q). Tags
    // with q=0 ("not acceptable") are dropped. The entries are then sorted by
    // q descending — stable, so ties keep the header's listed order — and the
    // first supported locale (en/ru/ko/vi) wins. English resolves to `None`
    // (the root URL); a non-English locale becomes a redirect target.
    //
    // Q-values matter: an English-primary bilingual ("en;q=0.9,ru;q=0.8") must
    // NOT be redirected to Russian. A naive "first supported non-English tag"
    // scan would send them there.
    let mut entries: Vec<(String, f32)> = header
        .split(',')
        .filter_map(|raw| {
            let mut parts = raw.split(';');
            let tag = parts.next()?.trim().to_lowercase();
            if tag.is_empty() {
                return None;
            }
            let primary = tag.split('-').next()?.to_string();
            let q = parts
                .next()
                .and_then(|param| param.trim().strip_prefix("q="))
                .and_then(|v| v.trim().parse::<f32>().ok())
                .unwrap_or(1.0);
            if q <= 0.0 {
                return None;
            }
            Some((primary, q))
        })
        .collect();
    entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (primary, _) in &entries {
        match primary.as_str() {
            "en" => return None,
            "ru" => return Some(Locale::Ru),
            "ko" => return Some(Locale::Ko),
            "vi" => return Some(Locale::Vi),
            _ => {},
        }
    }
    None
}

/// Middleware: redirect `/path/` to `/path` with HTTP 308 Permanent Redirect.
///
/// Canonical URLs for `origa_landing` have no trailing slash (e.g. `/ru`,
/// `/ru/features`). The site root `/` is the sole exception and keeps its
/// slash by convention.
///
/// - GET/HEAD only: other methods are passed through verbatim so the redirect
///   cannot mask an accidental POST to a slash-suffixed URL. This mirrors
///   RFC 7231 §6.4 redirect semantics.
/// - Query string is preserved (relative `Location`).
/// - Multiple trailing slashes collapse to one canonical form, e.g.
///   `/ru//` → `/ru`.
/// - The 308 response carries `Cache-Control: public, max-age=86400`
///   (`REDIRECT_CACHE`) so edge caches serve the redirect without re-hitting
///   origin on every crawl. This layer is the outermost one and short-circuits
///   the inner stack, so `enforce_cache_policy` cannot apply the header — it
///   must be set here directly.
async fn strip_trailing_slash(
    method: Method,
    uri: Uri,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    if method != Method::GET && method != Method::HEAD {
        return next.run(request).await;
    }
    let path = uri.path();
    if path.len() <= 1 || !path.ends_with('/') {
        return next.run(request).await;
    }
    let stripped = path.trim_end_matches('/');
    let stripped = if stripped.is_empty() { "/" } else { stripped };
    let location = match uri.query() {
        Some(q) if !q.is_empty() => format!("{stripped}?{q}"),
        _ => stripped.to_string(),
    };
    let mut response = Redirect::permanent(&location).into_response();
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static(REDIRECT_CACHE));
    response
}

/// Outermost security-headers middleware. Stamps four defensive headers on
/// every response the server emits — HTML pages, static assets, the 308
/// redirect from `strip_trailing_slash`, and the 404 from the fallback chain.
///
/// Being outermost is load-bearing: `strip_trailing_slash` short-circuits 308
/// responses without calling `next`, so a layer further in would never see
/// them and the redirect would ship without the headers. See ADR-015 for the
/// header-by-header rationale and the CSP trade-off.
///
/// `Permissions-Policy` locks down the three device capabilities Origa never
/// uses (camera, microphone, geolocation); any future feature needing one
/// must narrow this allowlist explicitly rather than widening it blindly.
async fn security_headers(request: Request<axum::body::Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("SAMEORIGIN"),
    );
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    response
}

/// Permanent 308 redirect from the legacy RU article slug
/// (`luchshee-prilozhenie-izucheniya-yaponskogo`) to the unified
/// locale-independent slug (`best-japanese-learning-app`). The rename unifies
/// all four locale translations of the article under one URL stem so that
/// `hreflang` alternates point at a single canonical article group; without
/// it, `/ru/blog/<ru-slug>` and `/blog/<en-slug>` would advertise different
/// articles that happen to share a translation.
///
/// The `Cache-Control: public, max-age=86400` header matches the cache
/// policy on `strip_trailing_slash`'s 308 responses — edge caches should
/// serve the redirect without re-hitting origin. `enforce_cache_policy`
/// passes 3xx through unchanged, so the header must be set on the response
/// itself rather than via middleware.
async fn redirect_old_ru_article_slug() -> Response {
    let mut response = Redirect::permanent("/ru/blog/best-japanese-learning-app").into_response();
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static(REDIRECT_CACHE));
    response
}
