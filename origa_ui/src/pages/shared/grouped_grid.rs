use std::collections::HashMap;

use crate::i18n::{I18nContext, Locale, use_i18n};
use crate::ui_components::{Heading, HeadingLevel};
use leptos::prelude::*;
use origa::domain::{JapaneseLevel, StudyCard};

use super::grouping::group_rank;

/// Renders a list of study cards split into JLPT-level groups (N5 first,
/// "Other" last). Each group has a `Heading` followed by a grid of cards.
///
/// Lives in the lib crate (which has `recursion_limit = 512`) so adding the
/// nested `<For>` over groups does not tip the bin crate over its 128-query
/// depth limit — see `origa_ui/AGENTS.md` (recursion_limit landmine) and
/// ADR-027 §B3 for the bin-vs-lib trade-off.
#[component]
pub fn GroupedGrid<F>(
    cards: Vec<StudyCard>,
    level_index: HashMap<ulid::Ulid, Option<JapaneseLevel>>,
    grid_classes: &'static str,
    test_id_prefix: &'static str,
    render_card: F,
) -> impl IntoView
where
    F: Fn(StudyCard) -> AnyView + Clone + Send + Sync + 'static,
{
    // Snapshot the cards once — they're already in group/card_id order coming
    // out of `card_list_view`. We bucket them by `group_rank` for the per-group
    // `<For>` loops below.
    let buckets = bucket_by_group(cards, &level_index);

    // The per-group rendering is delegated to a sub-component (GroupSection)
    // because `<For children>` requires a `Fn` closure, and a `view! { ... }`
    // block with nested `<Show>`/`<For>` inside the children closure degrades
    // to `FnOnce`. Splitting it lets Leptos see each `GroupSection` invocation
    // as a plain function call rather than a captured view-construction closure.
    view! {
        <For
            each=move || GROUP_ORDER.into_iter()
            key=|group| group.testid_suffix
            children=move |group| {
                let bucket = buckets[group.rank as usize].clone();
                view! {
                    <GroupSection
                        group=group
                        bucket=bucket
                        grid_classes=grid_classes
                        test_id_prefix=test_id_prefix
                        render_card=render_card.clone()
                    />
                }
            }
        />
    }
}

#[component]
fn GroupSection<F>(
    group: GroupMeta,
    bucket: Vec<StudyCard>,
    grid_classes: &'static str,
    test_id_prefix: &'static str,
    render_card: F,
) -> AnyView
where
    F: Fn(StudyCard) -> AnyView + Clone + Send + Sync + 'static,
{
    // Skip empty groups entirely — no heading, no empty grid. Returning early
    // also keeps the macro-generated children closure for the surrounding
    // `<For>` (in GroupedGrid) a plain `Fn` rather than `FnOnce`.
    if bucket.is_empty() {
        return ().into_any();
    }

    let i18n = use_i18n();

    // Render the bucket once into a static Vec<AnyView>. `bucket` is a snapshot
    // taken at mount time (no per-card reactivity needed), so a plain iterator
    // is both simpler and avoids the Fn/FnOnce dance that `<For children>`
    // would force on a captured `render_card`.
    let rendered_cards: Vec<AnyView> = bucket
        .into_iter()
        .map(|card| render_card.clone()(card))
        .collect();

    view! {
        <section
            class="grouped-section"
            data-testid=format!("{}-grid-{}", test_id_prefix, group.testid_suffix)
        >
            <div class="border-b border-[var(--border-dark)] mb-4">
                <Heading
                    level=Signal::derive(|| HeadingLevel::H3)
                    class=Signal::derive(|| "mb-2 font-mono text-xs uppercase tracking-[0.18em] text-[var(--fg-muted)]".to_string())
                >
                    {group.label(&i18n)}
                </Heading>
            </div>
            <div class=grid_classes>
                {rendered_cards}
            </div>
        </section>
    }
    .into_any()
}

fn bucket_by_group(
    cards: Vec<StudyCard>,
    level_index: &HashMap<ulid::Ulid, Option<JapaneseLevel>>,
) -> Vec<Vec<StudyCard>> {
    let mut buckets: Vec<Vec<StudyCard>> = (0..6).map(|_| Vec::new()).collect();
    for card in cards {
        // group_rank is documented to return 0..=5; clamp defensively.
        let idx = level_index
            .get(card.card_id())
            .map_or(5usize, |l| group_rank(l) as usize)
            .min(5);
        buckets[idx].push(card);
    }
    buckets
}

#[derive(Clone, Copy)]
enum GroupKind {
    Level(JapaneseLevel),
    Other,
}

#[derive(Clone, Copy)]
struct GroupMeta {
    rank: u8,
    kind: GroupKind,
    testid_suffix: &'static str,
}

impl GroupMeta {
    fn label(&self, i18n: &I18nContext<Locale>) -> String {
        match self.kind {
            GroupKind::Level(level) => format!("JLPT {}", level.code()),
            GroupKind::Other => i18n
                .get_keys()
                .shared()
                .group_other_label()
                .inner()
                .to_string(),
        }
    }
}

// Static ordering: N5 -> N4 -> N3 -> N2 -> N1 -> Other
// Match `group_rank` in `grouping.rs` exactly.
const GROUP_ORDER: [GroupMeta; 6] = [
    GroupMeta {
        rank: 0,
        kind: GroupKind::Level(JapaneseLevel::N5),
        testid_suffix: "N5",
    },
    GroupMeta {
        rank: 1,
        kind: GroupKind::Level(JapaneseLevel::N4),
        testid_suffix: "N4",
    },
    GroupMeta {
        rank: 2,
        kind: GroupKind::Level(JapaneseLevel::N3),
        testid_suffix: "N3",
    },
    GroupMeta {
        rank: 3,
        kind: GroupKind::Level(JapaneseLevel::N2),
        testid_suffix: "N2",
    },
    GroupMeta {
        rank: 4,
        kind: GroupKind::Level(JapaneseLevel::N1),
        testid_suffix: "N1",
    },
    GroupMeta {
        rank: 5,
        kind: GroupKind::Other,
        testid_suffix: "other",
    },
];
