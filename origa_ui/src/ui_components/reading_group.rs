use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::RARE_READING_MAX_FREQ;
use std::cmp::Reverse;

/// A reading paired with its corpus frequency. Frequency `0` means the
/// reading has no corpus coverage; readings at or below
/// [`RARE_READING_MAX_FREQ`] are rendered muted with a small "rare" hint.
pub type ReadingWithFreq = (String, u32);

#[component]
pub fn ReadingGroup(
    #[prop(into)] label: Signal<String>,
    readings: StoredValue<Option<Vec<ReadingWithFreq>>>,
    #[prop(optional, into)] rare_hint: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    // Sort by frequency desc (Reverse makes sort_by_key produce descending
    // order while staying stable — preserves the source order of tied
    // readings, e.g. all-rare kanji in fallback mode).
    let sorted = move || {
        let mut items = readings.get_value().unwrap_or_default();
        items.sort_by_key(|(_, f)| Reverse(*f));
        items
    };

    let has_rare = move || {
        readings
            .get_value()
            .map(|rs| rs.iter().any(|(_, f)| *f <= RARE_READING_MAX_FREQ))
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
                            key=|(reading, _)| reading.clone()
                            children=move |(reading, freq): ReadingWithFreq| {
                                let is_rare = freq <= RARE_READING_MAX_FREQ;
                                let class = if is_rare {
                                    "reading-tag reading-tag--rare"
                                } else {
                                    "reading-tag"
                                };
                                // Stringify so tachys always renders the
                                // attribute value (not presence-only). E2E
                                // relies on `[data-rare="true"]` matching.
                                let rare_attr = if is_rare { "true" } else { "false" };
                                view! {
                                    <span class=class data-rare=rare_attr>
                                        {reading}
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
