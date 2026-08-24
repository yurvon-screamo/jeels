use super::acquaintance_state::{AcquaintanceContext, AcquaintanceSlideData, AcquaintanceStage};
use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant, FuriganaText};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{AcquaintanceSubphase, AnswerOutcome};
use origa::use_cases::CompleteAcquaintanceHandUseCase;
use ulid::Ulid;

/// Тренировка (S5): полные ротации руки до критерия каждой карты.
/// Витковый порядок — последовательный обход presentation_order;
/// закрывшие критерий продолжают отвечаться с замороженным прогрессом
/// (docs/acquaintance-mode.md, правило «Тренировка»).
///
/// Единственный источник истины для перерисовки — `AnswerOutcome`
/// из доменной машины; UI не дублирует подсчёт успехов.
#[component]
pub fn TrainingBody(ctx: AcquaintanceContext) -> impl IntoView {
    let i18n = use_i18n();
    let ctx_stored = StoredValue::new(ctx);
    let showing_answer = RwSignal::new(false);
    let rotation_index = RwSignal::new(0usize);

    let current_id = Memo::new(move |_| {
        let c = ctx_stored.get_value();
        c.state.with(|state| {
            let Some(hand) = state.hand.as_ref() else {
                return Ulid::nil();
            };
            let order = hand.presentation_order();
            if order.is_empty() {
                return Ulid::nil();
            }
            order[rotation_index.get() % order.len()]
        })
    });

    view! {
        <div class="min-h-[60vh] flex flex-col" data-testid="acquaintance-training">
            <div class="flex-1 py-6">
                {move || {
                    let Some(card_id) = current_id.get().non_nil() else {
                        return ().into_any();
                    };
                    let reverse = ctx_stored.get_value().state.with(|state| {
                        state.hand.as_ref().and_then(|h| h.subphase())
                    }) == Some(AcquaintanceSubphase::Reverse);
                    if showing_answer.get() {
                        view! { <TrainingAnswerSlide ctx=ctx_stored.get_value() card_id=card_id reverse /> }.into_any()
                    } else {
                        view! { <TrainingFrontSlide ctx=ctx_stored.get_value() card_id=card_id reverse /> }.into_any()
                    }
                }}
            </div>
            <Show when=move || !showing_answer.get() fallback=move || ()>
                <div class="flex justify-center">
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Filled)
                        on_click=Callback::new(move |_| showing_answer.set(true))
                        test_id=Signal::derive(|| "acquaintance-reveal-btn".to_string())
                    >
                        {t!(i18n, lesson.show_answer)}
                    </Button>
                </div>
            </Show>
            <Show when=move || showing_answer.get() fallback=move || ()>
                <div class="mt-4 grid grid-cols-2 gap-3">
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Default)
                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                            let outcome =
                                record_on_hand(&ctx_stored.get_value(), current_id.get_untracked(), false);
                            finish_answer(&ctx_stored.get_value(), &showing_answer, &rotation_index, outcome);
                        })
                        test_id=Signal::derive(|| "acquaintance-rating-dont-know".to_string())
                    >
                        {t!(i18n, acquaintance.dont_remember)}
                        <span class="kbd-hint">"[1]"</span>
                    </Button>
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Olive)
                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                            let outcome =
                                record_on_hand(&ctx_stored.get_value(), current_id.get_untracked(), true);
                            finish_answer(&ctx_stored.get_value(), &showing_answer, &rotation_index, outcome);
                        })
                        test_id=Signal::derive(|| "acquaintance-rating-remember".to_string())
                    >
                        {t!(i18n, acquaintance.remember)}
                        <span class="kbd-hint">"[2]"</span>
                    </Button>
                </div>
            </Show>
        </div>
    }
}

trait NonNilUlid {
    fn non_nil(self) -> Option<Ulid>;
}

impl NonNilUlid for Ulid {
    fn non_nil(self) -> Option<Ulid> {
        (!self.is_nil()).then_some(self)
    }
}

fn record_on_hand(ctx: &AcquaintanceContext, card_id: Ulid, remembered: bool) -> AnswerOutcome {
    let mut outcome = AnswerOutcome::ProgressFrozen;
    ctx.state.update(|state| {
        outcome = match state.hand.as_mut() {
            Some(hand) => hand.record_answer(card_id, remembered).unwrap_or_else(|e| {
                tracing::error!("Record answer failed for {card_id}: {e}");
                AnswerOutcome::ProgressFrozen
            }),
            None => AnswerOutcome::ProgressFrozen,
        };
    });
    outcome
}

fn finish_answer(
    ctx: &AcquaintanceContext,
    showing_answer: &RwSignal<bool>,
    rotation_index: &RwSignal<usize>,
    outcome: AnswerOutcome,
) {
    showing_answer.set(false);

    if matches!(outcome, AnswerOutcome::HandCompleted) {
        // Сидирование первого ревью назавтра + лимит одной операцией (S2).
        let ids = ctx.state.with(|state| {
            state
                .hand
                .as_ref()
                .map(|h| h.presentation_order())
                .unwrap_or_default()
        });
        let repo = ctx.repository.clone();
        spawn_local(async move {
            if let Err(e) = CompleteAcquaintanceHandUseCase::new(&repo)
                .execute(ids)
                .await
            {
                tracing::error!("Acquaintance hand completion failed: {e}");
            }
        });
        ctx.state
            .update(|state| state.stage = AcquaintanceStage::Summary);
        return;
    }

    // Любой некомплитный исход продвигает виток к следующей карте.
    rotation_index.set(rotation_index.get_untracked() + 1);
}

/// Фронт тренировки: японская сторона (Forward) или перевод (Reverse,
/// только слова; несловесные карты показывают единственный фронт).
#[component]
fn TrainingFrontSlide(ctx: AcquaintanceContext, card_id: Ulid, reverse: bool) -> impl IntoView {
    let known_kanji = ctx.known_kanji;
    view! {
        <div class="text-center py-10" data-testid="acquaintance-training-front">
            {move || {
                let Some(slide) = ctx
                    .slides
                    .get()
                    .iter()
                    .find(|s| s.card_id() == card_id)
                    .cloned()
                else {
                    return ().into_any();
                };
                if reverse {
                    let AcquaintanceSlideData::Vocabulary { translations, .. } = slide
                    else {
                        // Несловесные карты направления не меняют.
                        return ().into_any();
                    };
                    view! {
                        <p class="font-mono text-3xl text-[var(--fg-black)]">
                            {translations.join(", ")}
                        </p>
                    }
                        .into_any()
                } else {
                    match slide {
                        AcquaintanceSlideData::Vocabulary { word, .. } => view! {
                            <p class="font-serif text-5xl text-[var(--fg-black)] break-words">
                                <FuriganaText text=word known_kanji=known_kanji.get_untracked() />
                            </p>
                        }
                            .into_any(),
                        AcquaintanceSlideData::Kanji { kanji, name, .. } => view! {
                            <p class="font-serif text-6xl text-[var(--fg-black)]">{kanji}</p>
                            <p class="font-mono text-sm text-[var(--fg-muted)] pt-2">{name}</p>
                        }
                            .into_any(),
                        AcquaintanceSlideData::Grammar { title, short_description, .. } => view! {
                            <h2 class="font-serif text-3xl text-[var(--fg-black)]">{title}</h2>
                            <p class="font-mono text-sm pt-2">{short_description}</p>
                        }
                            .into_any(),

                    }
                }
            }}
        </div>
    }
}

/// Ответ тренировки: противоположная фронту сторона.
#[component]
fn TrainingAnswerSlide(ctx: AcquaintanceContext, card_id: Ulid, reverse: bool) -> impl IntoView {
    let known_kanji = ctx.known_kanji;
    view! {
        <div class="text-center space-y-3" data-testid="acquaintance-training-answer">
            {move || {
                let Some(slide) = ctx
                    .slides
                    .get()
                    .iter()
                    .find(|s| s.card_id() == card_id)
                    .cloned()
                else {
                    return ().into_any();
                };
                match slide {
                    AcquaintanceSlideData::Vocabulary { word, translations, .. } => {
                        if reverse {
                            view! {
                                <p class="font-serif text-5xl text-[var(--fg-black)] break-words">
                                    <FuriganaText text=word known_kanji=known_kanji.get_untracked() />
                                </p>
                            }
                                .into_any()
                        } else {
                            view! {
                                <p class="font-mono text-2xl text-[var(--fg-black)]">
                                    {translations.join(", ")}
                                </p>
                            }
                                .into_any()
                        }
                    },
                    AcquaintanceSlideData::Kanji { kanji, name, .. } => view! {
                        <p class="font-serif text-5xl text-[var(--fg-black)]">{kanji}</p>
                        <p class="font-mono text-sm text-[var(--fg-muted)]">{name}</p>
                    }
                        .into_any(),
                    AcquaintanceSlideData::Grammar { title, short_description, .. } => view! {
                        <h2 class="font-serif text-2xl text-[var(--fg-black)]">{title}</h2>
                        <p class="font-mono text-sm">{short_description}</p>
                    }
                        .into_any(),

                }
            }}
        </div>
    }
}
