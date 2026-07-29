use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use std::cmp::Reverse;

/// A reading rendered by [`ReadingGroup`]. Carries the corpus `freq` (for
/// sort order; 0 when no data is available) and a pre-computed `is_rare`
/// flag. The flag is computed by the caller via the domain
/// (`KanjiInfo::is_rare_reading`) — the presentation layer does not decide
/// rarity, it only renders it. This avoids the "no data → all rare" trap
/// that would otherwise fire if the field compared `freq` to a threshold
/// directly.
#[derive(Clone)]
pub struct ReadingItem {
    pub reading: String,
    pub freq: u32,
    pub is_rare: bool,
}

#[component]
pub fn ReadingGroup(
    #[prop(into)] label: Signal<String>,
    readings: StoredValue<Option<Vec<ReadingItem>>>,
    #[prop(optional, into)] rare_hint: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    // Sort by frequency desc (Reverse makes sort_by_key produce descending
    // order while staying stable — preserves the source order of tied
    // readings, e.g. all-rare kanji in fallback mode).
    let sorted = move || {
        let mut items = readings.get_value().unwrap_or_default();
        items.sort_by_key(|item| Reverse(item.freq));
        items
    };

    let has_rare = move || {
        readings
            .get_value()
            .map(|rs| rs.iter().any(|item| item.is_rare))
            .unwrap_or(false)
    };

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <Show when=move || readings.get_value().is_some()>
            <div class="reading-group" data-testid=test_id_val>
                <div class="reading-kanji">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        {move || label.get()}
                    </Text>
                </div>
                <div class="reading-main">
                    <div class="reading-furigana">
                        <For
                            each=sorted
                            key=|item| item.reading.clone()
                            children=move |item: ReadingItem| {
                                let class = if item.is_rare {
                                    "reading-tag reading-tag--rare"
                                } else {
                                    "reading-tag"
                                };
                                // Stringify so tachys always renders the
                                // attribute value (not presence-only). E2E
                                // relies on `[data-rare="true"]` / `"false"`.
                                let rare_attr = if item.is_rare { "true" } else { "false" };
                                view! {
                                    <span class=class data-rare=rare_attr>
                                        {item.reading}
                                    </span>
                                }
                            }
                        />
                    </div>
                    <Show when=move || has_rare() && !rare_hint.get().is_empty()>
                        <div class="reading-rare-hint">
                            {"* "}{move || rare_hint.get()}
                        </div>
                    </Show>
                </div>
            </div>
        </Show>
    }
}
