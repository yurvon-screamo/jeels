use super::super::shared::{
    CardStatus, DeleteRequest, create_delete_callback, create_mark_as_known_callback,
    format_answer_text,
};
use super::kanji_detail_parts::{KanjiDetailHeroCard, MobileOverview};
use crate::i18n::use_i18n;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    CardActionBar, DeleteConfirmModal, FsrsMetrics, FuriganaText, KanjiDrawingPractice,
    KanjiViewMode, KanjiWritingSection, LoadingOverlay, MarkdownText, ReadingGroup, TabItem, Tabs,
    Tag, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use origa::domain::{Card as DomainCard, StudyCard, User};
use origa::traits::UserRepository;
use origa::use_cases::ToggleFavoriteUseCase;
use ulid::Ulid;

fn load_study_card(
    repository: HybridUserRepository,
    card_id: Ulid,
    result_signal: RwSignal<Option<StudyCard>>,
    current_user: RwSignal<Option<User>>,
    is_loading: RwSignal<bool>,
) {
    let disposed = StoredValue::new(());
    spawn_local(async move {
        match repository.get_current_user().await {
            Ok(Some(user)) => {
                if disposed.is_disposed() {
                    return;
                }
                let found = user
                    .knowledge_set()
                    .study_cards()
                    .iter()
                    .find(|(id, _)| **id == card_id)
                    .map(|(_, card)| card.clone());
                result_signal.set(found);
                current_user.set(Some(user));
                is_loading.set(false);
            },
            Ok(None) => {
                if disposed.is_disposed() {
                    return;
                }
                tracing::warn!("KanjiDetail: user not found");
                is_loading.set(false);
            },
            Err(e) => {
                if disposed.is_disposed() {
                    return;
                }
                tracing::error!("KanjiDetail: get_current_user error: {:?}", e);
                is_loading.set(false);
            },
        }
    });
}

#[component]
pub fn KanjiDetail() -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let params = use_params_map();
    let card_id_result: Memo<Option<Ulid>> = Memo::new(move |_| {
        params
            .read()
            .get("id")
            .and_then(|id| id.parse::<Ulid>().ok())
    });

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let study_card: RwSignal<Option<StudyCard>> = RwSignal::new(None);
    let is_loading = RwSignal::new(true);
    let refresh_trigger = RwSignal::new(0u32);

    let repo_for_effect = repository.clone();
    Effect::new(move |_| {
        let _ = refresh_trigger.get();
        let Some(card_id) = card_id_result.get() else {
            is_loading.set(false);
            return;
        };
        load_study_card(
            repo_for_effect.clone(),
            card_id,
            study_card,
            current_user,
            is_loading,
        );
    });

    let is_favorite_signal: RwSignal<bool> = RwSignal::new(false);

    Effect::new(move |_| {
        if let Some(card) = study_card.get() {
            is_favorite_signal.set(card.is_favorite());
        }
    });

    let favorite_pending = RwSignal::new(false);
    let on_toggle_favorite = {
        let repo = repository.clone();
        let current_user_fav = current_user;
        let refresh = refresh_trigger;
        let pending = favorite_pending;
        Callback::new(move |card_id: Ulid| {
            is_favorite_signal.update(|f| *f = !*f);
            let repo = repo.clone();
            spawn_local(async move {
                pending.set(true);
                let use_case = ToggleFavoriteUseCase::new(&repo);
                if use_case.execute(card_id).await.is_ok() {
                    current_user_fav.update(|u| {
                        if let Some(user) = u {
                            let _ = user.toggle_favorite(card_id);
                        }
                    });
                    refresh.update(|v| *v += 1);
                } else {
                    is_favorite_signal.update(|f| *f = !*f);
                }
                pending.set(false);
            });
        })
    };
    let (on_mark_as_known, mark_known_pending) =
        create_mark_as_known_callback(repository.clone(), refresh_trigger);
    let toasts: RwSignal<Vec<crate::ui_components::ToastData>> = RwSignal::new(Vec::new());
    let (is_deleting, on_delete) =
        create_delete_callback(repository.clone(), toasts, refresh_trigger);

    let native_lang =
        Memo::new(move |_| crate::i18n::locale_to_native_language(&i18n.get_locale()));

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let is_delete_modal_open = RwSignal::new(false);
    let navigate = StoredValue::new(use_navigate());

    let not_found_text =
        Signal::derive(move || i18n.get_keys().kanji_page().not_found().inner().to_string());

    let loading_text =
        Signal::derive(move || i18n.get_keys().common().loading().inner().to_string());

    let breadcrumbs_kanji_label = Signal::derive(move || {
        i18n.get_keys()
            .home()
            .kanji()
            .inner()
            .to_string()
            .to_uppercase()
    });

    let active_tab: RwSignal<String> = RwSignal::new("overview".to_string());
    let tab_items = Signal::derive(move || {
        vec![
            TabItem {
                id: "overview".to_string(),
                label: i18n
                    .get_keys()
                    .kanji_page()
                    .tab_overview()
                    .inner()
                    .to_string(),
            },
            TabItem {
                id: "writing".to_string(),
                label: i18n
                    .get_keys()
                    .kanji_page()
                    .tab_writing()
                    .inner()
                    .to_string(),
            },
        ]
    });

    view! {
        <div class="kanji-detail-container" data-testid="kanji-detail">
            <Show when=move || is_loading.get()>
                <LoadingOverlay message=loading_text />
            </Show>

            <Show when=move || !is_loading.get() && study_card.get().is_none()>
                <div class="flex items-center justify-center py-16">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        {not_found_text}
                    </Text>
            </div>
        </Show>

        <Show when=move || study_card.get().is_some()>
            {move || {
                let card = study_card.get()?;
                let card_id = *card.card_id();
                let memory = card.memory().clone();

                let status = CardStatus::from_study_card(&card);

                let (kanji_char, radicals, on_readings, kun_readings) = match card.card() {
                    DomainCard::Kanji(kanji_card) => (
                        kanji_card.kanji().text().to_string(),
                        kanji_card.radicals_chars().into_iter().collect::<String>(),
                        kanji_card
                            .on_readings_with_freq()
                            .into_iter()
                            .map(|(r, f, rare)| crate::ui_components::ReadingItem {
                                reading: r,
                                freq: f,
                                is_rare: rare,
                            })
                            .collect::<Vec<_>>(),
                        kanji_card
                            .kun_readings_with_freq()
                            .into_iter()
                            .map(|(r, f, rare)| crate::ui_components::ReadingItem {
                                reading: r,
                                freq: f,
                                is_rare: rare,
                            })
                            .collect::<Vec<_>>(),
                    ),
                    _ => (
                        "?".to_string(),
                        String::new(),
                        Vec::new(),
                        Vec::new(),
                    ),
                };

                let card_for_desc = card.clone();
                let description = Memo::new(move |_| {
                    let lang = native_lang.get();
                    match card_for_desc.card() {
                        DomainCard::Kanji(kanji_card) => {
                            match kanji_card.description(&lang).ok() {
                                Some(_) => format_answer_text(card_for_desc.card(), &lang),
                                None => String::new(),
                            }
                        },
                        _ => String::new(),
                    }
                });

                let card_for_answer = card.clone();
                let answer_text = Memo::new(move |_| {
                    let lang = native_lang.get();
                    format_answer_text(card_for_answer.card(), &lang)
                });

                let card_for_examples = card.clone();
                let example_words = Memo::new(move |_| {
                    let lang = native_lang.get();
                    match card_for_examples.card() {
                        DomainCard::Kanji(kanji_card) => kanji_card
                            .example_words(&lang)
                            .iter()
                            .map(|w| (w.word().to_string(), w.meaning().to_string()))
                            .collect::<Vec<_>>(),
                        _ => Vec::new(),
                    }
                });

                let has_radicals = !radicals.is_empty();
                let radicals_stored: StoredValue<String> = StoredValue::new(radicals.clone());
                let has_examples = Memo::new(move |_| !example_words.get().is_empty());

                let card_id_for_delete = card_id;
                let confirm_delete = Callback::new(move |_| {
                    on_delete.run(DeleteRequest {
                        card_id: card_id_for_delete,
                        on_success: Callback::new(move |_| {
                            is_delete_modal_open.set(false);
                            navigate.get_value()("/kanji", Default::default());
                        }),
                    })
                });

                let kanji_stored: StoredValue<String> = StoredValue::new(kanji_char.clone());
                let card_id_for_known = card_id;
                let card_id_for_fav = card_id;

                let active_tab_cell = active_tab;

                let on_readings_stored: Vec<crate::ui_components::ReadingItem> =
                    on_readings.clone();
                let kun_readings_stored: Vec<crate::ui_components::ReadingItem> =
                    kun_readings.clone();
                let on_readings_mobile: Vec<crate::ui_components::ReadingItem> =
                    on_readings.clone();
                let kun_readings_mobile: Vec<crate::ui_components::ReadingItem> =
                    kun_readings.clone();
                let kanji_char_stored = kanji_char.clone();
                let breadcrumbs_label = breadcrumbs_kanji_label;

                let writing_practice_title = Signal::derive(move || {
                    i18n.get_keys().kanji_page().writing_practice().inner().to_string()
                });
                let stroke_order_title = Signal::derive(move || {
                    i18n.get_keys().kanji_page().stroke_order().inner().to_string()
                });
                let vocabulary_title = Signal::derive(move || {
                    i18n.get_keys().kanji_page().vocabulary().inner().to_string()
                });
                let radicals_title = Signal::derive(move || {
                    i18n.get_keys().kanji_page().radicals().inner().to_string()
                });
                let on_label = Signal::derive(move || {
                    i18n.get_keys().kanji_page().on_reading().inner().to_string()
                });
                let kun_label = Signal::derive(move || {
                    i18n.get_keys().kanji_page().kun_reading().inner().to_string()
                });
                let rare_hint = Signal::derive(move || {
                    i18n.get_keys().kanji_page().reading_rare_hint().inner().to_string()
                });

                Some(view! {
                    // Breadcrumbs + Actions
                    <div class="kanji-detail-top-bar">
                        <div class="kanji-breadcrumbs">
                            <A href="/kanji">{breadcrumbs_label}</A>
                            <span class="kanji-breadcrumbs-separator">"/"</span>
                            <span class="kanji-breadcrumbs-current">
                                {kanji_char_stored}
                            </span>
                        </div>
                        <FsrsMetrics
                            difficulty=memory.difficulty().map(|d| d.value())
                            stability=memory.stability().map(|s| s.value())
                            test_id=Signal::derive(|| "kanji-detail-fsrs".to_string())
                        />
                        <CardActionBar
                            tag_variant=Signal::derive(move || status.tag_variant())
                            tag_label=Signal::derive(move || status.label(&i18n))
                            is_favorite=is_favorite_signal.into()
                            on_toggle_favorite=Callback::new(move |_| on_toggle_favorite.run(card_id_for_fav))
                            favorite_pending=favorite_pending
                            show_mark_as_known=Signal::derive(move || status != CardStatus::Learned)
                            on_mark_as_known=Callback::new(move |_| on_mark_as_known.run(card_id_for_known))
                            mark_known_pending=mark_known_pending
                            on_delete=Callback::new(move |_| is_delete_modal_open.set(true))
                            test_id=Signal::derive(|| "kanji-detail-actions".to_string())
                            show_tag=Signal::derive(|| false)
                        />
                    </div>

                    // Desktop: hero+vocab | practice | stroke full-width
                    <div class="kanji-detail-grid kanji-detail-desktop">
                        // Left column: Hero + Vocabulary stacked
                        <div class="kanji-detail-left-col">
                            <div class="kanji-detail-hero-cell">
                                <KanjiDetailHeroCard
                                    kanji_stored=kanji_stored
                                    answer_text=answer_text
                                    on_readings=on_readings_stored
                                    kun_readings=kun_readings_stored
                                    has_radicals=has_radicals
                                    radicals_stored=radicals_stored
                                    radicals_title=radicals_title
                                    tag_variant=Signal::derive(move || status.tag_variant())
                                    tag_label=Signal::derive(move || status.label(&i18n))
                                    on_label=on_label
                                    kun_label=kun_label
                                    rare_hint=rare_hint
                                />
                            </div>
                            <div class="kanji-detail-vocab-cell">
                                <div class="kanji-detail-section-card">
                                    <div class="kanji-detail-section-title">{vocabulary_title}</div>
                                    <Show when=move || has_examples.get()>
                                        <div class="kanji-vocab-list">
                                            <For
                                                each=move || example_words.get()
                                                key=|(word, _)| word.clone()
                                                children=move |(word, meaning): (String, String)| {
                                                    view! {
                                                        <div class="kanji-vocab-item">
                                                            <div class="kanji-vocab-item-kanji">
                                                                {word.chars().next().unwrap_or('?').to_string()}
                                                            </div>
                                                            <div>
                                                                <div class="kanji-vocab-item-reading">
                                                                    <FuriganaText text=word.clone() known_kanji=known_kanji.get()/>
                                                                </div>
                                                                <div class="kanji-vocab-item-meaning">
                                                                    <MarkdownText content=Signal::derive(move || meaning.clone()) known_kanji=known_kanji.get()/>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    }
                                                }
                                            />
                                        </div>
                                    </Show>
                                </div>
                            </div>
                        </div>

                        // Right column: Writing Practice
                        <div class="kanji-detail-practice-cell">
                            <div class="kanji-detail-section-card">
                                <div class="kanji-detail-section-title">{writing_practice_title}</div>
                                <KanjiDrawingPractice kanji=kanji_stored.get_value() />
                            </div>
                        </div>

                        // Full-width bottom: Stroke Order
                        <div class="kanji-detail-stroke-cell">
                            <div class="kanji-detail-section-card">
                                <div class="kanji-detail-section-title">{stroke_order_title}</div>
                                <KanjiWritingSection
                                    kanji=kanji_stored.get_value()
                                    mode=KanjiViewMode::Frames
                                />
                            </div>
                        </div>
                    </div>

                    // Mobile: hero card + tabs
                    <div class="kanji-detail-mobile">
                        <div class="kanji-detail-hero-card" style="margin-bottom:16px">
                            <div class="kanji-detail-hero-header">
                                <div
                                    class="kanji-detail-hero-kanji"
                                    style="width:72px;height:72px;font-size:40px"
                                >
                                    {kanji_stored.get_value()}
                                </div>
                                <div class="kanji-detail-hero-info">
                                    <div class="kanji-detail-hero-meaning">
                                        {answer_text}
                                    </div>
                                    <div class="kanji-detail-hero-readings">
                                        <ReadingGroup
                                            label=on_label
                                            readings=StoredValue::new(Some(on_readings_mobile.clone()))
                                            rare_hint=rare_hint
                                            test_id=Signal::derive(|| "kanji-detail-on-readings-mobile".to_string())
                                        />
                                        <ReadingGroup
                                            label=kun_label
                                            readings=StoredValue::new(Some(kun_readings_mobile.clone()))
                                            rare_hint=rare_hint
                                            test_id=Signal::derive(|| "kanji-detail-kun-readings-mobile".to_string())
                                        />
                                    </div>
                                </div>
                                <div class="kanji-detail-hero-badge">
                                    <Tag variant=Signal::derive(move || status.tag_variant())>
                                        {move || status.label(&i18n)}
                                    </Tag>
                                </div>
                            </div>
                            <div class="kanji-detail-hero-actions">
                                <FsrsMetrics
                                    difficulty=memory.difficulty().map(|d| d.value())
                                    stability=memory.stability().map(|s| s.value())
                                    test_id=Signal::derive(|| "kanji-detail-fsrs-mobile".to_string())
                                />
                                <CardActionBar
                                    tag_variant=Signal::derive(move || status.tag_variant())
                                    tag_label=Signal::derive(move || status.label(&i18n))
                                    is_favorite=is_favorite_signal.into()
                                    on_toggle_favorite=Callback::new(move |_| {
                                        on_toggle_favorite.run(card_id_for_fav)
                                    })
                                    favorite_pending=favorite_pending
                                    show_mark_as_known=Signal::derive(move || {
                                        status != CardStatus::Learned
                                    })
                                    on_mark_as_known=Callback::new(move |_| {
                                        on_mark_as_known.run(card_id_for_known)
                                    })
                                    mark_known_pending=mark_known_pending
                                    on_delete=Callback::new(move |_| {
                                        is_delete_modal_open.set(true)
                                    })
                                    test_id=Signal::derive(|| {
                                        "kanji-detail-actions-mobile".to_string()
                                    })
                                    show_tag=Signal::derive(|| false)
                                />
                            </div>
                        </div>

                        <div class="kanji-detail-section">
                            <Tabs
                                tabs=tab_items
                                active=active_tab_cell
                                test_id=Signal::derive(|| "kanji-detail-tabs".to_string())
                                class="tabs--underline".to_string()
                            />
                        </div>

                        <Show when=move || active_tab_cell.get() == "overview">
                            <MobileOverview
                                description=description
                                has_radicals=has_radicals
                                radicals_stored=radicals_stored
                                radicals_title=radicals_title
                                has_examples=has_examples
                                vocabulary_title=vocabulary_title
                                example_words=example_words
                                known_kanji=known_kanji.get()
                            />
                        </Show>

                        <Show when=move || active_tab_cell.get() == "writing">
                            <div class="kanji-detail-section">
                                <KanjiWritingSection
                                    kanji=kanji_stored.get_value()
                                    mode=KanjiViewMode::Frames
                                />
                            </div>
                            <div class="kanji-detail-section">
                                <KanjiDrawingPractice kanji=kanji_stored.get_value() />
                            </div>
                        </Show>
                    </div>

                    <DeleteConfirmModal
                        test_id="kanji-detail-delete-modal"
                        is_open=is_delete_modal_open
                        is_deleting=is_deleting.into()
                        on_confirm=confirm_delete
                        on_close=Callback::new(move |_| is_delete_modal_open.set(false))
                    />
                }.into_any())
            }}
        </Show>
        </div>
    }
}
