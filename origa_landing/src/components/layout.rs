use leptos::prelude::*;
use leptos_meta::Html;
use leptos_router::components::{A, Outlet};
use leptos_router::hooks::use_location;

use crate::content::{LOCALE_COOKIE, LOCALE_COOKIE_MAX_AGE_SECS, Locale};

fn make_href(prefix: &str, page: &str) -> String {
    format!("{prefix}/{page}")
}

#[component]
pub fn Layout(locale: Locale) -> impl IntoView {
    provide_context(locale);

    // The WIP banner ("KO/VI language support is under development") tells
    // visitors that the *site* UI for their locale is incomplete. On an
    // article URL served as an EN fallback (e.g. `/ko/blog/<en-slug>`), the
    // banner would read "Korean support is under development" above an
    // English article — a contradiction. The in-article locale marker
    // ("Showing English article · KO") already discloses the fallback, so
    // suppress the site-level banner there. On `/blog` index pages, no
    // fallback happens (strict locale filter), so the banner stays.
    let path = use_location().pathname;
    let is_blog_article_route = {
        let p = path.get();
        let mut segments = p.split('/').filter(|s| !s.is_empty());
        segments.next() == Some("blog") && segments.next().is_some()
    };

    let c = locale.content();
    let prefix = locale.path_prefix();

    let home_href = if prefix.is_empty() {
        "/".to_string()
    } else {
        prefix.to_string()
    };

    let lang_switcher_items: Vec<_> = Locale::ALL
        .iter()
        .filter(|loc| **loc != locale)
        .map(|loc| {
            let href = if loc.path_prefix().is_empty() {
                "/".to_string()
            } else {
                loc.path_prefix().to_string()
            };
            (*loc, loc.display_label(), href, loc.as_str())
        })
        .collect();

    view! {
        <Html {..} lang=locale.as_str() />
        <header class="landing-header">
            <a href=home_href class="landing-header__logo">
                <img src="/images/logo.png" alt="Origa" class="landing-header__logo-img" />
                <span class="landing-header__logo-name">"Origa"</span>
                <span class="landing-header__logo-kana">"オリガ"</span>
            </a>
            <button type="button" class="landing-header__hamburger"
                    aria-label="Open menu" aria-expanded="false">
                <span class="landing-header__hamburger-line"></span>
            </button>
            <nav class="landing-header__nav" id="main-nav"
                 aria-label="Main navigation">
                <NavLink prefix href="features" class="landing-header__link">
                    {c.header_features}
                </NavLink>
                <NavLink prefix href="compare" class="landing-header__link">
                    {c.header_compare}
                </NavLink>
                <NavLink prefix href="content" class="landing-header__link">
                    {c.header_integrations}
                </NavLink>
                <NavLink prefix href="blog" class="landing-header__link">
                    {c.header_blog}
                </NavLink>
                <NavLink prefix href="docs" class="landing-header__link">
                    {c.header_docs}
                </NavLink>
                <NavLink prefix href="download" class="landing-header__link">
                    {c.header_download}
                </NavLink>
                <span class="landing-header__nav-sep">"|"</span>
                <span class="landing-header__lang">
                    <span class="landing-header__lang-current">{locale.display_label()}</span>
                    {lang_switcher_items.into_iter().flat_map(|(_, label, href, code)| {
                        vec![
                            view! { <span class="landing-header__lang-sep">" · "</span> }.into_any(),
                            view! {
                                <a href=href class="landing-header__lang-link" attr:data-locale=code>
                                    {label}
                                </a>
                            }
                                .into_any(),
                        ]
                    }).collect_view()}
                </span>
            </nav>
        </header>
        <script inner_html=header_inline_script() />
        {if locale.is_development() && !is_blog_article_route {
            view! {
                <div class="landing-wip-banner">{c.banner_wip}</div>
            }
                .into_any()
        } else {
            ().into_any()
        }}
        <main>
            <Outlet />
        </main>
        <footer class="landing-footer">
            <div class="landing-footer__grid">
                <div>
                    <p class="landing-footer__heading">{c.footer_product}</p>
                    <NavLink prefix href="features" class="landing-footer__link">
                        {c.header_features}
                    </NavLink>
                    <NavLink prefix href="compare" class="landing-footer__link">
                        {c.header_compare}
                    </NavLink>
                    <NavLink prefix href="content" class="landing-footer__link">
                        {c.header_integrations}
                    </NavLink>
                    <NavLink prefix href="blog" class="landing-footer__link">
                        {c.header_blog}
                    </NavLink>
                    <NavLink prefix href="download" class="landing-footer__link">
                        {c.header_download}
                    </NavLink>
                </div>
                <div>
                    <p class="landing-footer__heading">{c.footer_resources}</p>
                    <NavLink prefix href="docs" class="landing-footer__link">
                        {c.header_docs}
                    </NavLink>
                    <a
                        href="https://github.com/yurvon-screamo/origa"
                        class="landing-footer__link"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        "GitHub"
                    </a>
                    <a
                        href="https://github.com/yurvon-screamo/origa/blob/master/LICENSE"
                        class="landing-footer__link"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        {c.footer_license}
                    </a>
                </div>
                <div>
                    <p class="landing-footer__heading">{c.footer_legal}</p>
                    <NavLink prefix href="privacy" class="landing-footer__link">
                        {c.legal_privacy_link}
                    </NavLink>
                    <NavLink prefix href="terms" class="landing-footer__link">
                        {c.legal_terms_link}
                    </NavLink>
                </div>
            </div>
        </footer>
    }
}

#[component]
fn NavLink(
    prefix: &'static str,
    href: &'static str,
    class: &'static str,
    children: Children,
) -> impl IntoView {
    let target = make_href(prefix, href);
    view! {
        <A href=target attr:class=class>{children()}</A>
    }
}

/// Inline script for the header: toggles the mobile nav and persists the
/// visitor's language choice. Built with `format!` so the cookie name and
/// max-age come from the same constants the `negotiate_locale` middleware
/// reads, instead of a second hardcoded copy that could drift.
fn header_inline_script() -> String {
    format!(
        r#"
            (function() {{
                var btn = document.querySelector('.landing-header__hamburger');
                var nav = document.getElementById('main-nav');
                if (!btn || !nav) return;
                btn.addEventListener('click', function() {{
                    var open = nav.classList.toggle('is-open');
                    btn.classList.toggle('is-open');
                    btn.setAttribute('aria-expanded', open ? 'true' : 'false');
                    btn.setAttribute('aria-label', open ? 'Close menu' : 'Open menu');
                }});
                document.addEventListener('keydown', function(e) {{
                    if (e.key === 'Escape' && nav.classList.contains('is-open')) {{
                        nav.classList.remove('is-open');
                        btn.classList.remove('is-open');
                        btn.setAttribute('aria-expanded', 'false');
                        btn.setAttribute('aria-label', 'Open menu');
                    }}
                }});
                // Persist the language choice so the locale-negotiation
                // middleware on "/" redirects returning visitors to their
                // locale. Writing the cookie on click (including for English)
                // before navigation is what prevents a user on a localised
                // path from being bounced back when they switch to English.
                document.querySelectorAll('.landing-header__lang-link').forEach(function(a) {{
                    a.addEventListener('click', function() {{
                        document.cookie = '{LOCALE_COOKIE}=' + a.getAttribute('data-locale')
                            + '; path=/; max-age={LOCALE_COOKIE_MAX_AGE_SECS}; SameSite=Lax';
                    }});
                }});
            }})();
        "#
    )
}
