use std::collections::HashSet;
use std::sync::Arc;

use super::{
    CardCounts, CardStatus, DeleteRequest, Filter, FilterBtn, GroupedGrid, LevelIndex,
    ListGrouping, LoadMoreButton, create_delete_callback, create_mark_as_known_callback,
    create_toggle_favorite_callback, order_cards_by_group,
};
use crate::i18n::use_i18n;
use crate::loaders::get_jlpt_content;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Input, LoadingOverlay, Text, TextSize, ToastContainer, ToastData, TypographyVariant,
};
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{Card, CardAnswer, NativeLanguage, StudyCard, User};
use origa::traits::UserRepository;
use std::collections::HashMap;

pub type CardsLoadedCallback = Arc<dyn Fn(&[StudyCard]) + Send + Sync>;

#[derive(Clone)]
pub struct CardListContext {
    #[expect(
        dead_code,
        reason = "read via closure capture in toggle_favorite_callback"
    )]
    pub current_user: RwSignal<Option<User>>,
    pub native_lang: Memo<NativeLanguage>,
    pub known_kanji: Memo<HashSet<char>>,
    pub on_toggle_favorite: Callback<ulid::Ulid>,
    pub on_mark_as_known: Callback<ulid::Ulid>,
    pub on_delete: Callback<DeleteRequest>,
    pub is_deleting: Signal<bool>,
    pub toasts: RwSignal<Vec<ToastData>>,
    pub all_cards: RwSignal<Vec<StudyCard>>,
    pub is_loading: RwSignal<bool>,
}

pub fn create_card_list_context(
    repository: HybridUserRepository,
    refresh_trigger: RwSignal<u32>,
    card_type_filter: fn(&Card) -> bool,
    on_cards_loaded: Option<CardsLoadedCallback>,
) -> CardListContext {
    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let is_loading = RwSignal::new(true);
    let all_cards: RwSignal<Vec<StudyCard>> = RwSignal::new(Vec::new());
    let repo_for_effect = repository.clone();

    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        let repo = repo_for_effect.clone();
        let on_loaded = on_cards_loaded.clone();
        spawn_local(async move {
            match repo.get_current_user().await {
                Ok(Some(user)) => {
                    let disposed = StoredValue::new(());
                    if disposed.is_disposed() {
                        return;
                    }
                    let cards: Vec<StudyCard> = user
                        .knowledge_set()
                        .study_cards()
                        .iter()
                        .filter(|(_, card)| card_type_filter(card.card()))
                        .map(|(_, card)| card.clone())
                        .collect();

                    if let Some(on_loaded) = on_loaded.as_ref() {
                        on_loaded(&cards);
                    }

                    if disposed.is_disposed() {
                        return;
                    }
                    all_cards.set(cards);
                    current_user.set(Some(user));
                    is_loading.set(false);
                },
                Ok(None) => {
                    tracing::warn!("CardListPage: user not found");
                    is_loading.set(false);
                },
                Err(e) => {
                    tracing::error!("CardListPage: get_current_user error: {:?}", e);
                    is_loading.set(false);
                },
            }
        });
    });

    let i18n = use_i18n();
    let native_lang =
        Memo::new(move |_| crate::i18n::locale_to_native_language(&i18n.get_locale()));

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let (on_toggle_favorite, _favorite_pending) =
        create_toggle_favorite_callback(repository.clone(), current_user, refresh_trigger);

    let (on_mark_as_known, _mark_known_pending) =
        create_mark_as_known_callback(repository.clone(), refresh_trigger);

    let toasts: RwSignal<Vec<ToastData>> = RwSignal::new(Vec::new());
    let (is_deleting, on_delete) =
        create_delete_callback(repository.clone(), toasts, refresh_trigger);

    CardListContext {
        current_user,
        native_lang,
        known_kanji,
        on_toggle_favorite,
        on_mark_as_known,
        on_delete,
        is_deleting: is_deleting.into(),
        toasts,
        all_cards,
        is_loading,
    }
}

pub fn card_list_view<F>(
    ctx: CardListContext,
    grouping: ListGrouping,
    sort_cards: bool,
    test_id_prefix: &'static str,
    empty_message: Signal<String>,
    grid_classes: Option<&'static str>,
    render_card: F,
) -> AnyView
where
    F: Fn(StudyCard) -> AnyView + Clone + Send + Sync + 'static,
{
    let i18n = use_i18n();
    let is_loading = ctx.is_loading;
    let all_cards = ctx.all_cards;
    let native_lang = ctx.native_lang;
    let toasts = ctx.toasts;

    let search = RwSignal::new(String::new());
    let filter = RwSignal::new(Filter::All);
    let visible_count: RwSignal<usize> = RwSignal::new(50);

    // Build the card_id -> JLPT level lookup once when grouping is enabled.
    // Computed only when `all_cards` changes — NOT on every filter/search tick.
    // For `ListGrouping::Flat` we skip the work entirely (empty map).
    let level_index: Memo<LevelIndex> = match grouping {
        ListGrouping::Flat => Memo::new(move |_| HashMap::new()),
        ListGrouping::ByJlptLevel { card_type } => Memo::new(move |_| {
            let content = get_jlpt_content();
            all_cards
                .get()
                .iter()
                .map(|card| {
                    (
                        *card.card_id(),
                        content.find_level(&card.card().content_key(), card_type),
                    )
                })
                .collect()
        }),
    };

    let filtered_cards = Memo::new(move |_| {
        let query = search.get().to_lowercase();
        let current_filter = filter.get();
        let lang = native_lang.get();

        let mut cards: Vec<_> = all_cards
            .get()
            .into_iter()
            .filter(|card| {
                let matches_search = query.is_empty() || {
                    let card_inner = card.card();
                    let question = card_inner.question(&lang);

                    let matches_question = question
                        .ok()
                        .is_some_and(|q| q.text().to_lowercase().contains(&query));

                    let matches_answer = match card_inner.answer(&lang).ok() {
                        Some(CardAnswer::Vocabulary {
                            translations,
                            description,
                        }) => {
                            translations
                                .iter()
                                .any(|t| t.to_lowercase().contains(&query))
                                || description
                                    .as_ref()
                                    .is_some_and(|d| d.to_lowercase().contains(&query))
                        },
                        Some(CardAnswer::Text(s)) => s.to_lowercase().contains(&query),
                        None => false,
                    };

                    matches_question || matches_answer
                };
                let matches_filter =
                    current_filter.matches(CardStatus::from_study_card(card), card.is_favorite());
                matches_search && matches_filter
            })
            .collect();

        match grouping {
            ListGrouping::ByJlptLevel { .. } => {
                // Clone the memo once here, not inside `sort_by`'s comparator.
                let index = level_index.get();
                cards = order_cards_by_group(&cards, &index);
            },
            ListGrouping::Flat => {
                if sort_cards {
                    cards.sort_by_key(|c| *c.card_id());
                }
            },
        }
        cards
    });

    Effect::new(move |_| {
        let _ = search.get();
        let _ = filter.get();
        visible_count.set(50);
    });

    let visible_cards = Memo::new(move |_| {
        filtered_cards
            .get()
            .into_iter()
            .take(visible_count.get())
            .collect::<Vec<_>>()
    });

    let counts = Memo::new(move |_| {
        let cards = all_cards.get();
        cards.iter().fold(CardCounts::default(), |mut acc, card| {
            acc.total += 1;
            if card.is_favorite() {
                acc.favorite += 1;
            }
            match CardStatus::from_study_card(card) {
                CardStatus::New => acc.new += 1,
                CardStatus::Hard => acc.hard += 1,
                CardStatus::InProgress => acc.in_progress += 1,
                CardStatus::Learned => acc.learned += 1,
            }
            acc
        })
    });

    let grid_id = Signal::derive(move || format!("{test_id_prefix}-grid"));
    let empty_id = Signal::derive(move || format!("{test_id_prefix}-empty-state"));
    let search_id = Signal::derive(move || format!("{test_id_prefix}-search-input"));
    let load_more_id = Signal::derive(move || format!("{test_id_prefix}-load-more-btn"));
    let render_card = StoredValue::new(render_card);

    let flat_grid_classes = grid_classes.unwrap_or(
        "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-4 items-start",
    );

    view! {
        <div class="space-y-4">
            <Show when=move || is_loading.get()>
                <LoadingOverlay message=Signal::derive(move || i18n.get_keys().common().loading().inner().to_string()) />
            </Show>
            <Show when=move || !is_loading.get()>
                <Input
                    value=search
                    placeholder=Signal::derive(move || i18n.get_keys().common().search().inner().to_string())
                    test_id=search_id
                />

                <div class="flex flex-wrap gap-2">
                    <FilterBtn filter=Filter::All count=move || counts.get().total active=filter test_id=format!("{test_id_prefix}-filter-all") />
                    <FilterBtn filter=Filter::New count=move || counts.get().new active=filter test_id=format!("{test_id_prefix}-filter-new") />
                    <FilterBtn filter=Filter::Hard count=move || counts.get().hard active=filter test_id=format!("{test_id_prefix}-filter-hard") />
                    <FilterBtn filter=Filter::InProgress count=move || counts.get().in_progress active=filter test_id=format!("{test_id_prefix}-filter-in-progress") />
                    <FilterBtn filter=Filter::Learned count=move || counts.get().learned active=filter test_id=format!("{test_id_prefix}-filter-learned") />
                    <FilterBtn filter=Filter::Favorite count=move || counts.get().favorite active=filter test_id=format!("{test_id_prefix}-filter-favorite") />
                </div>

                {match grouping {
                    ListGrouping::Flat => view! {
                        <div class=flat_grid_classes data-testid=move || grid_id.get()>
                            {move || {
                                let cards = visible_cards.get();
                                if cards.is_empty() {
                                    Either::Left(view! {
                                        <div class="col-span-full" data-testid=move || empty_id.get()>
                                            <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                                {empty_message.get()}
                                            </Text>
                                        </div>
                                    })
                                } else {
                                    Either::Right(view! {
                                        <For
                                            each=move || visible_cards.get()
                                            key=|card| format!("{}-{}", card.card_id(), card.is_favorite())
                                            children=move |card| {
                                                let render = render_card.with_value(|r| r.clone());
                                                render(card)
                                            }
                                        />
                                    })
                                }
                            }}
                        </div>
                    }
                    .into_any(),
                    ListGrouping::ByJlptLevel { .. } => view! {
                        <div data-testid=move || grid_id.get() class="space-y-8">
                            {move || {
                                let cards = visible_cards.get();
                                if cards.is_empty() {
                                    Either::Left(view! {
                                        <div data-testid=move || empty_id.get()>
                                            <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                                {empty_message.get()}
                                            </Text>
                                        </div>
                                    })
                                } else {
                                    let render = render_card.with_value(|r| r.clone());
                                    Either::Right(view! {
                                        <GroupedGrid
                                            cards=visible_cards
                                            level_index=level_index
                                            grid_classes=flat_grid_classes
                                            test_id_prefix=test_id_prefix
                                            render_card=render
                                        />
                                    })
                                }
                            }}
                        </div>
                    }
                    .into_any(),
                }}

                <LoadMoreButton
                    visible_count=visible_count
                    total=Signal::derive(move || filtered_cards.get().len())
                    test_id=load_more_id
                />
                <ToastContainer toasts=toasts duration_ms=5000 />
            </Show>
        </div>
    }
    .into_any()
}
