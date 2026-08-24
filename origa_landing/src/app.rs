use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use crate::components::{Layout, NotFound};
use crate::content::Locale;
use crate::pages::*;

pub fn shell(_options: LeptosOptions) -> impl IntoView {
    // Compile-time counter ID (set via ORIGA_YANDEX_METRIKA_ID, see
    // `build.rs`). Empty = analytics fully absent from the HTML.
    const METRIKA_ID: &str = env!("ORIGA_YANDEX_METRIKA_ID");
    view! {
        <!DOCTYPE html>
        <html>
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <meta name="theme-color" content="#3d4535" />
                <meta name="msapplication-TileColor" content="#3d4535" />
                <meta name="msapplication-config" content="/browserconfig.xml" />
                <link rel="icon" href="/favicon.ico" sizes="16x16 32x32 48x48" />
                <link rel="icon" type="image/png" href="/favicon.png" />
                <link rel="apple-touch-icon" href="/apple-touch-icon.png" />
                // Fonts are self-hosted on the project CDN (@font-face rules in
                // style/input.css); preload only the hero-critical serif faces.
                // Cross-origin font preloads REQUIRE the crossorigin attribute
                // even when anonymous, or browsers re-fetch and double-load.
                <link rel="preconnect" href="https://s3.origa.uwuwu.net" crossorigin="anonymous" />
                <link
                    rel="preload"
                    attr:as="font"
                    type="font/woff2"
                    crossorigin="anonymous"
                    href="https://s3.origa.uwuwu.net/fonts/landing/cormorant-garamond-v21-cyrillic_latin-300.woff2"
                />
                <link
                    rel="preload"
                    attr:as="font"
                    type="font/woff2"
                    crossorigin="anonymous"
                    href="https://s3.origa.uwuwu.net/fonts/landing/dm-mono-v16-latin_latin-ext-regular.woff2"
                />
                <meta name="yandex-verification" content="95bbd9366a113be4" />
                <meta name="google-site-verification" content="8HXC9phyHedz5AeimJ12tIo7HtXXHrnm2ewE4Qm3zEw" />
                <meta name="msvalidate.01" content="36F67711155024DF2B7F9B5EBF72E9D0" />
                {if METRIKA_ID.is_empty() {
                    ().into_any()
                } else {
                    let snippet = metrika_inline_script(METRIKA_ID);
                    view! { <script inner_html=snippet /> }.into_any()
                }}
                <MetaTags />
            </head>
            <body class="min-h-screen paper-texture">
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/landing.processed.css" />
        <Title text="Origa — Japanese Learning App" />
        <Router>
            <Routes fallback=NotFound>
                <ParentRoute path=path!("") view=move || view! { <Layout locale=Locale::En /> }>
                    <Route path=path!("") view=HomePage />
                    <Route path=path!("features") view=FeaturesPage />
                    <Route path=path!("compare") view=ComparePage />
                    <Route path=path!("content") view=IntegrationsPage />
                    <Route path=path!("download") view=DownloadPage />
                    <Route path=path!("privacy") view=PrivacyPage />
                    <Route path=path!("terms") view=TermsPage />
                    <Route path=path!("blog") view=BlogIndexPage />
                    <Route path=path!("blog/:slug") view=BlogPostPage />
                    <Route path=path!("docs") view=DocsIndexPage />
                    <Route path=path!("docs/:slug") view=DocsArticlePage />
                </ParentRoute>
                <ParentRoute path=path!("ru") view=move || view! { <Layout locale=Locale::Ru /> }>
                    <Route path=path!("") view=HomePage />
                    <Route path=path!("features") view=FeaturesPage />
                    <Route path=path!("compare") view=ComparePage />
                    <Route path=path!("content") view=IntegrationsPage />
                    <Route path=path!("download") view=DownloadPage />
                    <Route path=path!("privacy") view=PrivacyPage />
                    <Route path=path!("terms") view=TermsPage />
                    <Route path=path!("blog") view=BlogIndexPage />
                    <Route path=path!("blog/:slug") view=BlogPostPage />
                    <Route path=path!("docs") view=DocsIndexPage />
                    <Route path=path!("docs/:slug") view=DocsArticlePage />
                </ParentRoute>
                <ParentRoute path=path!("ko") view=move || view! { <Layout locale=Locale::Ko /> }>
                    <Route path=path!("") view=HomePage />
                    <Route path=path!("features") view=FeaturesPage />
                    <Route path=path!("compare") view=ComparePage />
                    <Route path=path!("content") view=IntegrationsPage />
                    <Route path=path!("download") view=DownloadPage />
                    <Route path=path!("privacy") view=PrivacyPage />
                    <Route path=path!("terms") view=TermsPage />
                    <Route path=path!("blog") view=BlogIndexPage />
                    <Route path=path!("blog/:slug") view=BlogPostPage />
                    <Route path=path!("docs") view=DocsIndexPage />
                    <Route path=path!("docs/:slug") view=DocsArticlePage />
                </ParentRoute>
                <ParentRoute path=path!("vi") view=move || view! { <Layout locale=Locale::Vi /> }>
                    <Route path=path!("") view=HomePage />
                    <Route path=path!("features") view=FeaturesPage />
                    <Route path=path!("compare") view=ComparePage />
                    <Route path=path!("content") view=IntegrationsPage />
                    <Route path=path!("download") view=DownloadPage />
                    <Route path=path!("privacy") view=PrivacyPage />
                    <Route path=path!("terms") view=TermsPage />
                    <Route path=path!("blog") view=BlogIndexPage />
                    <Route path=path!("blog/:slug") view=BlogPostPage />
                    <Route path=path!("docs") view=DocsIndexPage />
                    <Route path=path!("docs/:slug") view=DocsArticlePage />
                </ParentRoute>
            </Routes>
        </Router>
    }
}

/// Standard Yandex.Metrika async snippet for `counter_id`. The webvisor and
/// click-map options stay off to keep the data collection within what the
/// privacy policy discloses. `defer`-style loading: the script tag is
/// injected by the snippet itself, so first paint is never blocked.
fn metrika_inline_script(counter_id: &str) -> String {
    format!(
        r#"
            (function(m,e,t,r,i,k,a){{
                m[i]=m[i]||function(){{(m[i].a=m[i].a||[]).push(arguments)}};
                m[i].l=1*new Date();
                for(var j=0;j<document.scripts.length;j++){{if(document.scripts[j].src===r){{return;}}}}
                k=e.createElement(t),a=e.getElementsByTagName(t)[0],k.async=1,k.src=r,a.parentNode.insertBefore(k,a)
            }})
            (window, document, "script", "https://mc.yandex.ru/metrika/tag.js", "ym");

            ym({counter_id}, "init", {{
                clickmap:false,
                trackLinks:true,
                accurateTrackBounce:true,
                webvisor:false
            }});
        "#
    )
}
