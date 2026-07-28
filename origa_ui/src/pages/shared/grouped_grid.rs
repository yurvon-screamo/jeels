use std::collections::HashMap;

use crate::i18n::{I18nContext, Locale, use_i18n};
use crate::ui_components::{Heading, HeadingLevel};
use leptos::prelude::*;
use origa::domain::{JapaneseLevel, StudyCard};

use super::grouping::{LevelIndex, group_rank};

/// Renders a list of study cards split into JLPT-level groups (N5 first,
/// "Other" last). Each group has a `Heading` followed by a grid of cards.
///
/// Takes `cards` and `level_index` as `Memo`s so a favorite-toggle + reload
/// cycle that produces a new `cards` snapshot re-runs `bucket_by_group` and
/// updates every group's `<For>` via keyed diff. The previous version took
/// owned snapshots, which Leptos diffs in place without re-running setup —
/// leaving `buckets` stale after the parent's reactive closure re-evaluates.
///
/// Lives in the lib crate (which has `recursion_limit = 512`) so adding the
/// nested `<For>` over groups does not tip the bin crate over its 128-query
/// depth limit — see `origa_ui/AGENTS.md` (recursion_limit landmine) and
/// ADR-027 §B3 for the bin-vs-lib trade-off.
#[component]
pub fn GroupedGrid<F>(
    cards: Memo<Vec<StudyCard>>,
    level_index: Memo<LevelIndex>,
    grid_classes: &'static str,
    test_id_prefix: &'static str,
    render_card: F,
) -> impl IntoView
where
    F: Fn(StudyCard) -> AnyView + Clone + Send + Sync + 'static,
{
    let render_card_stored = StoredValue::new(render_card);

    // Reactive buckets — re-bucket every time `cards` or `level_index`
    // changes. The downstream `<For>` over GROUP_ORDER is itself reactive on
    // `buckets`, so each group's section re-renders only when its bucket
    // contents change (keyed by card_id + is_favorite).
    let buckets = Memo::new(move |_| bucket_by_group(cards.get().as_slice(), &level_index.get()));

    view! {
        <For
            each=move || GROUP_ORDER.into_iter()
            key=|group| group.testid_suffix
            children=move |group| {
                let bucket_for_group = Memo::new(move |_| {
                    buckets.with(|b| b[group.rank as usize].clone())
                });
                let render = render_card_stored.with_value(|r| r.clone());
                view! {
                    <GroupSection
                        group=group
                        bucket=bucket_for_group
                        grid_classes=grid_classes
                        test_id_prefix=test_id_prefix
                        render_card=render
                    />
                }
            }
        />
    }
}

#[component]
fn GroupSection<F>(
    group: GroupMeta,
    bucket: Memo<Vec<StudyCard>>,
    grid_classes: &'static str,
    test_id_prefix: &'static str,
    render_card: F,
) -> AnyView
where
    F: Fn(StudyCard) -> AnyView + Clone + Send + Sync + 'static,
{
    let i18n = use_i18n();
    let render_card_stored = StoredValue::new(render_card);

    view! {
        <Show when=move || !bucket.with(Vec::is_empty)>
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
                    <For
                        each=move || bucket.get()
                        key=|card| format!("{}-{}", card.card_id(), card.is_favorite())
                        children=move |card| {
                            let render = render_card_stored.with_value(|r| r.clone());
                            render(card)
                        }
                    />
                </div>
            </section>
        </Show>
    }
    .into_any()
}

fn bucket_by_group(
    cards: &[StudyCard],
    level_index: &HashMap<ulid::Ulid, Option<JapaneseLevel>>,
) -> Vec<Vec<StudyCard>> {
    let mut buckets: Vec<Vec<StudyCard>> = (0..6).map(|_| Vec::new()).collect();
    for card in cards {
        // group_rank is documented to return 0..=5; clamp defensively.
        let idx = level_index
            .get(card.card_id())
            .map_or(5usize, |l| group_rank(l) as usize)
            .min(5);
        buckets[idx].push(card.clone());
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
