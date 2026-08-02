use crate::i18n::*;
use crate::pages::lesson::card_type::CardType;
use crate::repository::HybridUserRepository;
use crate::ui_components::{Spinner, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_use::use_event_listener;
use origa::traits::UserRepository;
use origa::use_cases::MarkCardAsKnownUseCase;
use std::collections::HashSet;
use ulid::Ulid;

use super::scoring_card_view::ScoringCardView;
use super::scoring_helpers::{ScoringCard, build_scoring_cards};
use super::scoring_mark_all::{MarkAllState, spawn_mark_all_effect};
use super::scoring_progress::{ScoringProgressBar, compute_section_bounds};

#[component]
pub fn ScoringStep(
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional)] mark_all_trigger: RwSignal<u32>,
    scoring_completed: RwSignal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let cards: RwSignal<Vec<ScoringCard>> = RwSignal::new(Vec::new());
    let current_index: RwSignal<usize> = RwSignal::new(0);
    let is_loading = RwSignal::new(true);
    let is_rating = RwSignal::new(false);
    let known_count: RwSignal<usize> = RwSignal::new(0);
    let disposed = StoredValue::new(());

    let repo_for_load = repository.clone();
    let repo_for_know = repository.clone();
    let repo_for_skip = repository.clone();
    let repo_for_mark_all = repository.clone();

    let i18n_for_load = i18n;
    Effect::new(move |_| {
        let repo = repo_for_load.clone();
        spawn_local(async move {
            let Ok(Some(user)) = repo.get_current_user().await else {
                is_loading.set(false);
                return;
            };
            if disposed.is_disposed() {
                return;
            }

            let lang = crate::i18n::locale_to_native_language(&i18n_for_load.get_locale());
            let mut scoring_cards =
                build_scoring_cards(user.knowledge_set().study_cards(), &lang, i18n_for_load);

            // Drop cards the user already answered "don't know" to in a prior
            // session — these are persisted per-click so the scoring step
            // resumes mid-queue after an app restart instead of starting over.
            let skipped: HashSet<Ulid> =
                user.onboarding_scoring_skipped().iter().copied().collect();
            scoring_cards.retain(|c| !skipped.contains(&c.card_id));

            if disposed.is_disposed() {
                return;
            }

            let total = scoring_cards.len();
            cards.set(scoring_cards);
            if total == 0 {
                scoring_completed.set(true);
            }
            is_loading.set(false);
        });
    });

    let total = Memo::new(move |_| cards.get().len());

    let on_dont_know = Callback::new(move |_: ()| {
        let idx = current_index.get_untracked();
        let t = total.get_untracked();
        if let Some(scoring_card) = cards.get_untracked().get(idx) {
            let card_id = scoring_card.card_id;
            let repo = repo_for_skip.clone();
            is_rating.set(true);
            spawn_local(async move {
                // Persist failure is non-fatal and matches the trade-off of
                // `on_know` (`MarkCardAsKnownUseCase` logs+continues on save
                // error). Worst case: a skipped card reappears after the next
                // app restart. Blocking the UI on a save error would deadlock
                // scoring when offline, which is a worse UX than re-evaluating
                // one card later.
                let outcome = async {
                    let mut user = repo
                        .get_current_user()
                        .await?
                        .ok_or(origa::domain::OrigaError::CurrentUserNotExist)?;
                    user.mark_card_skipped_in_onboarding(card_id);
                    repo.save(&user).await?;
                    Ok::<(), origa::domain::OrigaError>(())
                }
                .await;
                if let Err(e) = outcome {
                    tracing::warn!(error = ?e, card_id = %card_id, "Failed to persist skipped card");
                }
                if disposed.is_disposed() {
                    return;
                }
                is_rating.set(false);
                if idx + 1 >= t {
                    scoring_completed.set(true);
                } else {
                    current_index.set(idx + 1);
                }
            });
        } else if idx + 1 >= t {
            scoring_completed.set(true);
        } else {
            current_index.set(idx + 1);
        }
    });

    let on_know = Callback::new(move |_: ()| {
        let idx = current_index.get_untracked();
        let t = total.get_untracked();

        if let Some(scoring_card) = cards.get_untracked().get(idx) {
            let card_id = scoring_card.card_id;
            let repo = repo_for_know.clone();
            is_rating.set(true);

            spawn_local(async move {
                let use_case = MarkCardAsKnownUseCase::new(&repo);
                if use_case.execute(card_id).await.is_ok() {
                    known_count.update(|c| *c += 1);
                }

                if disposed.is_disposed() {
                    return;
                }

                is_rating.set(false);

                if idx + 1 >= t {
                    scoring_completed.set(true);
                } else {
                    current_index.set(idx + 1);
                }
            });
        }
    });

    let kb_on_dont_know = on_dont_know;
    let kb_on_know = on_know;
    let _ = use_event_listener(document(), leptos::ev::keydown, move |ev| {
        if is_loading.get() || is_rating.get() || scoring_completed.get() {
            return;
        }
        match ev.key().as_str() {
            "1" => kb_on_dont_know.run(()),
            "2" => kb_on_know.run(()),
            _ => {},
        }
    });

    let current_card: Signal<Option<ScoringCard>> =
        Signal::derive(move || cards.get().get(current_index.get()).cloned());

    let section_bounds = Memo::new(move |_| {
        compute_section_bounds(
            &cards
                .get()
                .iter()
                .map(|c| c.card_type)
                .collect::<Vec<CardType>>(),
        )
    });

    spawn_mark_all_effect(MarkAllState {
        repository: repo_for_mark_all,
        disposed,
        mark_all_trigger,
        cards,
        current_index,
        is_loading,
        is_rating,
        scoring_completed,
        known_count,
    });

    view! {
        <div class="scoring-step" data-testid=test_id_val>
            <Show when=move || is_loading.get()>
                <div class="flex flex-col items-center py-8 gap-4">
                    <Spinner test_id="scoring-step-loading-spinner" />
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        {t!(i18n, onboarding.scoring.loading)}
                    </Text>
                </div>
            </Show>

            <Show when=move || !is_loading.get() && !scoring_completed.get()>
                <div>
                    <div class="text-center mb-4">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "scoring-step-hint".to_string())>
                            {t!(i18n, onboarding.scoring.hint)}
                        </Text>
                    </div>

                    <div class="mb-6">
                        <ScoringProgressBar
                            current_index=Signal::derive(move || current_index.get())
                            total=Signal::derive(move || total.get())
                            sections=Signal::derive(move || section_bounds.get())
                            test_id=Signal::derive(|| "scoring-step-progress".to_string())
                        />
                    </div>

                    <ScoringCardView
                        card=current_card
                        is_rating=Signal::derive(move || is_rating.get())
                        on_know=on_know
                        on_dont_know=on_dont_know
                        test_id=Signal::derive(|| "scoring-step-card".to_string())
                    />
                </div>
            </Show>

            <Show when=move || scoring_completed.get()>
                <div class="text-center py-8">
                    <Text size=TextSize::Large variant=TypographyVariant::Primary test_id=Signal::derive(|| "scoring-step-complete".to_string())>
                        {t!(i18n, onboarding.scoring.great)}
                    </Text>
                    <div class="mt-2">
                        <Text size=TextSize::Default variant=TypographyVariant::Muted test_id=Signal::derive(|| "scoring-step-complete-info".to_string())>
                            {move || {
                                let known = known_count.get();
                                let t = total.get();
                                let locale = i18n.get_locale();
                                if t == 0 {
                                    td_string!(locale, onboarding.scoring.no_new_cards).to_string()
                                } else if known == 0 {
                                    td_string!(locale, onboarding.scoring.all_new).to_string()
                                } else {
                                    td_string!(locale, onboarding.scoring.you_know_count)
                                        .replace("{known}", &known.to_string())
                                        .replace("{total}", &t.to_string())
                                }
                            }}
                        </Text>
                    </div>
                    <div class="mt-2">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {t!(i18n, onboarding.scoring.press_finish)}
                        </Text>
                    </div>
                </div>
            </Show>
        </div>
    }
}
