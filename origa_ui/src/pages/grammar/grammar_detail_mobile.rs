use std::collections::HashSet;

use leptos::prelude::*;
use origa::dictionary::grammar::{Nuances, RelatedPattern};
use origa::domain::{NativeLanguage, User};

use super::grammar_warnings::GrammarWarnings;
use super::nuances_section::NuancesSection;
use super::related_pattern_list::RelatedPatternList;
use crate::ui_components::MarkdownText;

#[component]
pub fn GrammarMobileOverview(
    explanation: Memo<Option<String>>,
    how_to_form: Memo<Option<String>>,
    examples: Memo<Option<String>>,
    nuances: Memo<Option<Nuances>>,
    pro_tip: Memo<Option<String>>,
    related_patterns: Memo<Vec<RelatedPattern>>,
    warnings: Memo<Vec<String>>,
    native_language: NativeLanguage,
    current_user: Option<User>,
    #[prop(into)] explanation_title: Signal<String>,
    #[prop(into)] how_to_form_title: Signal<String>,
    #[prop(into)] examples_title: Signal<String>,
    #[prop(into)] nuances_title: Signal<String>,
    #[prop(into)] pro_tip_title: Signal<String>,
    #[prop(into)] related_title: Signal<String>,
    known_kanji: HashSet<char>,
) -> impl IntoView {
    let known_kanji_stored = StoredValue::new(known_kanji);
    let current_user_stored = StoredValue::new(current_user);

    view! {
        <Show when=move || explanation.get().is_some_and(|s| !s.is_empty())>
            <div class="grammar-detail-section">
                <div class="grammar-detail-section-card">
                    <div class="grammar-detail-section-title">{explanation_title}</div>
                    <MarkdownText
                        content=Signal::derive(move || explanation.get().unwrap_or_default())
                        known_kanji=known_kanji_stored.get_value()
                    />
                    <Show when=move || !warnings.get().is_empty()>
                        <GrammarWarnings
                            warnings=warnings.get()
                            known_kanji=known_kanji_stored.get_value()
                            test_id=Signal::derive(|| "grammar-detail-warnings-mobile".to_string())
                        />
                    </Show>
                </div>
            </div>
        </Show>

        <Show when=move || how_to_form.get().is_some_and(|s| !s.is_empty())>
            <div class="grammar-detail-section">
                <div class="grammar-detail-section-card">
                    <div class="grammar-detail-section-title">{how_to_form_title}</div>
                    <MarkdownText
                        content=Signal::derive(move || how_to_form.get().unwrap_or_default())
                        known_kanji=known_kanji_stored.get_value()
                    />
                </div>
            </div>
        </Show>

        <Show when=move || examples.get().is_some_and(|s| !s.is_empty())>
            <div class="grammar-detail-section">
                <div class="grammar-detail-section-card">
                    <div class="grammar-detail-section-title">{examples_title}</div>
                    <MarkdownText
                        content=Signal::derive(move || examples.get().unwrap_or_default())
                        known_kanji=known_kanji_stored.get_value()
                    />
                </div>
            </div>
        </Show>

        <Show when=move || nuances.get().is_some_and(|n| !n.is_empty())>
            <div class="grammar-detail-section">
                <div class="grammar-detail-section-card">
                    <div class="grammar-detail-section-title">{nuances_title}</div>
                    {move || {
                        let nuances = nuances.get()?;
                        Some(view! {
                            <NuancesSection
                                common_mistakes=nuances.common_mistakes().to_vec()
                                notes=nuances.notes().to_vec()
                                known_kanji=known_kanji_stored.get_value()
                                test_id=Signal::derive(|| "grammar-detail-nuances-mobile".to_string())
                            />
                        }.into_any())
                    }}
                </div>
            </div>
        </Show>

        <Show when=move || pro_tip.get().is_some_and(|s| !s.is_empty())>
            <div class="grammar-detail-section">
                <div class="grammar-detail-section-card">
                    <div class="grammar-detail-section-title">{pro_tip_title}</div>
                    <MarkdownText
                        content=Signal::derive(move || pro_tip.get().unwrap_or_default())
                        known_kanji=known_kanji_stored.get_value()
                    />
                </div>
            </div>
        </Show>

        <Show when=move || !related_patterns.get().is_empty()>
            <div class="grammar-detail-section">
                <div class="grammar-detail-section-card">
                    <div class="grammar-detail-section-title">{related_title}</div>
                    <RelatedPatternList
                        related=related_patterns.get()
                        native_language=native_language
                        current_user=current_user_stored.get_value()
                        known_kanji=known_kanji_stored.get_value()
                        test_id=Signal::derive(|| "grammar-detail-related-mobile".to_string())
                    />
                </div>
            </div>
        </Show>
    }
}
