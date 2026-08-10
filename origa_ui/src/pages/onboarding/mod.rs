mod onboarding_actions;
mod onboarding_state;

mod apps_step;
mod intro_step;
mod jlpt_step;
mod load_step;
mod progress;
mod scoring_card_view;
mod scoring_helpers;
mod scoring_mark_all;
mod scoring_progress;
mod scoring_step;
mod summary_step;

use crate::i18n::{
    Locale, locale_to_native_language, native_language_to_locale, t, td_string, use_i18n,
};
use crate::loaders::WellKnownSetLoaderImpl;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Button, ButtonVariant, CardLayout, CardLayoutSize, Modal, PageLayout, PageLayoutVariant,
    Spinner, Stepper, StepperStep, Text, TextSize, TypographyVariant,
};
use apps_step::AppsStep;
use intro_step::IntroStep;
use jlpt_step::JlptStep;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use load_step::LoadStep;
use onboarding_actions::{
    create_on_finish_callback, create_on_skip_callback, create_on_start_import_callback,
};
use onboarding_state::{OnboardingState, OnboardingStep};
use origa::domain::NativeLanguage;
use origa::domain::User;
use origa::traits::UserRepository;
use origa::use_cases::UpdateUserProfileUseCase;
use progress::ProgressStep;
use scoring_step::ScoringStep;
use summary_step::SummaryStep;

/// Which destructive scoring action is awaiting confirmation in the modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingScoringAction {
    KnowAll,
    Skip,
}

#[component]
pub fn Onboarding() -> impl IntoView {
    let i18n = use_i18n();
    let current_locale: RwSignal<Locale> = RwSignal::new(i18n.get_locale_untracked());
    let selected_language: RwSignal<NativeLanguage> =
        RwSignal::new(locale_to_native_language(&i18n.get_locale_untracked()));

    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");
    let navigate = use_navigate();
    let navigate_for_init = navigate.clone();
    let navigate_for_skip = navigate.clone();
    let navigate_for_finish = navigate.clone();

    let state = RwSignal::new(OnboardingState::new());
    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let sets_loaded = RwSignal::new(false);
    let is_loading = RwSignal::new(true);
    let is_importing = RwSignal::new(false);
    let disposed = StoredValue::new(());
    let mark_all_trigger: RwSignal<u32> = RwSignal::new(0);
    let scoring_completed: RwSignal<bool> = RwSignal::new(false);
    let lang_initialized = RwSignal::new(false);

    // Confirm-modal open state. When `true`, the modal shows the confirmation
    // for whichever action was last requested.
    let confirm_modal_open: RwSignal<bool> = RwSignal::new(false);
    let pending_scoring_action: RwSignal<Option<PendingScoringAction>> = RwSignal::new(None);

    provide_context(state);

    let i18n_for_lang = i18n;
    let locale_for_sync = current_locale;
    Effect::new(move |_| {
        let lang = selected_language.get();
        let locale = native_language_to_locale(&lang);
        i18n_for_lang.set_locale(locale);
        locale_for_sync.set(locale);
    });

    let lang_for_init = selected_language;
    let current_user_for_lang = current_user;
    let lang_init_flag = lang_initialized;
    Effect::new(move |_| {
        if let Some(user) = current_user_for_lang.get() {
            lang_for_init.set(*user.native_language());
            lang_init_flag.set(true);
        }
    });

    let disposed_for_lang = disposed;
    let repo_for_lang = repository.clone();
    let lang_signal_for_save = selected_language;
    let current_user_for_save = current_user;
    let lang_init_flag_for_save = lang_initialized;
    Effect::new(move |_| {
        let lang = lang_signal_for_save.get();
        if !lang_init_flag_for_save.get() {
            return;
        }
        let disposed = disposed_for_lang;
        let repo = repo_for_lang.clone();
        let current_user = current_user_for_save;
        spawn_local(async move {
            if disposed.is_disposed() {
                return;
            }
            if let Some(user) = current_user.get_untracked() {
                let use_case = UpdateUserProfileUseCase::new(&repo);
                let daily_load = *user.daily_load();
                let telegram_id = user.telegram_user_id().copied();
                if let Err(e) = use_case.execute(lang, daily_load, telegram_id).await {
                    tracing::error!("Failed to save language on onboarding: {:?}", e);
                }
            }
        });
    });

    let steps: Signal<Vec<StepperStep>> = Signal::derive(move || {
        let locale = current_locale.get();
        vec![
            StepperStep {
                number: 1,
                label: td_string!(locale, onboarding.steps.greeting).to_string(),
            },
            StepperStep {
                number: 2,
                label: td_string!(locale, onboarding.steps.pace).to_string(),
            },
            StepperStep {
                number: 3,
                label: td_string!(locale, onboarding.steps.level).to_string(),
            },
            StepperStep {
                number: 4,
                label: td_string!(locale, onboarding.steps.apps).to_string(),
            },
            StepperStep {
                number: 5,
                label: td_string!(locale, onboarding.steps.progress).to_string(),
            },
            StepperStep {
                number: 6,
                label: td_string!(locale, onboarding.steps.import).to_string(),
            },
            StepperStep {
                number: 7,
                label: td_string!(locale, onboarding.steps.scoring).to_string(),
            },
        ]
    });

    let active_step: RwSignal<usize> = RwSignal::new(0);

    Effect::new(move |_| {
        let step = state.get().current_step.as_usize();
        active_step.set(step);
    });

    let repo_for_init = repository.clone();
    let loader = WellKnownSetLoaderImpl::new();
    let initialized = RwSignal::new(false);

    Effect::new(move |_| {
        if initialized.get() {
            return;
        }
        initialized.set(true);

        let repo = repo_for_init.clone();
        let loader = loader.clone();
        let nav = navigate_for_init.clone();
        spawn_local(async move {
            match repo.get_current_user().await {
                Ok(Some(user)) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    current_user.set(Some(user.clone()));
                    if user.is_onboarding_completed() {
                        nav("/home", Default::default());
                        return;
                    }
                    // Resume mid-flight onboarding (after app restart): if
                    // sets have already been imported, jump straight to the
                    // scoring step instead of forcing the user to walk through
                    // the intro/jlpt/apps flow again.
                    if !user.imported_sets().is_empty() {
                        state.update(|s| {
                            s.current_step = OnboardingStep::Scoring;
                        });
                    }
                },
                Ok(None) => {
                    tracing::warn!("Onboarding: user not found");
                    nav("/login", Default::default());
                    return;
                },
                Err(e) => {
                    tracing::error!("Onboarding: get_current_user error: {:?}", e);
                    nav("/login", Default::default());
                    return;
                },
            };

            match loader.load_meta_list().await {
                Ok(meta_list) => {
                    if disposed.is_disposed() {
                        return;
                    }
                    state.update(|s| {
                        s.set_available_sets(meta_list);
                        sets_loaded.set(true);
                    });
                },
                Err(e) => {
                    tracing::error!("Onboarding: load_meta_list error: {:?}", e);
                },
            }
            is_loading.set(false);
        });
    });

    let on_next = Callback::new(move |_: ()| {
        state.update(|s| {
            s.go_to_next_step();
        });
    });

    let on_prev = Callback::new(move |_: ()| {
        state.update(|s| {
            s.go_to_prev_step();
        });
    });

    let on_skip = create_on_skip_callback(repository.clone(), state, disposed, navigate_for_skip);

    let on_finish = create_on_finish_callback(repository.clone(), disposed, navigate_for_finish);

    let on_start_import =
        create_on_start_import_callback(repository, state, current_user, is_importing, disposed);

    let can_proceed = Memo::new(move |_| state.get().can_proceed());

    view! {
        <PageLayout variant=PageLayoutVariant::Full test_id="onboarding-page">
            <CardLayout size=CardLayoutSize::Adaptive class="py-6" test_id="onboarding-card">
                <Show when=move || is_loading.get()>
                    <div class="flex flex-col items-center py-8 gap-4">
                        <Spinner test_id="onboarding-spinner" />
                        <Text size=TextSize::Small variant=TypographyVariant::Muted test_id=Signal::derive(|| "onboarding-loading-text".to_string())>
                            {t!(i18n, onboarding.loading)}
                        </Text>
                    </div>
                </Show>

                <Show when=move || !is_loading.get()>
                    <div class="onboarding-container">
                        <Stepper steps=steps active=active_step test_id="onboarding-stepper" />

                        <div class="onboarding-content mt-8">
                            <Show when=move || matches!(state.get().current_step, OnboardingStep::Intro)>
                                <IntroStep selected_language=selected_language test_id=Signal::derive(|| "onboarding-intro-step".to_string()) />
                            </Show>

                            <Show when=move || matches!(state.get().current_step, OnboardingStep::Load)>
                                <LoadStep test_id=Signal::derive(|| "onboarding-load-step".to_string()) />
                            </Show>

                            <Show when=move || matches!(state.get().current_step, OnboardingStep::Jlpt)>
                                <JlptStep test_id=Signal::derive(|| "onboarding-jlpt-step".to_string()) />
                            </Show>

                            <Show when=move || matches!(state.get().current_step, OnboardingStep::Apps)>
                                <AppsStep test_id=Signal::derive(|| "onboarding-apps-step".to_string()) />
                            </Show>

                            <Show when=move || matches!(state.get().current_step, OnboardingStep::Progress)>
                                <ProgressStep test_id=Signal::derive(|| "onboarding-progress-step".to_string()) />
                            </Show>

                            <Show when=move || matches!(state.get().current_step, OnboardingStep::Summary)>
                                <SummaryStep test_id=Signal::derive(|| "onboarding-summary-step".to_string()) />
                            </Show>

                            <Show when=move || matches!(state.get().current_step, OnboardingStep::Scoring)>
                                <ScoringStep test_id=Signal::derive(|| "onboarding-scoring-step".to_string()) mark_all_trigger=mark_all_trigger scoring_completed=scoring_completed />
                            </Show>
                        </div>

                        <div class="onboarding-actions mt-8 flex justify-between">
                            <div>
                                <Show when=move || state.get().is_first_step()>
                                    <Button
                                        variant=ButtonVariant::Ghost
                                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                            on_skip.run(());
                                        })
                                        test_id="onboarding-skip"
                                    >
                                        {t!(i18n, onboarding.skip)}
                                    </Button>
                                </Show>

                                <Show when=move || !state.get().is_first_step()
                                    && !matches!(state.get().current_step, OnboardingStep::Scoring)
                                >
                                    <Button
                                        variant=ButtonVariant::Ghost
                                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                            on_prev.run(());
                                        })
                                        test_id="onboarding-prev"
                                    >
                                        {t!(i18n, onboarding.back)}
                                    </Button>
                                </Show>
                            </div>

                            <div>
                                <Show when=move || !matches!(state.get().current_step, OnboardingStep::Summary)
                                    && !matches!(state.get().current_step, OnboardingStep::Scoring)
                                >
                                    <Button
                                        variant=ButtonVariant::Olive
                                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                            on_next.run(());
                                        })
                                        disabled=Signal::derive(move || !can_proceed.get())
                                        test_id="onboarding-next"
                                    >
                                        {t!(i18n, onboarding.next)}
                                    </Button>
                                </Show>

                                <Show when=move || matches!(state.get().current_step, OnboardingStep::Summary)>
                                    <Button
                                        variant=ButtonVariant::Olive
                                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                            on_start_import.run(());
                                        })
                                        disabled=Signal::derive(move || is_importing.get() || !can_proceed.get())
                                        test_id="onboarding-import"
                                        attr:data-loading=Signal::derive(move || is_importing.get().to_string())
                                    >
                                        {move || if is_importing.get() { t!(i18n, onboarding.importing).into_any() } else { t!(i18n, onboarding.start_import).into_any() }}
                                    </Button>
                                </Show>
                            </div>
                        </div>

                        // Scoring step actions are on a separate row below
                        // the card so they don't sit adjacent to the Know /
                        // Don't-Know buttons — reducing accidental taps on
                        // Skip / Know-All.
                        <Show when=move || matches!(state.get().current_step, OnboardingStep::Scoring)>
                            <div class="onboarding-scoring-actions mt-4 flex justify-between items-center">
                                <Show when=move || !scoring_completed.get()>
                                    <Button
                                        variant=ButtonVariant::Ghost
                                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                            pending_scoring_action.set(Some(PendingScoringAction::Skip));
                                            confirm_modal_open.set(true);
                                        })
                                        test_id="onboarding-skip-scoring"
                                    >
                                        {t!(i18n, onboarding.skip)}
                                    </Button>
                                </Show>

                                <div>
                                    <Show when=move || !scoring_completed.get()>
                                        <Button
                                            variant=ButtonVariant::Olive
                                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                                pending_scoring_action.set(Some(PendingScoringAction::KnowAll));
                                                confirm_modal_open.set(true);
                                            })
                                            test_id="onboarding-mark-all-known"
                                        >
                                            {t!(i18n, onboarding.know_all)}
                                        </Button>
                                    </Show>

                                    <Show when=move || scoring_completed.get()>
                                        <Button
                                            variant=ButtonVariant::Olive
                                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                                on_finish.run(());
                                            })
                                            test_id="onboarding-finish"
                                        >
                                            {t!(i18n, onboarding.finish)}
                                        </Button>
                                    </Show>
                                </div>
                            </div>

                            // Confirmation modal for destructive scoring actions.
                            // Both "Know all" and "Skip" are irreversible from the
                            // user's perspective — the modal makes the consequence
                            // explicit before committing.
                            <Modal
                                is_open=confirm_modal_open
                                title=Signal::derive(move || {
                                    let locale = i18n.get_locale();
                                    match pending_scoring_action.get() {
                                        Some(PendingScoringAction::KnowAll) => {
                                            td_string!(locale, onboarding.scoring.know_all_confirm_title).to_string()
                                        },
                                        Some(PendingScoringAction::Skip) => {
                                            td_string!(locale, onboarding.scoring.skip_confirm_title).to_string()
                                        },
                                        None => String::new(),
                                    }
                                })
                                test_id=Signal::derive(|| "onboarding-scoring-confirm".to_string())
                            >
                                <div class="flex flex-col gap-4">
                                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                        {move || {
                                            let locale = i18n.get_locale();
                                            match pending_scoring_action.get() {
                                                Some(PendingScoringAction::KnowAll) => {
                                                    td_string!(locale, onboarding.scoring.know_all_confirm_body).to_string()
                                                },
                                                Some(PendingScoringAction::Skip) => {
                                                    td_string!(locale, onboarding.scoring.skip_confirm_body).to_string()
                                                },
                                                None => String::new(),
                                            }
                                        }}
                                    </Text>
                                    <div class="flex justify-end gap-3">
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                                pending_scoring_action.set(None);
                                                confirm_modal_open.set(false);
                                            })
                                            test_id="onboarding-scoring-confirm-cancel"
                                        >
                                            {move || {
                                                let locale = i18n.get_locale();
                                                td_string!(locale, onboarding.scoring.cancel).to_string().into_any()
                                            }}
                                        </Button>
                                        <Button
                                            variant=ButtonVariant::Olive
                                            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                                                let action = pending_scoring_action.get();
                                                pending_scoring_action.set(None);
                                                confirm_modal_open.set(false);
                                                match action {
                                                    Some(PendingScoringAction::KnowAll) => {
                                                        mark_all_trigger.update(|n| *n += 1);
                                                    },
                                                    Some(PendingScoringAction::Skip) => {
                                                        on_skip.run(());
                                                    },
                                                    None => {},
                                                }
                                            })
                                            test_id="onboarding-scoring-confirm-ok"
                                        >
                                            {move || {
                                                let locale = i18n.get_locale();
                                                td_string!(locale, onboarding.scoring.confirm).to_string().into_any()
                                            }}
                                        </Button>
                                    </div>
                                </div>
                            </Modal>
                        </Show>
                    </div>
                </Show>
            </CardLayout>
        </PageLayout>
    }
}
