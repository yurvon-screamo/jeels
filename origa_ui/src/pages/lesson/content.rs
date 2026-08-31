use super::acquaintance_slides::build_acquaintance_slides;
use super::acquaintance_state::{
    AcquaintanceContext, AcquaintanceSlideData, AcquaintanceStage, AcquaintanceState,
};
use super::acquaintance_view::AcquaintanceView;
use super::complete_screen::LessonCompleteScreen;
use super::empty_state_view::LessonEmptyState;
use super::header::LessonHeader;
use super::lesson_card_container::LessonCardContainer;
use super::lesson_state::{LessonContext, LessonMode, LessonState};
use crate::i18n::*;
use crate::loaders::phrase_data_loader::load_phrase_details_batch;
use crate::repository::HybridUserRepository;
use crate::store::auth_store::AuthStore;
use crate::ui_components::{Spinner, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{Card, LessonEmptyDiagnosis, diagnose_empty_lesson};
use origa::traits::UserRepository;
use origa::use_cases::SelectAcquaintanceHandUseCase;
use origa::use_cases::SelectCardsToLessonUseCase;
use origa::use_cases::{classify_orphaned_phrases, delete_phrase_cards_by_phrase_ids};
use std::collections::HashSet;
use ulid::Ulid;

/// Parses `?mode=grammar_practice&grammar_id=<ulid>` from a raw query string.
///
/// Returns `Some(LessonMode::GrammarPractice)` only when both parameters are
/// present and the grammar id is a valid Ulid. Returns `None` for the normal
/// lesson flow.
///
/// Gated by the `grammar_practice_lesson_mode` feature flag so the wire format
/// can evolve without affecting the default build.
#[cfg(feature = "grammar_practice_lesson_mode")]
fn parse_grammar_practice_query(raw_query: &str) -> Option<LessonMode> {
    let stripped = raw_query.trim_start_matches('?');
    let mode = stripped
        .split('&')
        .find_map(|pair| pair.strip_prefix("mode="))?;
    if mode != "grammar_practice" {
        return None;
    }
    let grammar_id_raw = stripped
        .split('&')
        .find_map(|pair| pair.strip_prefix("grammar_id="))?;
    let grammar_rule_id = Ulid::from_string(grammar_id_raw).ok()?;
    Some(LessonMode::GrammarPractice { grammar_rule_id })
}

#[component]
pub fn LessonContent() -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");

    #[cfg(feature = "grammar_practice_lesson_mode")]
    let resolved_mode: LessonMode = {
        let raw_query = web_sys::window()
            .and_then(|w| w.location().search().ok())
            .unwrap_or_default();
        parse_grammar_practice_query(&raw_query).unwrap_or_default()
    };
    #[cfg(not(feature = "grammar_practice_lesson_mode"))]
    let resolved_mode: LessonMode = LessonMode::default();

    let resolved_mode = StoredValue::new(resolved_mode);

    let lesson_state = RwSignal::new(LessonState::default());
    let is_loading = RwSignal::new(true);
    let is_completed = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);
    let empty_diagnosis = RwSignal::new(None::<LessonEmptyDiagnosis>);
    // Диагноз пустого ревью при активной руке: показывается, когда рука
    // закроется и обычный урок остаётся пустым.
    let pending_empty_diagnosis = RwSignal::new(None::<LessonEmptyDiagnosis>);
    let reload_trigger = RwSignal::new(0u32);
    let is_muted = RwSignal::new(false);
    let is_syncing_cards = RwSignal::new(false);
    let known_kanji = RwSignal::new(HashSet::<char>::new());
    // get_locale_untracked: инициализация один раз при монтировании,
    // подписка не нужна (локаль меняется перезагрузкой) — иначе
    // console-warning о доступе вне реактивного контекста.
    let native_language = RwSignal::new(crate::i18n::locale_to_native_language(
        &i18n.get_locale_untracked(),
    ));
    let core_count_signal = RwSignal::new(0usize);

    let is_disposed = StoredValue::new(());
    provide_context(is_disposed);

    Effect::new(move |_| {
        native_language.set(crate::i18n::locale_to_native_language(&i18n.get_locale()));
    });

    let lesson_ctx = LessonContext {
        repository: repository.clone(),
        lesson_state,
        is_completed,
        reload_trigger,
        is_muted,
        known_kanji,
        native_language,
        core_count: core_count_signal,
    };
    provide_context(lesson_ctx);

    {
        let acquaintance_state = RwSignal::new(AcquaintanceState::default());
        let acquaintance_slides = RwSignal::new(Vec::<AcquaintanceSlideData>::new());
        provide_context(AcquaintanceContext {
            repository: repository.clone(),
            state: acquaintance_state,
            slides: acquaintance_slides,
            known_kanji,
            native_language,
            current_card: RwSignal::new(None),
            showing_answer: RwSignal::new(false),
        });
    }

    // Активна ли сейчас префаза руки — производное от стадии в домене.
    let acq_ctx = use_context::<AcquaintanceContext>().expect("acquaintance context not provided");
    let acq_state_signal = acq_ctx.state;
    let acq_slides_signal = acq_ctx.slides;
    let acq_hand_active = move || {
        acq_state_signal
            .with(|state| state.stage != AcquaintanceStage::Inactive && state.hand.is_some())
    };

    // Закрытие руки открывает обычный урок. Если ревью пусто, юзер
    // раньше получал два бесполезных слайда подряд — переходный экран
    // руки, затем полу-пустой empty-state урока (юзер-репорт): теперь
    // закрытие руки при пустом ревью завершает урок сразу — экран
    // завершения с «Следующий урок» / «На главную».
    {
        let pending = pending_empty_diagnosis;
        let diagnosis = empty_diagnosis;
        let completed = is_completed;
        Effect::new(move |_| {
            let stage_completed =
                acq_state_signal.with(|state| state.stage == AcquaintanceStage::Completed);
            if stage_completed && pending.get_untracked().is_some() {
                pending.set(None);
                completed.set(true);
                return;
            }
            // Fallback: рука ушла в Inactive с живым отложенным
            // диагнозом (прерывание) — штатный empty-state урока.
            if !acq_hand_active() && pending.get_untracked().is_some() {
                diagnosis.set(pending.get_untracked());
                pending.set(None);
            }
        });
    }

    let repo_for_user_data = repository.clone();
    Effect::new(move |_| {
        let repo = repo_for_user_data.clone();
        spawn_local(async move {
            if let Ok(Some(user)) = repo.get_current_user().await {
                if is_disposed.is_disposed() {
                    return;
                }
                known_kanji.set(user.knowledge_set().get_known_kanji());
            }
        });
    });

    Effect::new(move |_| {
        if !is_loading.get_untracked() {
            is_syncing_cards.set(true);
        }

        reload_trigger.set(reload_trigger.get_untracked() + 1);
    });

    Effect::new(move |_| {
        reload_trigger.get();

        if !auth_store.is_all_data_loaded().get() {
            return;
        }

        let repo = repository.clone();
        spawn_local(async move {
            if is_disposed.is_disposed() {
                return;
            }
            is_loading.set(true);
            error_message.set(None);
            empty_diagnosis.set(None);
            pending_empty_diagnosis.set(None);
            acq_state_signal.update(|state| *state = Default::default());

            let jlpt_content = crate::loaders::get_jlpt_content();

            // Рука знакомства — только нормальный урок: спец-режим практики
            // грамматики строится по конкретному правилу и новые карты
            // не вводит.
            #[cfg(feature = "grammar_practice_lesson_mode")]
            let is_gated_practice_mode = matches!(
                resolved_mode.get_value(),
                LessonMode::GrammarPractice { .. }
            );
            #[cfg(not(feature = "grammar_practice_lesson_mode"))]
            let is_gated_practice_mode = false;

            let (hand_order, hand_user_snapshot) = if is_gated_practice_mode {
                (Vec::new(), None)
            } else {
                let select_hand = SelectAcquaintanceHandUseCase::new(&repo);
                match select_hand.execute(jlpt_content).await {
                    Ok(Some(ids)) if !ids.is_empty() => match repo.get_current_user().await {
                        Ok(Some(user)) => (ids, Some(user)),
                        other => {
                            tracing::error!(
                                "Acquaintance hand skipped: user unavailable ({other:?})"
                            );
                            (Vec::new(), None)
                        },
                    },
                    // Легитимная пустота: пул исчерпан или лимит дня.
                    Ok(_) => {
                        tracing::debug!("Acquaintance hand: pool empty or limit exhausted");
                        (Vec::new(), None)
                    },
                    Err(e) => {
                        tracing::error!("Acquaintance hand selection failed: {e}");
                        (Vec::new(), None)
                    },
                }
            };

            if !hand_order.is_empty() {
                if let Some(user) = &hand_user_snapshot {
                    let pairs: Vec<(Ulid, origa::domain::CardType)> = hand_order
                        .iter()
                        .filter_map(|card_id| {
                            user.knowledge_set().get_card(*card_id).map(|study_card| {
                                (*card_id, origa::domain::CardType::from(study_card.card()))
                            })
                        })
                        .collect();
                    let built_hand = origa::domain::AcquaintanceHand::new(pairs);
                    if let Err(e) = &built_hand {
                        tracing::error!("Acquaintance hand build failed: {e}");
                    }
                    if let Ok(hand) = built_hand {
                        let slides = build_acquaintance_slides(
                            user,
                            &hand.presentation_order(),
                            native_language.get_untracked(),
                            &i18n,
                        );
                        acq_state_signal.update(|state| {
                            state.stage = AcquaintanceStage::Presentation;
                            state.hand = Some(hand);
                            state.slide_index = 0;
                            state.skipped_ids.clear();
                        });
                        acq_slides_signal.set(slides);
                    }
                }
            }

            let use_case = SelectCardsToLessonUseCase::new(&repo);
            let new_card_policy = origa::domain::NewCardPolicy::Exclude;
            let cards_result = use_case.execute(new_card_policy, jlpt_content).await;

            if is_disposed.is_disposed() {
                return;
            }

            match cards_result {
                Ok(mut lesson_data) => {
                    let phrase_ids: Vec<Ulid> = lesson_data
                        .cards
                        .iter()
                        .filter_map(|(_, lc)| {
                            if let Card::Phrase(pc) = lc.view().card() {
                                Some(*pc.phrase_id())
                            } else {
                                None
                            }
                        })
                        .collect();

                    if !phrase_ids.is_empty() {
                        let results = load_phrase_details_batch(&phrase_ids).await;

                        let failed_phrase_ids: Vec<Ulid> = phrase_ids
                            .iter()
                            .zip(results.iter())
                            .filter_map(|(id, result)| result.as_ref().err().map(|_| *id))
                            .collect();

                        if !failed_phrase_ids.is_empty() {
                            let (permanent, _transient) =
                                classify_orphaned_phrases(&failed_phrase_ids);

                            let failed_set: HashSet<Ulid> = failed_phrase_ids.into_iter().collect();
                            let mut cards_to_delete: Vec<Ulid> = Vec::new();

                            lesson_data.cards.retain(|(card_id, lc)| {
                                if let Card::Phrase(pc) = lc.view().card() {
                                    let phrase_id = pc.phrase_id();
                                    if failed_set.contains(phrase_id) {
                                        if permanent.contains(phrase_id) {
                                            cards_to_delete.push(*card_id);
                                        }
                                        return false;
                                    }
                                }
                                true
                            });

                            if !cards_to_delete.is_empty() {
                                if let Ok(Some(mut user)) = repo.get_current_user().await {
                                    let deleted =
                                        delete_phrase_cards_by_phrase_ids(&mut user, &permanent);
                                    if deleted > 0 {
                                        if let Err(e) = repo.save(&user).await {
                                            tracing::warn!(
                                                "Failed to save user after phrase cleanup: {e}"
                                            );
                                        }
                                    }
                                }
                                tracing::warn!(
                                    deleted = cards_to_delete.len(),
                                    phrase_ids = ?permanent.iter().take(5).collect::<Vec<_>>(),
                                    "Removed permanently missing phrase cards from user deck"
                                );
                            } else {
                                tracing::warn!(
                                    count = failed_set.len(),
                                    "Filtered transient-failed phrases from lesson (not deleting from deck)"
                                );
                            }
                        }
                    }

                    let card_ids = lesson_data.card_ids();
                    let cards = lesson_data.cards_map();
                    let core_count = lesson_data.core_count;
                    core_count_signal.set(core_count);
                    if cards.is_empty() {
                        // `lesson.no_cards` remains the fallback ONLY for
                        // feature-gated modes (grammar practice): their
                        // emptiness has different causes the Normal-mode
                        // diagnosis does not model.
                        #[cfg(feature = "grammar_practice_lesson_mode")]
                        let is_gated_mode = matches!(
                            resolved_mode.get_value(),
                            LessonMode::GrammarPractice { .. }
                        );
                        #[cfg(not(feature = "grammar_practice_lesson_mode"))]
                        let is_gated_mode = false;

                        if is_gated_mode {
                            error_message.set(Some(
                                i18n.get_keys_untracked()
                                    .lesson()
                                    .no_cards()
                                    .inner()
                                    .to_string(),
                            ));
                        } else {
                            // Single extra fetch, only on the (rare) empty
                            // path: the diagnosis needs the user's daily
                            // load, which the lesson selection did not carry.
                            let diagnosis = match repo.get_current_user().await {
                                Ok(Some(user)) => Some(diagnose_empty_lesson(
                                    user.knowledge_set(),
                                    *user.daily_load(),
                                )),
                                Ok(None) => {
                                    tracing::warn!(
                                        "Empty lesson diagnosis degraded: no current user"
                                    );
                                    Some(LessonEmptyDiagnosis::default())
                                },
                                Err(e) => {
                                    tracing::warn!("Empty lesson diagnosis degraded: {e}");
                                    Some(LessonEmptyDiagnosis::default())
                                },
                            };
                            if is_disposed.is_disposed() {
                                return;
                            }
                            if acq_hand_active() {
                                // Рука активна: пустое ревью не прячут её —
                                // диагноз откладывается до закрытия руки.
                                pending_empty_diagnosis.set(diagnosis);
                            } else {
                                empty_diagnosis.set(diagnosis);
                            }
                        }
                    } else {
                        lesson_state.set(LessonState {
                            mode: resolved_mode.get_value().clone(),
                            cards,
                            card_ids,
                            current_index: 0,
                            showing_answer: false,
                            review_count: 0,
                            selected_quiz_option: None,
                            selected_yesno_answer: None,
                            dont_know_selected: false,
                            core_count,
                            waiting_for_next: false,
                            pending_rating: None,
                            selected_quiz_options: HashSet::new(),
                            multi_quiz_submitted: false,
                            multi_result: None,
                        });
                    }
                },
                Err(e) => {
                    error_message.set(Some(
                        i18n.get_keys_untracked()
                            .lesson()
                            .load_error()
                            .inner()
                            .replace("{}", &e.to_string()),
                    ));
                },
            }

            is_loading.set(false);
            is_syncing_cards.set(false);
        });
    });

    // Мемоизированный гейт: Memo<bool> не уведомляет подписчиков при
    // true→true, поэтому Show не пересоздаёт AcquaintanceView на каждом
    // ответе тренировки (record_answer обновляет state, а с ним и условия).
    // Пересоздание сбрасывало локальные rotation_index/showing_answer
    // тренировки — порядок «хаотился» и смена стороны была недостижима.
    let show_acquaintance = Memo::new(move |_| {
        acq_hand_active()
            && !is_loading.get()
            && !is_completed.get()
            && error_message.get().is_none()
    });

    let show_lesson_content = move || {
        !acq_hand_active()
            && !is_loading.get()
            && !is_completed.get()
            && error_message.get().is_none()
            && empty_diagnosis.get().is_none()
    };

    view! {
        <LessonHeader />

        <Show when=move || is_loading.get()>
            <div data-testid="lesson-loading" class="flex flex-col items-center py-8 gap-4">
                <Spinner test_id="lesson-spinner" />
                <Text size=TextSize::Default variant=TypographyVariant::Muted test_id="lesson-loading-text">
                    {t!(i18n, lesson.loading)}
                </Text>
            </div>
        </Show>

        <Show when=move || error_message.get().is_some() && !is_loading.get()>
            <div data-testid="lesson-error" class="text-center py-8">
                <Text size=TextSize::Default variant=TypographyVariant::Muted>
                    {move || error_message.get().unwrap_or_default()}
                </Text>
            </div>
        </Show>

        <LessonEmptyState diagnosis=empty_diagnosis />

        <Show when=move || is_completed.get()>
            <LessonCompleteScreen
                is_completed
                review_count=lesson_state.get().review_count
            />
        </Show>

        <Show when=move || show_acquaintance.get()>
            <AcquaintanceView />
        </Show>

        <Show when=show_lesson_content>
            <div data-testid="lesson-content" class="relative px-0.5 sm:px-1 py-1 sm:py-2">
                <Show when=move || is_syncing_cards.get()>
                    <div data-testid="lesson-sync-indicator" class="absolute top-0 right-0 flex items-center gap-1 text-sm text-muted-foreground p-2">
                        <Spinner test_id="lesson-sync-spinner" class=Signal::derive(|| "".to_string()) size=Signal::derive(|| "sm".to_string()) />
                        {t!(i18n, lesson.syncing)}
                    </div>
                </Show>

                <LessonCardContainer />
            </div>
        </Show>
    }
}
