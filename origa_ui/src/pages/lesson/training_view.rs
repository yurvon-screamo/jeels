use super::acquaintance_keyboard::{
    AcquaintanceKeyboardActions, create_acquaintance_keyboard_handler,
};
use super::acquaintance_state::{AcquaintanceContext, AcquaintanceSlideData};
use super::keyboard_handler::is_typing_target;
use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant, FuriganaText, is_speech_supported, speak_word};
use leptos::prelude::*;
use leptos_use::use_event_listener;
use origa::domain::{AcquaintanceSubphase, AnswerOutcome};
use ulid::Ulid;

/// Раскрытие ответа: и кнопка, и Space ведут себя одинаково.
fn do_reveal(
    ctx_stored: &StoredValue<AcquaintanceContext>,
    current_id: &Memo<Ulid>,
    showing_answer: &RwSignal<bool>,
) {
    showing_answer.set(true);
    let card_id = current_id.get_untracked();
    if !card_id.is_nil() {
        speak_reverse_answer(&ctx_stored.get_value(), card_id);
    }
}

/// Запись ответа: и кнопки [1]/[2], и клавиши 1/2 ведут себя одинаково.
fn do_rate(
    ctx_stored: &StoredValue<AcquaintanceContext>,
    current_id: &Memo<Ulid>,
    showing_answer: &RwSignal<bool>,
    rotation_index: &RwSignal<usize>,
    training_order: &RwSignal<Vec<Ulid>>,
    remembered: bool,
) {
    let outcome = record_on_hand(
        &ctx_stored.get_value(),
        current_id.get_untracked(),
        remembered,
    );
    finish_answer(
        &ctx_stored.get_value(),
        showing_answer,
        rotation_index,
        training_order,
        outcome,
    );
    // Шапке нужен тип новой текущей карты (тег типа).
    let next_card = current_id.get_untracked();
    let state = ctx_stored.get_value().state;
    state.update(|s| s.current_card_id = (!next_card.is_nil()).then_some(next_card));
}

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

    // Текущая карта тренировки: шапка читает её для тега типа карты.
    // Обновляется в явных точках смены карты (старт тренировки и переход
    // к следующей после оценки) — без Effect, чтобы не дёргать реактивность
    // во время монтажа.
    let state_for_current = ctx_stored.get_value().state;
    let first_card = current_id.get_untracked();
    if !first_card.is_nil() {
        state_for_current.update(|state| state.current_card_id = Some(first_card));
    }

    // Клавиатура: те же хендлы, что у кнопок (спека §8.3).
    let handle_keydown = {
        let c = ctx_stored;
        create_acquaintance_keyboard_handler(
            c.get_value(),
            showing_answer,
            AcquaintanceKeyboardActions {
                // Advance разрешён только в показе; в тренировке — Reveal/Rate.
                on_advance: Box::new(|| {}),
                on_reveal: Box::new(move || {
                    do_reveal(&c, &current_id, &showing_answer);
                }),
                on_rate: Box::new(move |remembered: bool| {
                    do_rate(
                        &c,
                        &current_id,
                        &showing_answer,
                        &rotation_index,
                        &training_order,
                        remembered,
                    );
                }),
            },
        )
    };

    let _ = use_event_listener(document(), leptos::ev::keydown, move |ev| {
        if is_typing_target(ev.target().as_ref()) {
            return;
        }
        handle_keydown(ev);
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
                            do_reveal(&ctx_stored, &current_id, &showing_answer);
                        })
                        test_id=Signal::derive(|| "acquaintance-reveal-btn".to_string())
                    >
                        {t!(i18n, lesson.show_answer)}
                        <span class="kbd-hint">{t!(i18n, lesson.space_key)}</span>
                    </Button>
                </div>
            </Show>
            <Show when=move || showing_answer.get() fallback=move || ()>
                <div class="mt-4 grid grid-cols-2 gap-3">
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Default)
                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                            do_rate(
                                &ctx_stored,
                                &current_id,
                                &showing_answer,
                                &rotation_index,
                                &training_order,
                                false,
                            );
                        })
                        test_id=Signal::derive(|| "acquaintance-rating-dont-know".to_string())
                    >
                        {t!(i18n, acquaintance.dont_remember)}
                        <span class="kbd-hint">"[1]"</span>
                    </Button>
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Olive)
                        on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                            do_rate(
                                &ctx_stored,
                                &current_id,
                                &showing_answer,
                                &rotation_index,
                                &training_order,
                                true,
                            );
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

/// Граница витка тренировки: что делать после очередного ответа.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationBoundary {
    /// Круг не закончен — порядок стабилен, направление не меняется.
    MidRotation,
    /// Последняя карта круга отвечена: направление могло смениться
    /// (только здесь), порядок следующего круга перемешивается.
    RotationEnded { subphase_advanced: bool },
}

/// Чистая логика границы круга: смена направления слов и перемешивание
/// происходят ровно на границе — ошибка юзера порядок не трогает
/// (docs/acquaintance-mode.md, правило «Тренировка»).
pub fn rotation_boundary(
    rotation_index: usize,
    active_len: usize,
    hand: &mut origa::domain::AcquaintanceHand,
) -> RotationBoundary {
    if active_len == 0 || rotation_index % active_len != 0 {
        return RotationBoundary::MidRotation;
    }
    let subphase_advanced = hand.advance_subphase_if_words_done();
    RotationBoundary::RotationEnded { subphase_advanced }
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

    rotation_index.set(rotation_index.get_untracked() + 1);
    let mut boundary = RotationBoundary::MidRotation;
    ctx.state.update(|state| {
        if let Some(hand) = state.hand.as_mut() {
            boundary = rotation_boundary(
                rotation_index.get_untracked(),
                training_order.get_untracked().len(),
                hand,
            );
        }
    });
    if matches!(boundary, RotationBoundary::RotationEnded { .. }) {
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

#[cfg(test)]
mod rotation_tests {
    use super::*;
    use origa::domain::CardType;

    /// Полный цикл двух слов: три круга Forward → смена направления на
    /// границе → три круга Reverse → HandCompleted. Смена направления
    /// происходит ТОЛЬКО на границе круга — в середине порядок стабилен.
    #[test]
    fn full_cycle_advances_subphase_only_at_rotation_boundaries() {
        // Arrange: рука из двух слов, порядок круга [a, b]
        let [a, b] = (0..2)
            .map(|_| Ulid::new())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let mut hand = origa::domain::AcquaintanceHand::new(vec![
            (a, CardType::Vocabulary),
            (b, CardType::Vocabulary),
        ])
        .unwrap();

        // Act / Assert: Forward — три полных круга по 2 ответа
        for rotation in 0..3 {
            assert_eq!(
                hand.record_answer(a, true).unwrap(),
                AnswerOutcome::Counted {
                    progress: rotation + 1
                }
            );
            // середина круга: границы нет, направление стабильно
            assert_eq!(
                rotation_boundary(1, 2, &mut hand),
                RotationBoundary::MidRotation
            );
            assert_eq!(
                hand.record_answer(b, true).unwrap(),
                AnswerOutcome::Counted {
                    progress: rotation + 1
                }
            );
            let boundary = rotation_boundary(2, 2, &mut hand);
            if rotation < 2 {
                assert_eq!(
                    boundary,
                    RotationBoundary::RotationEnded {
                        subphase_advanced: false
                    },
                    "Forward ещё не закрыт всеми словами"
                );
            } else {
                assert_eq!(
                    boundary,
                    RotationBoundary::RotationEnded {
                        subphase_advanced: true
                    },
                    "последний круг Forward закрывает направление"
                );
            }
        }

        // Reverse: два полных круга + последний ответ закрывает руку
        assert_eq!(
            hand.subphase(),
            Some(origa::domain::AcquaintanceSubphase::Reverse)
        );
        for rotation in 0..3 {
            let a_outcome = hand.record_answer(a, true).unwrap();
            assert_eq!(
                rotation_boundary(1, 2, &mut hand),
                RotationBoundary::MidRotation
            );
            let b_outcome = hand.record_answer(b, true).unwrap();
            let boundary = rotation_boundary(2, 2, &mut hand);
            assert_eq!(
                boundary,
                RotationBoundary::RotationEnded {
                    subphase_advanced: false
                },
                "в Reverse смены направления не существует"
            );
            if rotation < 2 {
                assert_eq!(
                    a_outcome,
                    AnswerOutcome::Counted {
                        progress: rotation + 1
                    }
                );
                assert_eq!(
                    b_outcome,
                    AnswerOutcome::Counted {
                        progress: rotation + 1
                    }
                );
            } else {
                assert_eq!(
                    b_outcome,
                    AnswerOutcome::HandCompleted,
                    "третий reverse-успех последнего слова закрывает руку"
                );
            }
        }
    }

    /// Ошибка не влияет на порядок: провал в середине круга не двигает
    /// границу и не сбрасывает прогресс.
    #[test]
    fn failed_answer_mid_rotation_keeps_boundary_and_progress() {
        // Arrange
        let [a, b] = (0..2)
            .map(|_| Ulid::new())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let mut hand = origa::domain::AcquaintanceHand::new(vec![
            (a, CardType::Vocabulary),
            (b, CardType::Vocabulary),
        ])
        .unwrap();

        // Act: успех, затем провал — оба в середине круга
        hand.record_answer(a, true).unwrap();
        let outcome = hand.record_answer(b, false).unwrap();

        // Assert: Failed без прогресса, граница не достигнута
        assert_eq!(outcome, AnswerOutcome::Failed);
        assert_eq!(
            rotation_boundary(1, 2, &mut hand),
            RotationBoundary::MidRotation
        );
        assert_eq!(
            hand.subphase(),
            Some(origa::domain::AcquaintanceSubphase::Forward)
        );
    }
}
