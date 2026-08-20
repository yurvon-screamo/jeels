use leptos::prelude::*;

use crate::components::seo::{PageMeta, SchemaOrg, breadcrumb_schema};
use crate::content::Locale;

const EFFECTIVE_DATE: &str = "2026-08-20";

#[component]
pub fn PrivacyPage() -> impl IntoView {
    let locale = use_context::<Locale>().expect("Locale context missing");
    let c = locale.content();

    view! {
        <PageMeta locale path="/privacy" title=c.privacy_meta_title description=c.privacy_meta_description/>
        <SchemaOrg json=breadcrumb_schema(locale, &[
            (c.breadcrumb_home, "/"),
            (c.legal_privacy_link, "/privacy"),
        ])/>

        <article class="legal-doc">
            <header class="legal-doc__header">
                <h1 class="legal-doc__title">{c.privacy_h1}</h1>
                <p class="legal-doc__updated">
                    {c.privacy_last_updated_label} " " {EFFECTIVE_DATE}
                </p>
            </header>
            <div class="legal-doc__body" inner_html=c.privacy_body></div>
        </article>
    }
}
