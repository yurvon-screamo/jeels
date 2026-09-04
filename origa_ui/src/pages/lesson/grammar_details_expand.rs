use crate::i18n::*;
use crate::ui_components::MarkdownText;
use leptos::prelude::*;
use origa::dictionary::grammar::get_rule_by_id;
use std::collections::HashSet;
use ulid::Ulid;

use super::lesson_state::LessonContext;
use crate::pages::grammar::grammar_warnings::GrammarWarnings;
use crate::pages::grammar::nuances_section::NuancesSection;

#[component]
pub fn GrammarDetailsExpand(
    rule_id: Ulid,
    is_expanded: RwSignal<bool>,
    known_kanji: HashSet<char>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let lesson_ctx = use_context::<LessonContext>().expect("LessonContext");
    let native_lang = lesson_ctx.native_language;
    let known_kanji_stored = StoredValue::new(known_kanji);

    let rule = StoredValue::new(get_rule_by_id(&rule_id));

    let more_text =
        Signal::derive(move || i18n.get_keys().common().more_details().inner().to_string());
    let collapse_text =
        Signal::derive(move || i18n.get_keys().common().collapse().inner().to_string());

    let explanation_title = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .explanation()
            .inner()
            .to_string()
    });
    let how_to_form_title = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .how_to_form()
            .inner()
            .to_string()
    });
    let examples_title = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .examples()
            .inner()
            .to_string()
    });
    let nuances_title =
        Signal::derive(move || i18n.get_keys().grammar_page().nuances().inner().to_string());
    let pro_tip_title =
        Signal::derive(move || i18n.get_keys().grammar_page().pro_tip().inner().to_string());

    view! {
        <div class="mt-3" data-testid=move || test_id.get()>
            <button
                class="font-mono text-sm text-[var(--fg-muted)] cursor-pointer hover:text-[var(--fg-black)] underline underline-offset-4 decoration-[var(--border-light)]"
                on:click=move |_| is_expanded.update(|v| *v = !*v)
            >
                {move || if is_expanded.get() { collapse_text.get() } else { more_text.get() }}
            </button>

            <Show when=move || is_expanded.get()>
                {move || {
                    match rule.get_value() {
                        Some(r) => {
                            let lang = native_lang.get();
                            let content = r.content(&lang);

                            view! {
                                <div class="mt-3 space-y-3 text-left">
                                    <div class="space-y-2">
                                        <GrammarSection
                                            title=explanation_title.get()
                                            content=content.explanation().to_string()
                                            known_kanji=known_kanji_stored.get_value()
                                        />
                                        <Show when=move || !content.warnings().is_empty()>
                                            <GrammarWarnings
                                                warnings=content.warnings().to_vec()
                                                known_kanji=known_kanji_stored.get_value()
                                                test_id=Signal::derive(|| "grammar-expand-warnings".to_string())
                                            />
                                        </Show>
                                    </div>
                                    <GrammarSection
                                        title=how_to_form_title.get()
                                        content=content.how_to_form().to_string()
                                        known_kanji=known_kanji_stored.get_value()
                                    />
                                    <GrammarSection
                                        title=examples_title.get()
                                        content=content.examples().to_string()
                                        known_kanji=known_kanji_stored.get_value()
                                    />
                                    <div>
                                        <div class="font-mono text-xs text-[var(--fg-muted)] mb-2">
                                            {nuances_title.get()}
                                        </div>
                                        <NuancesSection
                                            common_mistakes=content.nuances().common_mistakes().to_vec()
                                            notes=content.nuances().notes().to_vec()
                                            known_kanji=known_kanji_stored.get_value()
                                            test_id=Signal::derive(|| "grammar-expand-nuances".to_string())
                                        />
                                    </div>
                                    <GrammarSection
                                        title=pro_tip_title.get()
                                        content=content.pro_tip().to_string()
                                        known_kanji=known_kanji_stored.get_value()
                                    />
                                </div>
                            }
                                .into_any()
                        },
                        None => {
                            tracing::warn!("GrammarDetailsExpand: rule not found for id {}", rule_id);
                            view! { <div /> }.into_any()
                        },
                    }
                }}
            </Show>
        </div>
    }
}

#[component]
fn GrammarSection(title: String, content: String, known_kanji: HashSet<char>) -> impl IntoView {
    let title_stored = StoredValue::new(title);
    let content_stored = StoredValue::new(content);
    let kanji_stored = StoredValue::new(known_kanji);

    view! {
        <Show when=move || !content_stored.get_value().is_empty()>
            <div class="grammar-detail-section">
                <div class="grammar-detail-section-card">
                    <div class="grammar-detail-section-title">{title_stored.get_value()}</div>
                    <MarkdownText
                        content=Signal::derive(move || content_stored.get_value())
                        known_kanji=kanji_stored.get_value()
                    />
                </div>
            </div>
        </Show>
    }
}
