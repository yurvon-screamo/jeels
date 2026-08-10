use crate::i18n::*;
use crate::pages::lesson::card_type::CardType;
use crate::ui_components::{Text, TextSize, TypographyVariant};
use leptos::prelude::*;

/// One segment of the scoring queue. `end` is exclusive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct SectionBound {
    pub card_type: CardType,
    pub start: usize,
    pub end: usize,
}

/// Builds the visible section segments from the (already sorted) scoring card
/// list. Empty sections are dropped so the progress bar never paints a marker
/// for a type that does not appear in this user's import (e.g. an onboarding
/// that produced no phrase cards).
pub(super) fn compute_section_bounds(card_types: &[CardType]) -> Vec<SectionBound> {
    let mut bounds = Vec::new();
    let mut current: Option<(CardType, usize)> = None;
    for (idx, &card_type) in card_types.iter().enumerate() {
        match &mut current {
            Some((ct, start)) if *ct == card_type => {},
            Some((ct, start)) => {
                bounds.push(SectionBound {
                    card_type: *ct,
                    start: *start,
                    end: idx,
                });
                current = Some((card_type, idx));
            },
            None => {
                current = Some((card_type, idx));
            },
        }
    }
    if let Some((ct, start)) = current {
        bounds.push(SectionBound {
            card_type: ct,
            start,
            end: card_types.len(),
        });
    }
    bounds
}

fn section_label(i18n: &I18nContext<Locale>, card_type: CardType) -> String {
    let locale = i18n.get_locale();
    match card_type {
        CardType::Grammar => td_string!(locale, onboarding.scoring.section.grammar).to_string(),
        CardType::Kanji => td_string!(locale, onboarding.scoring.section.kanji).to_string(),
        CardType::Vocabulary => {
            td_string!(locale, onboarding.scoring.section.vocabulary).to_string()
        },
        CardType::Phrase => td_string!(locale, onboarding.scoring.section.phrase).to_string(),
    }
}

/// CSS class for a section's colour in the progress bar legend and marker.
/// Grammar → Terracotta, Kanji → Olive, Vocabulary/Phrase → default (black).
fn section_color_class(card_type: CardType) -> &'static str {
    match card_type {
        CardType::Grammar => "scoring-section-grammar",
        CardType::Kanji => "scoring-section-kanji",
        CardType::Vocabulary | CardType::Phrase => "",
    }
}

/// Horizontal progress bar with one tick per scoring section. The fill width
/// tracks the user's position in the overall queue (not per-section), and a
/// vertical "marker" line is drawn at every section boundary so the user can
/// see how the queue is laid out: Grammar | Kanji | Vocabulary | …
#[component]
pub fn ScoringProgressBar(
    current_index: Signal<usize>,
    total: Signal<usize>,
    sections: Signal<Vec<SectionBound>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let fill_percent = Signal::derive(move || {
        let total = total.get();
        if total == 0 {
            return 0.0;
        }
        let idx = current_index.get();
        ((idx as f64 / total as f64) * 100.0).clamp(0.0, 100.0)
    });

    view! {
        <div class="scoring-progress-bar" data-testid=test_id_val>
            <div class="scoring-progress-track relative">
                <div
                    class="scoring-progress-fill"
                    style=move || format!("width: {}%", fill_percent.get())
                ></div>
                <For
                    each=move || {
                        let total_val = total.get();
                        sections
                            .get()
                            .into_iter()
                            .filter(move |s| s.end < total_val)
                            .collect::<Vec<_>>()
                    }
                    key=|s| (s.card_type.sort_order(), s.start, s.end)
                    children=move |s| {
                        let total_val = total.get();
                        let pct = (s.end as f64 / total_val.max(1) as f64) * 100.0;
                        let marker_test_id = format!("scoring-progress-marker-{}", s.card_type.sort_order());
                        let color_class = section_color_class(s.card_type);
                        view! {
                            <div
                                class=format!("scoring-progress-marker {}", color_class)
                                style=format!("left: {}%", pct)
                                data-testid=marker_test_id
                            ></div>
                        }
                    }
                />
            </div>
            <div class="scoring-progress-legend flex flex-wrap justify-center gap-x-4 gap-y-1 mt-2">
                {move || {
                    let i18n = i18n;
                    sections
                        .get()
                        .into_iter()
                        .map(move |s| {
                            let label = section_label(&i18n, s.card_type);
                            let size = s.end.saturating_sub(s.start);
                            let color_class = section_color_class(s.card_type);
                            view! {
                                <Text size=TextSize::Small variant=TypographyVariant::Muted class=Signal::derive(move || color_class.to_string())>
                                    {format!("{}: {}", label, size)}
                                </Text>
                            }.into_any()
                        })
                        .collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_section_bounds_drops_empty_sections() {
        // Arrange — queue with only Grammar and Vocabulary (no Kanji, no
        // Phrase) like a typical onboarding scoring without phrase cards.
        let card_types = vec![
            CardType::Grammar,
            CardType::Grammar,
            CardType::Vocabulary,
            CardType::Vocabulary,
            CardType::Vocabulary,
        ];

        // Act
        let bounds = compute_section_bounds(&card_types);

        // Assert
        assert_eq!(bounds.len(), 2);
        assert_eq!(bounds[0].card_type, CardType::Grammar);
        assert_eq!(bounds[0].start, 0);
        assert_eq!(bounds[0].end, 2);
        assert_eq!(bounds[1].card_type, CardType::Vocabulary);
        assert_eq!(bounds[1].start, 2);
        assert_eq!(bounds[1].end, 5);
    }

    #[test]
    fn compute_section_bounds_handles_empty_input() {
        let bounds = compute_section_bounds(&[]);
        assert!(bounds.is_empty());
    }

    #[test]
    fn compute_section_bounds_preserves_sorted_input() {
        // Sorted by sort_order: Grammar(0) → Kanji(1) → Vocabulary(2) → Phrase(3)
        let card_types = vec![
            CardType::Grammar,
            CardType::Kanji,
            CardType::Kanji,
            CardType::Vocabulary,
            CardType::Phrase,
        ];

        let bounds = compute_section_bounds(&card_types);

        assert_eq!(bounds.len(), 4);
        assert_eq!(bounds[0].card_type, CardType::Grammar);
        assert_eq!(bounds[1].card_type, CardType::Kanji);
        assert_eq!(bounds[2].card_type, CardType::Vocabulary);
        assert_eq!(bounds[3].card_type, CardType::Phrase);
    }
}
