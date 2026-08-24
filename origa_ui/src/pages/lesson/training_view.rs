use super::acquaintance_state::{AcquaintanceContext, AcquaintanceSlideData};
use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant, FuriganaText, is_speech_supported, speak_word};
use leptos::prelude::*;
use origa::domain::{AcquaintanceSubphase, AnswerOutcome};
use ulid::Ulid;

/// Fisher–Yates поверх xorshift64; источник энтропии — случайные биты
/// новых Ulid (крейт уже в дереве), внешних зависимостей не добавляет.
fn shuffled_order(mut cards: Vec<Ulid>) -> Vec<Ulid> {
    let mut seed = {
        let a = Ulid::new().0;
        let b = Ulid::new().0;
        (a ^ (b >> 1)) as u64 | 1
    };
    for i in (1..cards.len()).rev() {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let j = (seed as usize) % (i + 1);
        cards.swap(i, j);
    }
    cards
}

/// Витковый порядок тренировки: presentation_order минус выведенные карты,
/// перемешанный на каждый виток и каждую смену подфазы.
fn build_rotation_order(ctx: &AcquaintanceContext) -> Vec<Ulid> {
    ctx.state.with(|state| {
        let Some(hand) = state.hand.as_ref() else {
            return Vec::new();
        };
        let active: Vec<Ulid> = hand
            .presentation_order()
            .into_iter()
            .filter(|id| hand.entry(*id).is_some_and(|e| !e.is_retired()))
            .collect();
        shuffled_order(active)
    })
}

/// Тренировка (S5): полные ротации руки до критерия каждой карты.
/// Порядок витка перемешивается заново на каждый виток и смену подфазы;
/// закрывшие критерий продолжают отвечаться с замороженным прогрессом.
///
/// Единственный источник истины для перерисовки — `AnswerOutcome`
/// из доменной машины; UI не дублирует подсчёт успехов.
#[component]
pub fn TrainingBody(ctx: AcquaintanceContext) -> impl IntoView {
    let i18n = use_i18n();
    let ctx_stored = StoredValue::new(ctx);
    let showing_answer = RwSignal::new(false);
    let rotation_index = RwSignal::new(0usize);
    let training_order = RwSignal::new(build_rotation_order(&ctx_stored.get_value()));

    let current_id = Memo::new(move |_| {
        let order = training_order.get();
        if order.is_empty() {
            return Ulid::nil();
        }
        order[rotation_index.get() % order.len()]
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
                        on_click=Callback::new(move |_| {
                            showing_answer.set(true);
                            // Спека §8.2: рус→яп ответ = слово + повтор аудио.
                            let card_id = current_id.get_untracked();
                            if !card_id.is_nil() {
                                speak_reverse_answer(&ctx_stored.get_value(), card_id);
                            }
                        })
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
                            finish_answer(
                                &ctx_stored.get_value(),
                                &showing_answer,
                                &rotation_index,
                                &training_order,
                                outcome,
                            );
                        })
                        test_id=Signal::derive(|| "acquaintance-rating-dont-know".to_string())
                    >
                        {t!(i18n, acquaintance.dont_remember)}
                    </Button>
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Olive)
                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                            let outcome =
                                record_on_hand(&ctx_stored.get_value(), current_id.get_untracked(), true);
                            finish_answer(
                                &ctx_stored.get_value(),
                                &showing_answer,
                                &rotation_index,
                                &training_order,
                                outcome,
                            );
                        })
                        test_id=Signal::derive(|| "acquaintance-rating-remember".to_string())
                    >
                        {t!(i18n, acquaintance.remember)}
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

/// Повтор аудио в ответе Reverse-подфазы (спека §8.2); остальные случаи —
/// фронт Forward уже прозвучал при показе слова в презентации.
fn speak_reverse_answer(ctx: &AcquaintanceContext, card_id: Ulid) {
    let reverse = ctx
        .state
        .with(|state| state.hand.as_ref().and_then(|h| h.subphase()))
        == Some(AcquaintanceSubphase::Reverse);
    if !reverse {
        return;
    }
    if let Some(AcquaintanceSlideData::Vocabulary { word, .. }) =
        ctx.slides.get().iter().find(|s| s.card_id() == card_id)
    {
        speak_if_supported(word);
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
    training_order: &RwSignal<Vec<Ulid>>,
    outcome: AnswerOutcome,
) {
    showing_answer.set(false);

    if matches!(outcome, AnswerOutcome::HandCompleted) {
        ctx.complete_hand_and_show_summary();
        return;
    }

    // Смена подфазы — порядок витка перемешивается заново (спека §Тренировка).
    if matches!(outcome, AnswerOutcome::SubphaseAdvanced) {
        training_order.set(shuffled_order(training_order.get_untracked()));
        rotation_index.set(0);
        return;
    }

    rotation_index.set(rotation_index.get_untracked() + 1);
    // Новый виток — новый шафл.
    let len = training_order.get_untracked().len();
    if len > 0 && rotation_index.get_untracked() % len == 0 {
        training_order.set(shuffled_order(training_order.get_untracked()));
    }
}

/// Повтор аудио слова в ответе Reverse-подфазы (спека §8.2); guard
/// is_speech_supported гасит среды без TTS.
fn speak_if_supported(word: &str) {
    if is_speech_supported() {
        speak_word(word, 1.0);
    }
}

/// Фронт тренировки: японская сторона (Forward) или перевод (Reverse,
/// только слова); несловесные карты показывают единственный фронт.
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
                match slide {
                    AcquaintanceSlideData::Vocabulary { word, translations, .. } => {
                        if reverse {
                            view! {
                                <p class="font-mono text-3xl text-[var(--fg-black)]">
                                    {translations.join(", ")}
                                </p>
                            }
                                .into_any()
                        } else {
                            view! {
                                <p class="font-serif text-5xl text-[var(--fg-black)] break-words">
                                    <FuriganaText text=word known_kanji=known_kanji.get_untracked() />
                                </p>
                            }
                                .into_any()
                        }
                    },
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
            }}
        </div>
    }
}

/// Ответ тренировки: противоположная фронту сторона. Для слов Reverse —
/// слово с фуриганой и повтор аудио (спека §8.2).
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
