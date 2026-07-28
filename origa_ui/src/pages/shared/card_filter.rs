use super::card_status::CardStatus;
use crate::i18n::{Locale, use_i18n};
use crate::ui_components::{Tag, TagVariant};
use leptos::prelude::*;
use leptos_i18n::I18nContext;

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Filter {
    #[default]
    All,
    New,
    Hard,
    InProgress,
    Learned,
    Favorite,
}

impl Filter {
    pub fn label(&self, i18n: &I18nContext<Locale>) -> String {
        match self {
            Filter::All => i18n.get_keys().shared().filter_all().inner().to_string(),
            Filter::New => i18n.get_keys().shared().filter_new().inner().to_string(),
            Filter::Hard => i18n.get_keys().shared().filter_hard().inner().to_string(),
            Filter::InProgress => i18n
                .get_keys()
                .shared()
                .filter_in_progress()
                .inner()
                .to_string(),
            Filter::Learned => i18n
                .get_keys()
                .shared()
                .filter_learned()
                .inner()
                .to_string(),
            Filter::Favorite => i18n
                .get_keys()
                .shared()
                .filter_favorite()
                .inner()
                .to_string(),
        }
    }

    /// Returns true if the card matches this filter.
    ///
    /// `is_favorite` is independent of `CardStatus` (a card can be both Learned
    /// and favorite at once), so it is passed as a separate argument rather than
    /// collapsed into the status enum.
    pub fn matches(&self, status: CardStatus, is_favorite: bool) -> bool {
        match self {
            Filter::All => true,
            Filter::New => status == CardStatus::New,
            Filter::Hard => status == CardStatus::Hard,
            Filter::InProgress => status == CardStatus::InProgress,
            Filter::Learned => status == CardStatus::Learned,
            Filter::Favorite => is_favorite,
        }
    }
}

#[component]
pub fn FilterBtn<F: Fn() -> usize + Send + 'static>(
    filter: Filter,
    count: F,
    active: RwSignal<Filter>,
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

    fn any_status() -> CardStatus {
        CardStatus::New
    }

    #[test]
    fn filter_favorite_matches_only_favorited_cards() {
        // Arrange
        let filter = Filter::Favorite;

        // Assert
        assert!(filter.matches(any_status(), true));
        assert!(!filter.matches(any_status(), false));
    }

    #[test]
    fn filter_favorite_ignores_card_status() {
        // Arrange — favorite filter must not depend on card status
        let filter = Filter::Favorite;

        // Assert
        assert!(filter.matches(CardStatus::Learned, true));
        assert!(filter.matches(CardStatus::InProgress, true));
        assert!(filter.matches(CardStatus::Hard, true));
        assert!(filter.matches(CardStatus::New, true));
    }

    #[test]
    fn filter_all_matches_anything() {
        // Arrange
        let filter = Filter::All;

        // Assert — both status and is_favorite are irrelevant to Filter::All
        assert!(filter.matches(CardStatus::Learned, true));
        assert!(filter.matches(CardStatus::New, false));
    }

    #[test]
    fn filter_status_variants_ignore_is_favorite() {
        // Arrange
        let new = Filter::New;
        let learned = Filter::Learned;

        // Assert — favorite state does not affect status-based filters
        assert!(new.matches(CardStatus::New, true));
        assert!(new.matches(CardStatus::New, false));
        assert!(!new.matches(CardStatus::Learned, true));
        assert!(learned.matches(CardStatus::Learned, true));
        assert!(learned.matches(CardStatus::Learned, false));
        assert!(!learned.matches(CardStatus::New, false));
    }
}
