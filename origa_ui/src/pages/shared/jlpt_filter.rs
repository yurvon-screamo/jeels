use crate::i18n::{Locale, use_i18n};
use crate::ui_components::{Tag, TagVariant};
use leptos::prelude::*;
use leptos_i18n::I18nContext;
use origa::domain::JapaneseLevel;

/// Orthogonal axis to [`super::Filter`]: filters cards by their JLPT level.
/// Composes with status filter and search through AND in `card_list_view`.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum JlptFilter {
    #[default]
    All,
    Level(JapaneseLevel),
    Other,
}

impl JlptFilter {
    /// `None` (no JLPT data) matches only `Other` and `All`.
    pub fn matches(self, level: Option<JapaneseLevel>) -> bool {
        match self {
            JlptFilter::All => true,
            JlptFilter::Level(target) => level == Some(target),
            JlptFilter::Other => level.is_none(),
        }
    }

    pub fn label(self, i18n: &I18nContext<Locale>) -> String {
        match self {
            JlptFilter::All => i18n.get_keys().shared().filter_all().inner().to_string(),
            JlptFilter::Level(level) => level.code().to_string(),
            JlptFilter::Other => i18n
                .get_keys()
                .shared()
                .group_other_label()
                .inner()
                .to_string(),
        }
    }
}

/// Sibling of [`super::CardCounts`] — mirrors its `#[derive(Clone, Copy,
/// PartialEq, Default)]` so it can flow through `Memo::get()` cheaply. The
/// `by_level` layout matches `group_rank` ([`super::group_rank`]) and
/// `GROUP_ORDER` (`grouped_grid.rs`), keeping a single indexing convention
/// across the family.
#[derive(Clone, Copy, PartialEq, Default)]
pub struct JlptCounts {
    pub total: usize,
    pub by_level: [usize; 5],
    pub other: usize,
}

impl JlptCounts {
    pub fn level_count(self, level: JapaneseLevel) -> usize {
        self.by_level[jlpt_level_idx(level)]
    }
}

/// Index into [`JlptCounts::by_level`]. Mirrors `group_rank` (grouping.rs:16)
/// so the array layout matches the render order of `GROUP_ORDER`.
pub fn jlpt_level_idx(level: JapaneseLevel) -> usize {
    match level {
        JapaneseLevel::N5 => 0,
        JapaneseLevel::N4 => 1,
        JapaneseLevel::N3 => 2,
        JapaneseLevel::N2 => 3,
        JapaneseLevel::N1 => 4,
    }
}

/// Filter chip for the JLPT axis. Visual twin of [`super::FilterBtn`] (same
/// `Tag` / `TagVariant` styling, same `format!("{} ({})", ...)` label); kept
/// as a separate component because `JlptFilter` and `Filter` are independent
/// domain concepts that evolve separately (status axis vs level axis).
#[component]
pub fn JlptFilterBtn<F: Fn() -> usize + Send + 'static>(
    filter: JlptFilter,
    count: F,
    active: RwSignal<JlptFilter>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let is_active = Memo::new(move |_| active.get() == filter);
    let filter_for_click = filter;

    view! {
        <Tag
            variant=Signal::derive(move || if is_active.get() { TagVariant::Filled } else { TagVariant::Default })
            test_id=test_id
            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                active.set(filter_for_click);
            })
        >
            {move || format!("{} ({})", filter.label(&i18n), count())}
        </Tag>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_all_accepts_any_level() {
        assert!(JlptFilter::All.matches(Some(JapaneseLevel::N5)));
        assert!(JlptFilter::All.matches(Some(JapaneseLevel::N1)));
        assert!(JlptFilter::All.matches(None));
    }

    #[test]
    fn matches_level_only_accepts_exact() {
        assert!(JlptFilter::Level(JapaneseLevel::N4).matches(Some(JapaneseLevel::N4)));
        assert!(!JlptFilter::Level(JapaneseLevel::N4).matches(Some(JapaneseLevel::N5)));
        assert!(!JlptFilter::Level(JapaneseLevel::N4).matches(None));
    }

    #[test]
    fn matches_other_only_accepts_none() {
        assert!(JlptFilter::Other.matches(None));
        assert!(!JlptFilter::Other.matches(Some(JapaneseLevel::N5)));
    }

    #[test]
    fn counts_level_count_reads_correct_slot() {
        let counts = JlptCounts {
            total: 100,
            by_level: [10, 20, 30, 5, 1],
            other: 34,
        };
        assert_eq!(counts.level_count(JapaneseLevel::N5), 10);
        assert_eq!(counts.level_count(JapaneseLevel::N4), 20);
        assert_eq!(counts.level_count(JapaneseLevel::N3), 30);
        assert_eq!(counts.level_count(JapaneseLevel::N2), 5);
        assert_eq!(counts.level_count(JapaneseLevel::N1), 1);
    }
}
