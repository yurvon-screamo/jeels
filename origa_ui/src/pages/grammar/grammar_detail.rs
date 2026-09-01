use std::collections::HashSet;

use super::super::shared::{
    CardStatus, DeleteRequest, create_delete_callback, create_mark_as_known_callback,
};
use super::grammar_detail_hero_card::GrammarDetailHeroCard;
use super::grammar_detail_mobile::GrammarMobileOverview;
use super::grammar_practice_session::GrammarPracticeSession;
use super::grammar_warnings::GrammarWarnings;
use super::nuances_section::NuancesSection;
use super::related_pattern_list::RelatedPatternList;
use crate::i18n::use_i18n;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    CardActionBar, DeleteConfirmModal, FsrsMetrics, FuriganaText, LoadingOverlay, MarkdownText,
    TabItem, Tabs, Tag, Text, TextSize, TypographyVariant,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_use::use_media_query;
use origa::dictionary::grammar::{GrammarRule, get_rule_by_id};
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
                tracing::warn!("GrammarDetail: user not found");
                is_loading.set(false);
            },
            Err(e) => {
                if disposed.is_disposed() {
                    return;
                }
                tracing::error!("GrammarDetail: get_current_user error: {:?}", e);
                is_loading.set(false);
            },
        }
    });
}

fn extract_grammar_rule(study_card: &StudyCard) -> Option<&'static GrammarRule> {
    match study_card.card() {
        DomainCard::Grammar(grammar) => get_rule_by_id(grammar.rule_id()),
        _ => None,
    }
}

#[component]
pub fn GrammarDetail() -> impl IntoView {
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
    let active_tab: RwSignal<String> = RwSignal::new("overview".to_string());

    // The practice session is stateful (current question, score, keyboard
    // handler), so it must mount at most once. use_media_query renders it in
    // exactly one of the desktop/mobile layouts, avoiding duplicate state and
    // test-id collisions. Breakpoint mirrors the .grammar-detail-desktop/-mobile
    // CSS split (1024px).
    let is_desktop = use_media_query("(min-width: 1024px)");

    let has_quiz_signal = Memo::new(move |_| {
        study_card
            .with(|sc| sc.as_ref().and_then(extract_grammar_rule))
            .map(|r| r.has_format_map())
            .unwrap_or(false)
    });

    let not_found_text = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .not_found()
            .inner()
            .to_string()
    });
    let loading_text =
        Signal::derive(move || i18n.get_keys().common().loading().inner().to_string());

    let breadcrumbs_label = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .header()
            .inner()
            .to_string()
            .to_uppercase()
    });

    let tab_items = Signal::derive(move || {
        let mut items = vec![TabItem {
            id: "overview".to_string(),
            label: i18n
                .get_keys()
                .grammar_page()
                .tab_overview()
                .inner()
                .to_string(),
        }];
        if has_quiz_signal.get() {
            items.push(TabItem {
                id: "practice".to_string(),
                label: i18n
                    .get_keys()
                    .grammar_page()
                    .tab_practice()
                    .inner()
                    .to_string(),
            });
        }
        items
    });

    let practice_title = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .practice()
            .inner()
            .to_string()
    });
    let explanation_title = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .explanation()
            .inner()
            .to_string()
    });
    let how_to_form_title = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .how_to_form()
            .inner()
            .to_string()
    });
    let examples_title = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .examples()
            .inner()
            .to_string()
    });
    let nuances_title =
        Signal::derive(move || i18n.get_keys().grammar_page().nuances().inner().to_string());
    let pro_tip_title =
        Signal::derive(move || i18n.get_keys().grammar_page().pro_tip().inner().to_string());
    let related_title = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .related_patterns()
            .inner()
            .to_string()
    });

    view! {
        <div class="grammar-detail-container" data-testid="grammar-detail-container">
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
                    let grammar_rule = extract_grammar_rule(&card);

                    let title_text = match card.card() {
                        DomainCard::Grammar(grammar) => {
                            let lang = native_lang.get();
                            grammar.title(&lang).ok().map(|t| t.text().to_string()).unwrap_or_default()
                        },
                        _ => "?".to_string(),
                    };

                    let short_description = Memo::new(move |_| {
                        grammar_rule
                            .map(|r| r.content(&native_lang.get()).short_description().to_string())
                            .unwrap_or_default()
                    });

                    let explanation = Memo::new(move |_| {
                        grammar_rule.map(|r| r.content(&native_lang.get()).explanation().to_string())
                    });
                    let warnings = Memo::new(move |_| {
                        grammar_rule
                            .map(|r| r.content(&native_lang.get()).warnings().to_vec())
                            .unwrap_or_default()
                    });
                    let how_to_form = Memo::new(move |_| {
                        grammar_rule.map(|r| r.content(&native_lang.get()).how_to_form().to_string())
                    });
                    let examples = Memo::new(move |_| {
                        grammar_rule.map(|r| r.content(&native_lang.get()).examples().to_string())
                    });
                    let nuances = Memo::new(move |_| {
                        grammar_rule.map(|r| r.content(&native_lang.get()).nuances().clone())
                    });
                    let pro_tip = Memo::new(move |_| {
                        grammar_rule.map(|r| r.content(&native_lang.get()).pro_tip().to_string())
                    });
                    let related_patterns = Memo::new(move |_| {
                        grammar_rule
                            .map(|r| r.content(&native_lang.get()).related_patterns().to_vec())
                            .unwrap_or_default()
                    });
                    let title_stored: StoredValue<String> = StoredValue::new(title_text.clone());
                    let known_kanji_stored: StoredValue<HashSet<char>> =
                        StoredValue::new(known_kanji.get());

                    let card_id_for_delete = card_id;
                    let confirm_delete = Callback::new(move |_| {
                        on_delete.run(DeleteRequest {
                            card_id: card_id_for_delete,
                            on_success: Callback::new(move |_| {
                                is_delete_modal_open.set(false);
                                navigate.get_value()("/grammar", Default::default());
                            }),
                        })
                    });

                    let card_id_for_fav = card_id;
                    let card_id_for_known = card_id;
                    let active_tab_cell = active_tab;
                    let breadcrumbs_label_val = breadcrumbs_label;

                    Some(view! {
                        <div class="grammar-detail-top-bar">
                            <div class="grammar-breadcrumbs" data-testid="grammar-detail-breadcrumbs">
                                <A href="/grammar" attr:data-testid="grammar-detail-breadcrumbs-back">{breadcrumbs_label_val}</A>
                                <span class="grammar-breadcrumbs-separator">"/"</span>
                                <span class="grammar-breadcrumbs-current">
                                    {title_stored.get_value()}
                                </span>
                            </div>
                            <FsrsMetrics
                                difficulty=memory.difficulty().map(|d| d.value())
                                stability=memory.stability().map(|s| s.value())
                                test_id=Signal::derive(|| "grammar-detail-fsrs".to_string())
                            />
                            <div data-testid="grammar-detail-actions">
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
                                    test_id=Signal::derive(|| "grammar-detail-actions".to_string())
                                    show_tag=Signal::derive(|| false)
                                />
                            </div>
                        </div>

                        // Desktop layout
                        <div class="grammar-detail-grid grammar-detail-desktop">
                            <div class="grammar-detail-left-col">
                                <GrammarDetailHeroCard
                                    title_stored=title_stored
                                    short_description=short_description
                                    tag_variant=Signal::derive(move || status.tag_variant())
                                    tag_label=Signal::derive(move || status.label(&i18n))
                                    known_kanji=known_kanji_stored.get_value()
                                />

                                <Show when=move || explanation.get().is_some_and(|s| !s.is_empty())>
                                    <div class="grammar-detail-section-card">
                                        <div class="grammar-detail-section-title">{explanation_title}</div>
                                        <MarkdownText
                                            content=Signal::derive(move || explanation.get().unwrap_or_default())
                                            known_kanji=known_kanji_stored.get_value()
                                        />
                                        <Show when=move || !warnings.get().is_empty()>
                                            <GrammarWarnings
                                                warnings=warnings.get()
                                                known_kanji=known_kanji_stored.get_value()
                                                test_id=Signal::derive(|| "grammar-detail-warnings".to_string())
                                            />
                                        </Show>
                                    </div>
                                </Show>

                                <Show when=move || examples.get().is_some_and(|s| !s.is_empty())>
                                    <div class="grammar-detail-section-card">
                                        <div class="grammar-detail-section-title">{examples_title}</div>
                                        <MarkdownText
                                            content=Signal::derive(move || examples.get().unwrap_or_default())
                                            known_kanji=known_kanji_stored.get_value()
                                        />
                                    </div>
                                </Show>
                            </div>

                            <div class="grammar-detail-right-col">
                                <Show when=move || how_to_form.get().is_some_and(|s| !s.is_empty())>
                                    <div class="grammar-detail-section-card">
                                        <div class="grammar-detail-section-title">{how_to_form_title}</div>
                                        <MarkdownText
                                            content=Signal::derive(move || how_to_form.get().unwrap_or_default())
                                            known_kanji=known_kanji_stored.get_value()
                                        />
                                    </div>
                                </Show>

                                <Show when=move || nuances.get().is_some_and(|n| !n.is_empty())>
                                    <div class="grammar-detail-section-card">
                                        <div class="grammar-detail-section-title">{nuances_title}</div>
                                        {move || {
                                            let nuances = nuances.get()?;
                                            Some(view! {
                                                <NuancesSection
                                                    common_mistakes=nuances.common_mistakes().to_vec()
                                                    notes=nuances.notes().to_vec()
                                                    known_kanji=known_kanji_stored.get_value()
                                                    test_id=Signal::derive(|| "grammar-detail-nuances".to_string())
                                                />
                                            }.into_any())
                                        }}
                                    </div>
                                </Show>

                                <Show when=move || pro_tip.get().is_some_and(|s| !s.is_empty())>
                                    <div class="grammar-detail-section-card">
                                        <div class="grammar-detail-section-title">{pro_tip_title}</div>
                                        <MarkdownText
                                            content=Signal::derive(move || pro_tip.get().unwrap_or_default())
                                            known_kanji=known_kanji_stored.get_value()
                                        />
                                    </div>
                                </Show>

                                <Show when=move || !related_patterns.get().is_empty()>
                                    <div class="grammar-detail-section-card">
                                        <div class="grammar-detail-section-title">{related_title}</div>
                                        <RelatedPatternList
                                            related=related_patterns.get()
                                            native_language=native_lang.get()
                                            current_user=current_user.get()
                                            known_kanji=known_kanji_stored.get_value()
                                            test_id=Signal::derive(|| "grammar-detail-related".to_string())
                                        />
                                    </div>
                                </Show>
                            </div>
                        </div>

                        // Desktop inline practice (only when the rule supports quizzes)
                        <Show when=move || is_desktop.get() && has_quiz_signal.get()>
                            {move || {
                                let rule = grammar_rule?;
                                let user = current_user.get()?;
                                Some(view! {
                                    <div class="grammar-detail-desktop">
                                        <div class="grammar-detail-section-card">
                                            <div class="grammar-detail-section-title">{practice_title}</div>
                                            <GrammarPracticeSession
                                                rule=rule
                                                user=user
                                                known_kanji=known_kanji.get()
                                            />
                                        </div>
                                    </div>
                                }.into_any())
                            }}
                        </Show>

                        // Mobile layout
                        <div class="grammar-detail-mobile">
                            <div class="grammar-detail-hero-card" style="margin-bottom:16px">
                                <div class="grammar-detail-hero-header">
                                    <div class="grammar-detail-hero-form" style="font-size:28px">
                                        <FuriganaText
                                            text=title_stored.get_value()
                                            known_kanji=known_kanji_stored.get_value()
                                        />
                                    </div>
                                    <Show when=move || !short_description.get().is_empty()>
                                        <div class="grammar-detail-hero-meaning">{short_description}</div>
                                    </Show>
                                    <div class="grammar-detail-hero-badge">
                                        <Tag variant=Signal::derive(move || status.tag_variant())>
                                            {move || status.label(&i18n)}
                                        </Tag>
                                    </div>
                                </div>
                                <div class="grammar-detail-hero-actions">
                                    <FsrsMetrics
                                        difficulty=memory.difficulty().map(|d| d.value())
                                        stability=memory.stability().map(|s| s.value())
                                        test_id=Signal::derive(|| "grammar-detail-fsrs-mobile".to_string())
                                    />
                                    <div data-testid="grammar-detail-actions-mobile">
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
                                            test_id=Signal::derive(|| "grammar-detail-actions-mobile".to_string())
                                            show_tag=Signal::derive(|| false)
                                        />
                                    </div>
                                </div>
                            </div>

                            <div class="grammar-detail-section">
                                <Tabs
                                    tabs=tab_items
                                    active=active_tab_cell
                                    test_id=Signal::derive(|| "grammar-detail-tabs".to_string())
                                    class="tabs--underline".to_string()
                                />
                            </div>

                            <Show when=move || active_tab_cell.get() == "overview">
                                <GrammarMobileOverview
                                    explanation=explanation
                                    how_to_form=how_to_form
                                    examples=examples
                                    nuances=nuances
                                    pro_tip=pro_tip
                                    related_patterns=related_patterns
                                    warnings=warnings
                                    native_language=native_lang.get()
                                    current_user=current_user.get()
                                    explanation_title=explanation_title
                                    how_to_form_title=how_to_form_title
                                    examples_title=examples_title
                                    nuances_title=nuances_title
                                    pro_tip_title=pro_tip_title
                                    related_title=related_title
                                    known_kanji=known_kanji_stored.get_value()
                                />
                            </Show>

                            <Show when=move || active_tab_cell.get() == "practice">
                                <div class="grammar-detail-section">
                                    <div class="grammar-detail-section-card">
                                        <div class="grammar-detail-section-title">{practice_title}</div>
                                        <Show when=move || !is_desktop.get()>
                                            {move || {
                                                let rule = grammar_rule?;
                                                let user = current_user.get()?;
                                                Some(view! {
                                                    <GrammarPracticeSession
                                                        rule=rule
                                                        user=user
                                                        known_kanji=known_kanji.get()
                                                    />
                                                }.into_any())
                                            }}
                                        </Show>
                                    </div>
                                </div>
                            </Show>
                        </div>

                        <DeleteConfirmModal
                            test_id="grammar-detail-delete-modal"
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
