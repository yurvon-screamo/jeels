use super::complete_screen::LessonCompleteScreen;
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
use origa::domain::Card;
use origa::traits::UserRepository;
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
    let reload_trigger = RwSignal::new(0u32);
    let is_muted = RwSignal::new(false);
    let is_syncing_cards = RwSignal::new(false);
    let known_kanji = RwSignal::new(HashSet::<char>::new());
    let native_language = RwSignal::new(crate::i18n::locale_to_native_language(&i18n.get_locale()));
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

            let use_case = SelectCardsToLessonUseCase::new(&repo);
            let jlpt_content = crate::loaders::get_jlpt_content();
            let cards_result = use_case.execute(jlpt_content).await;

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
                        error_message.set(Some(
                            i18n.get_keys_untracked()
                                .lesson()
                                .no_cards()
                                .inner()
                                .to_string(),
                        ));
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

        <Show when=move || is_completed.get()>
            <LessonCompleteScreen
                is_completed
                review_count=lesson_state.get().review_count
            />
        </Show>

        <Show when=move || !is_loading.get() && !is_completed.get() && error_message.get().is_none()>
            // No nested scroll container here — the whole page scrolls.
            // A separate scroll layer over the lesson zone used to fight the
            // page scroll on mobile (jitter + snap-back) and was removed.
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

#[cfg(all(test, feature = "grammar_practice_lesson_mode"))]
mod tests {
    use super::*;

    #[test]
    fn parse_query_returns_grammar_practice_for_valid_input() {
        let id = Ulid::new();
        let query = format!("mode=grammar_practice&grammar_id={id}");
        let parsed = parse_grammar_practice_query(&query);
        match parsed {
            Some(LessonMode::GrammarPractice { grammar_rule_id }) => {
                assert_eq!(grammar_rule_id, id);
            },
            other => panic!("expected GrammarPractice, got {other:?}"),
        }
    }

    #[test]
    fn parse_query_returns_none_for_normal_mode() {
        assert!(parse_grammar_practice_query("mode=normal").is_none());
        assert!(parse_grammar_practice_query("").is_none());
    }

    #[test]
    fn parse_query_returns_none_for_invalid_ulid() {
        assert!(
            parse_grammar_practice_query("mode=grammar_practice&grammar_id=not-a-ulid").is_none()
        );
    }

    #[test]
    fn parse_query_returns_none_when_grammar_id_missing() {
        assert!(parse_grammar_practice_query("mode=grammar_practice").is_none());
    }
}
