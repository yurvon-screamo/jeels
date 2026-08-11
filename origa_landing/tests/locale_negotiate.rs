//! Integration tests for the `negotiate_locale` middleware.
//!
//! Behaviour under test: a `GET`/`HEAD "/"` is redirected to the visitor's
//! preferred locale (`/ru`, `/ko`, `/vi`) when that locale is not English.
//! Preference comes from a saved `origa_locale` cookie, falling back to the
//! `Accept-Language` header. English and unresolvable requests fall through to
//! a normal 200. Redirects are temporary (`307`, method-preserving), non-cacheable, and carry
//! `Vary: Cookie, Accept-Language` so an edge cache never serves one user's
//! locale answer to another (ADR-011 / PR #182).

#![cfg(feature = "ssr")]

use axum::body::Body;
use http::header::{ACCEPT_LANGUAGE, CACHE_CONTROL, COOKIE, LOCATION, SET_COOKIE, VARY};
use http::{HeaderMap, HeaderValue, Method, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

mod common;

/// Run `method uri` through the production router, optionally stamping an
/// `Accept-Language` and/or `Cookie` header, and return the status plus a
/// cloned copy of the response headers.
async fn request(
    uri: &str,
    method: Method,
    accept_language: Option<&str>,
    cookie: Option<&str>,
) -> (StatusCode, HeaderMap) {
    let mut builder = http::Request::builder().method(method).uri(uri);
    if let Some(value) = accept_language {
        builder = builder.header(ACCEPT_LANGUAGE, value);
    }
    if let Some(value) = cookie {
        builder = builder.header(COOKIE, value);
    }

    let response = common::test_router()
        .oneshot(builder.body(Body::empty()).expect("valid request"))
        .await
        .expect("router responded");

    let status = response.status();
    let headers = response.headers().clone();
    let _ = response.into_body().collect().await.expect("body");
    (status, headers)
}

fn header_value(headers: &HeaderMap, name: &http::HeaderName) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}

#[rstest::rstest]
#[case::ru("ru", "/ru")]
#[case::ko("ko", "/ko")]
#[case::vi("vi", "/vi")]
#[case::ru_region("ru-RU,ru;q=0.9,en;q=0.8", "/ru")]
#[tokio::test]
async fn accept_language_redirects_to_localised_root(
    #[case] header: &str,
    #[case] expected_location: &str,
) {
    let (status, headers) = request("/", Method::GET, Some(header), None).await;

    assert_eq!(
        status,
        StatusCode::TEMPORARY_REDIRECT,
        "headers={:?}",
        headers
    );
    assert_eq!(
        header_value(&headers, &LOCATION).as_deref(),
        Some(expected_location)
    );
    assert_eq!(
        header_value(&headers, &CACHE_CONTROL).as_deref(),
        Some("no-cache")
    );
    assert_eq!(
        header_value(&headers, &VARY).as_deref(),
        Some("Cookie, Accept-Language")
    );
}

#[rstest::rstest]
#[case::ru("ru", "/ru")]
#[case::ko("ko", "/ko")]
#[case::vi("vi", "/vi")]
#[tokio::test]
async fn saved_cookie_redirects_to_localised_root(
    #[case] value: &str,
    #[case] expected_location: &str,
) {
    let (status, headers) = request(
        "/",
        Method::GET,
        None,
        Some(&format!("origa_locale={value}")),
    )
    .await;

    assert_eq!(status, StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(
        header_value(&headers, &LOCATION).as_deref(),
        Some(expected_location)
    );
    assert_eq!(
        header_value(&headers, &CACHE_CONTROL).as_deref(),
        Some("no-cache")
    );
}

#[rstest::rstest]
#[case::en("en")]
#[case::en_region("en-US,en;q=0.9")]
#[case::en_primary_with_ru_secondary("en-US,en;q=0.9,ru;q=0.8")]
#[case::unsupported_ja("ja,en;q=0.5")]
#[case::ru_not_acceptable("ru;q=0,en")]
#[case::empty("")]
#[tokio::test]
async fn english_or_unresolvable_falls_through_to_200(#[case] header: &str) {
    let (status, headers) = request("/", Method::GET, Some(header), None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(header_value(&headers, &LOCATION).is_none());
    assert_eq!(
        header_value(&headers, &CACHE_CONTROL).as_deref(),
        Some("public, max-age=300")
    );
    assert_eq!(
        header_value(&headers, &VARY).as_deref(),
        Some("Cookie, Accept-Language")
    );
}

#[tokio::test]
async fn higher_q_value_wins_between_supported_locales() {
    // When two supported non-English locales are listed, the one with the
    // higher q-value wins (ru;q=0.5 vs ko;q=0.9 -> Korean).
    let (status, headers) = request("/", Method::GET, Some("ru;q=0.5,ko;q=0.9"), None).await;

    assert_eq!(status, StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(header_value(&headers, &LOCATION).as_deref(), Some("/ko"));
}

#[tokio::test]
async fn bare_root_without_preferences_falls_through_to_200() {
    let (status, headers) = request("/", Method::GET, None, None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(header_value(&headers, &LOCATION).is_none());
}

#[tokio::test]
async fn english_cookie_does_not_redirect() {
    // English is the root URL, not a redirect target. A user who switched to
    // English (the switcher writes origa_locale=en) must land on "/" and not
    // be bounced by their own cookie.
    let (status, headers) = request("/", Method::GET, None, Some("origa_locale=en")).await;

    assert_eq!(status, StatusCode::OK);
    assert!(header_value(&headers, &LOCATION).is_none());
}

#[tokio::test]
async fn english_cookie_overrides_non_english_accept_language() {
    // Regression: an explicit English choice must beat the browser's
    // Accept-Language. A visitor on a localised path who clicks "EN" lands on
    // "/" carrying both `origa_locale=en` and a non-English Accept-Language;
    // the explicit cookie must win, otherwise they can never reach English.
    let (status, headers) = request("/", Method::GET, Some("ru"), Some("origa_locale=en")).await;

    assert_eq!(status, StatusCode::OK);
    assert!(header_value(&headers, &LOCATION).is_none());
}

#[tokio::test]
async fn cookie_takes_precedence_over_accept_language() {
    // A user whose browser is Russian but who previously chose Korean must
    // land on Korean: the explicit cookie wins over Accept-Language.
    let (status, headers) = request(
        "/",
        Method::GET,
        Some("ru-RU,ru;q=0.9"),
        Some("origa_locale=ko"),
    )
    .await;

    assert_eq!(status, StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(header_value(&headers, &LOCATION).as_deref(), Some("/ko"));
}

#[tokio::test]
async fn redirect_preserves_query_string() {
    let (status, headers) = request("/?ref=twitter", Method::GET, Some("ru"), None).await;

    assert_eq!(status, StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(
        header_value(&headers, &LOCATION).as_deref(),
        Some("/ru?ref=twitter")
    );
}

#[tokio::test]
async fn head_mirrors_get_redirect() {
    // HEAD must mirror GET redirect semantics per RFC 7231 §6.4.
    let (status, headers) = request("/", Method::HEAD, Some("ru"), None).await;

    assert_eq!(status, StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(header_value(&headers, &LOCATION).as_deref(), Some("/ru"));
}

#[tokio::test]
async fn post_is_not_redirected() {
    // Only safe methods (GET/HEAD) negotiate locale; a POST to "/" must reach
    // the route layer rather than be turned into a locale redirect.
    let (status, headers) = request("/", Method::POST, Some("ru"), None).await;

    assert_ne!(status, StatusCode::TEMPORARY_REDIRECT);
    assert!(header_value(&headers, &LOCATION).is_none());
}

#[tokio::test]
async fn non_root_path_is_not_redirected() {
    // Locale negotiation only fires on the root; a localised or unprefixed
    // subpath already carries its locale in the URL.
    let (status, headers) = request("/features", Method::GET, Some("ru"), None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(header_value(&headers, &LOCATION).is_none());
}

#[tokio::test]
async fn locale_redirect_carries_security_headers() {
    // The negotiate_locale layer sits inside `security_headers`, so its 307
    // must still receive the defensive headers stamped by the outermost layer.
    let (_status, headers) = request("/", Method::GET, Some("ru"), None).await;

    assert_eq!(
        headers.get("x-content-type-options"),
        Some(&HeaderValue::from_static("nosniff"))
    );
    assert_eq!(
        headers.get("x-frame-options"),
        Some(&HeaderValue::from_static("SAMEORIGIN"))
    );
}

#[tokio::test]
async fn invalid_cookie_value_is_ignored() {
    // A malformed cookie must not 500 or redirect to a nonsense path; it is
    // dropped and Accept-Language (if any) gets a chance to resolve.
    let (status, headers) = request(
        "/",
        Method::GET,
        Some("en"),
        Some("origa_locale=zzz;q=broken"),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(header_value(&headers, &LOCATION).is_none());
}

#[tokio::test]
async fn other_cookies_do_not_trigger_redirect() {
    // Only `origa_locale` is consulted; an unrelated cookie must not negotiate.
    let (status, headers) = request(
        "/",
        Method::GET,
        Some("en"),
        Some("_ga=GA1.2.x; theme=dark"),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(header_value(&headers, &LOCATION).is_none());
}

#[tokio::test]
async fn locale_cookie_among_other_cookies_is_found() {
    // Regression: the cookie parser must iterate past unrelated cookies rather
    // than return early on the first non-matching pair.
    let (status, headers) = request(
        "/",
        Method::GET,
        Some("en"),
        Some("_ga=GA1.2.x; origa_locale=vi; theme=dark"),
    )
    .await;

    assert_eq!(status, StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(header_value(&headers, &LOCATION).as_deref(), Some("/vi"));
}

#[tokio::test]
async fn lang_query_param_sets_cookie_and_does_not_redirect() {
    // A first-time visitor with a Russian OS (Accept-Language: ru) can force
    // the English landing page by visiting "/?lang=en". The server stamps the
    // `origa_locale=en` cookie so subsequent visits are served directly, and
    // renders the page immediately (200) instead of redirecting to /ru.
    let (status, headers) = request("/?lang=en", Method::GET, Some("ru"), None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        header_value(&headers, &CACHE_CONTROL).as_deref(),
        Some("no-cache"),
        "?lang=en must not be edge-cacheable"
    );
    assert!(
        header_value(&headers, &LOCATION).is_none(),
        "no redirect expected, got Location={:?}",
        header_value(&headers, &LOCATION)
    );
    let set_cookie = header_value(&headers, &SET_COOKIE).expect("Set-Cookie header present");
    assert!(
        set_cookie.contains("origa_locale=en"),
        "Set-Cookie must stamp origa_locale=en, got {set_cookie}"
    );
    assert!(
        set_cookie.contains("path=/"),
        "Set-Cookie must be scoped to root path, got {set_cookie}"
    );
    assert!(
        set_cookie.contains("max-age=31536000"),
        "Set-Cookie must persist for one year, got {set_cookie}"
    );
    assert!(
        set_cookie.contains("SameSite=Lax"),
        "Set-Cookie must use SameSite=Lax, got {set_cookie}"
    );
}

#[tokio::test]
async fn lang_query_param_ru_sets_cookie() {
    // A non-English ?lang value both sets the cookie AND redirects, so the
    // visitor lands on the localised path and the preference persists. This
    // makes `?lang=ru` a shareable deep link into the Russian site.
    let (status, headers) = request("/?lang=ru", Method::GET, None, None).await;

    assert_eq!(status, StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(header_value(&headers, &LOCATION).as_deref(), Some("/ru"));
    let set_cookie = header_value(&headers, &SET_COOKIE).expect("Set-Cookie header present");
    assert!(
        set_cookie.contains("origa_locale=ru"),
        "Set-Cookie must stamp origa_locale=ru, got {set_cookie}"
    );
}

#[tokio::test]
async fn invalid_lang_param_ignored() {
    // An unrecognised ?lang value (not en/ru/ko/vi) must not 500, set a bogus
    // cookie, or otherwise alter behaviour — it falls through to the normal
    // cookie / Accept-Language negotiation exactly as if the param were absent.
    let (status, headers) = request("/?lang=xyz", Method::GET, Some("ru"), None).await;

    assert_eq!(status, StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(header_value(&headers, &LOCATION).as_deref(), Some("/ru"));
    assert!(
        header_value(&headers, &SET_COOKIE).is_none(),
        "invalid lang must not stamp a cookie"
    );
}
