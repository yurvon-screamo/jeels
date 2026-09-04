use std::collections::HashSet;

use leptos::prelude::*;
use leptos_router::components::A;
use origa::dictionary::grammar::{RelatedPattern, get_rule_by_id};
use origa::domain::{NativeLanguage, User};
use ulid::Ulid;

use crate::ui_components::FuriganaText;

/// Structured render of related grammar patterns (schema v2). References are
/// stored by stable `rule_id`; the target title and JLPT level are resolved
/// at render time, so renames never break links. When the current user
/// already studies the related rule, the chip deep-links to its detail page.
#[component]
pub fn RelatedPatternList(
    related: Vec<RelatedPattern>,
    native_language: NativeLanguage,
    current_user: Option<User>,
    known_kanji: HashSet<char>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let known_kanji_stored = StoredValue::new(known_kanji);

    view! {
        <div class="grammar-related" data-testid=move || test_id.get()>
            <For
                each=move || related.clone().into_iter().enumerate()
                key=|(index, _): &(usize, RelatedPattern)| *index
                children=move |(_index, pattern)| {
                    let Some(rule) = get_rule_by_id(pattern.rule_id()) else {
                        tracing::warn!(
                            rule_id = %pattern.rule_id(),
                            "RelatedPatternList: target rule not found"
                        );
                        return view! { <span></span> }.into_any();
                    };
                    let title = rule.content(&native_language).title().to_string();
                    let level = rule.level().to_string();
                    let note = StoredValue::new(pattern.note().map(|n| n.to_string()).unwrap_or_default());
                    let card_href = current_user
                        .as_ref()
                        .and_then(|user| study_card_href(user, pattern.rule_id()));

                    let (linked_title, plain_title) = (title.clone(), title.clone());
                    let (linked_level, plain_level) = (level.clone(), level.clone());
                    view! {
                        <div class="grammar-related-chip">
                            {match card_href {
                                Some(href) => view! {
                                    <A href=href attr:class="grammar-related-chip-link">
                                        <span class="grammar-related-chip-title">
                                            <FuriganaText
                                                text=linked_title
                                                known_kanji=known_kanji_stored.get_value()
                                            />
                                        </span>
                                        <span class="grammar-related-chip-level">{linked_level}</span>
                                    </A>
                                }
                                    .into_any(),
                                None => view! {
                                    <span class="grammar-related-chip-title">
                                        <FuriganaText
                                            text=plain_title
                                            known_kanji=known_kanji_stored.get_value()
                                        />
                                    </span>
                                    <span class="grammar-related-chip-level">{plain_level}</span>
                                }
                                    .into_any(),
                            }}
                            <Show when=move || !note.get_value().is_empty()>
                                <div class="grammar-related-chip-note">{note.get_value()}</div>
                            </Show>
                        </div>
                    }
                        .into_any()
                }
            />
        </div>
    }
}

/// A related pattern links to the current user's study card for that rule;
/// rules the user does not study yet render as a plain (non-link) chip.
fn study_card_href(user: &User, rule_id: &Ulid) -> Option<String> {
    user.knowledge_set()
        .study_cards()
        .iter()
        .find(|(_, card)| {
            matches!(
                card.card(),
                origa::domain::Card::Grammar(grammar) if grammar.rule_id() == rule_id
            )
        })
        .map(|(card_id, _)| format!("/grammar/{}", card_id))
}
