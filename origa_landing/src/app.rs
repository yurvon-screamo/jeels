use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use crate::components::{Layout, NotFound};
use crate::content::Locale;
use crate::pages::*;

pub fn shell(_options: LeptosOptions) -> impl IntoView {
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
                <link rel="preconnect" href="https://fonts.googleapis.com" />
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
                <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Cormorant+Garamond:ital,wght@0,300;0,400;0,500;0,600;0,700;1,400&family=DM+Mono:ital,wght@0,300;0,400;0,500;1,400&display=swap" />
                <meta name="yandex-verification" content="95bbd9366a113be4" />
                <meta name="google-site-verification" content="8HXC9phyHedz5AeimJ12tIo7HtXXHrnm2ewE4Qm3zEw" />
                <meta name="msvalidate.01" content="36F67711155024DF2B7F9B5EBF72E9D0" />
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
