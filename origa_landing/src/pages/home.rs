use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::cta::CtaSection;
use crate::components::seo::{
    PageMeta, SchemaOrg, faq_schema, organization_schema, software_application_schema,
};
use crate::content::Locale;

#[component]
pub fn HomePage() -> impl IntoView {
    let locale = use_context::<Locale>().expect("Locale context missing");
    let c = locale.content();
    let prefix = locale.path_prefix();
    let lang = locale.image_prefix();
    let app_url = env!("ORIGA_APP_BASE_URL");

    let features_href = format!("{prefix}/features");
    let download_href = format!("{prefix}/download");

    // Mini-FAQ: the "free?" pair is homepage-specific; the other two reuse
    // the features-page strings so the same answer never diverges between
    // pages. Visible Q&A mirrors the FAQPage JSON-LD 1:1 (same contract as
    // `features.rs`).
    let faq_pairs = [
        (c.home_faq_free_question, c.home_faq_free_answer),
        (c.faq_q3, c.faq_a3),
        (c.faq_q5, c.faq_a5),
    ];
    let faq_json = faq_schema(locale, &faq_pairs);

    view! {
        <PageMeta locale title=c.home_meta_title description=c.home_meta_description/>
        <SchemaOrg json=software_application_schema(locale)/>
        <SchemaOrg json=organization_schema()/>
        <SchemaOrg json=faq_json/>

        // Section 1: Hero (Split layout)
        <section class="home-hero">
            <div class="home-hero__text">
                <h1 class="home-hero__title">{c.home_hero_title}</h1>
                <hr class="home-hero__rule" />
                <p class="home-hero__subtitle">{c.home_hero_subtitle}</p>
                <div class="home-hero__cta">
                    <A href=format!("{prefix}/download") attr:class="btn btn-filled">
                        {c.home_cta_primary}
                    </A>
                    <a href=app_url class="btn">{c.home_cta_secondary}</a>
                </div>
                <p class="home-hero__tagline">{c.home_hero_tagline}</p>
            </div>
            <div class="home-hero__decor">
                <div
                    class="home-hero__decor-img"
                    aria-hidden="true"
                    style=format!("background-image: url(/images/{lang}.hero.png)")
                ></div>
            </div>
        </section>

        <hr class="divider-full" />

        // Section 1b: stat strip — concrete, scannable facts under the hero
        <section class="home-stats">
            <p class="home-stats__line">{c.home_stats_line}</p>
        </section>

        <hr class="divider-full" />

        // Section 2: Problem + Principles (2-column)
        <section class="home-dual">
            <div class="home-dual__inner">
                // Left: Problem
                <div class="home-dual__problem">
                    <h2 class="home-dual__problem-title">{c.home_problem_h2}</h2>
                    <hr class="home-dual__problem-rule" />
                    <p class="home-dual__problem-text">{c.home_problem_text}</p>
                </div>
                // Right: Principles (no heading)
                <div class="home-dual__principles">
                    <div class="home-dual__principle">
                        <strong>{c.home_principle_content_title}</strong>
                        " "
                        {c.home_principle_content_text}
                    </div>
                    <div class="home-dual__principle">
                        <strong>{c.home_principle_fsrs_title}</strong>
                        " "
                        {c.home_principle_fsrs_text}
                    </div>
                    <div class="home-dual__principle">
                        <strong>{c.home_principle_local_title}</strong>
                        " "
                        {c.home_principle_local_text}
                    </div>
                    <div class="home-dual__principle">
                        <strong>{c.home_principle_offline_title}</strong>
                        " "
                        {c.home_principle_offline_text}
                    </div>
                </div>
            </div>
        </section>

        <hr class="divider-full" />

        // Section 3: Features Preview (staggered grid)
        <section class="home-features">
            <h2 class="home-features__title">{c.home_features_h2}</h2>
            <div class="home-features__grid home-features__grid--top">
                <HomeFeatureCard
                    number="01"
                    title=c.home_feature_vocab_title
                    text=c.home_feature_vocab_text
                    href=features_href.clone()
                />
                <HomeFeatureCard
                    number="02"
                    title=c.home_feature_kanji_title
                    text=c.home_feature_kanji_text
                    href=features_href.clone()
                />
            </div>
            <div
                class="home-features__grid home-features__grid--bottom"
                style="margin-top: var(--space-lg)"
            >
                <HomeFeatureCard
                    number="03"
                    title=c.home_feature_grammar_title
                    text=c.home_feature_grammar_text
                    href=features_href.clone()
                />
                <HomeFeatureCard
                    number="04"
                    title=c.home_feature_listening_title
                    text=c.home_feature_listening_text
                    href=features_href
                />
            </div>
        </section>

        <hr class="divider-full" />

        // Section 4: Mini-FAQ (visible Q&A mirrors the FAQPage JSON-LD 1:1)
        <section class="feat-faq">
            <div class="feat-faq__inner">
                <h2>{c.features_faq_h2}</h2>
                <div class="feat-faq__list">
                    {faq_pairs.map(|(question, answer)| {
                        view! {
                            <div class="feat-faq__item">
                                <p class="feat-faq__question">{question}</p>
                                <p class="feat-faq__answer">{answer}</p>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
        </section>

        <hr class="divider-full" />

        // Section 5: Final CTA (dark olive) with platforms
        <CtaSection title=c.home_cta_title button_text=c.home_cta_primary download_href=download_href />
    }
}

#[component]
fn HomeFeatureCard(
    number: &'static str,
    title: &'static str,
    text: &'static str,
    href: String,
) -> impl IntoView {
    view! {
        <div class="home-feature-card">
            <span class="home-feature-card__number">{number}</span>
            <h3 class="home-feature-card__title">{title}</h3>
            <p class="home-feature-card__text">{text}</p>
            <A href=href attr:class="landing-feature-card__link">{title}</A>
        </div>
    }
}
