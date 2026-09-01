use std::collections::HashSet;

use leptos::prelude::*;

use crate::ui_components::MarkdownText;

/// Render for grammar rule warnings (schema v2) — callouts lifted out of the
/// explanation markdown. Kept visually restrained: left rule + paper tone,
/// no radius, hard shadow only.
#[component]
pub fn GrammarWarnings(
    warnings: Vec<String>,
    known_kanji: HashSet<char>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let known_kanji_stored = StoredValue::new(known_kanji);

    view! {
        <div class="grammar-warnings" data-testid=move || test_id.get()>
            <For
                each=move || warnings.clone()
                key=|warning: &String| warning.clone()
                children=move |warning| {
                    view! {
                        <div class="grammar-warning">
                            <MarkdownText
                                content=Signal::derive(move || warning.clone())
                                known_kanji=known_kanji_stored.get_value()
                            />
                        </div>
                    }
                }
            />
        </div>
    }
}
