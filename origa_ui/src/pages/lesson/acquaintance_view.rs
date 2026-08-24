use super::acquaintance_state::{AcquaintanceContext, AcquaintanceSlideData, AcquaintanceStage};
use super::hand_progress_strip::HandProgressStrip;
use super::kanji_card_details::{KanjiCardDetails, RadicalDisplay};
use super::training_view::TrainingBody;
use crate::i18n::*;
use crate::ui_components::{
    Button, ButtonVariant, FuriganaText, MarkdownText, MarkdownVariant, ReadingItem, Tag,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{AcquaintanceSubphase, NativeLanguage};
use origa::use_cases::MarkCardAsKnownUseCase;
use std::collections::HashSet;
use ulid::Ulid;

/// Префаза руки знакомства на странице урока. Покрывает фазу показа (S4);
/// тренировка (S5) и итоговый экран (S6) подключаются следующими срезами.
#[component]
pub fn AcquaintanceView() -> impl IntoView {
    let ctx_stored = StoredValue::new(
        use_context::<AcquaintanceContext>().expect("acquaintance context not provided"),
    );
    let i18n = use_i18n();

    let phase_label = Signal::derive(move || {
        let ctx = ctx_stored.get_value();
        match ctx.state.with(|state| state.stage) {
            AcquaintanceStage::Presentation => i18n
                .get_keys()
                .acquaintance()
                .phase_presentation()
                .inner()
                .to_string(),
            AcquaintanceStage::Training => {
                let base = i18n
                    .get_keys()
                    .acquaintance()
                    .phase_training()
                    .inner()
                    .to_string();
                match ctx
                    .state
                    .with(|state| state.hand.as_ref().and_then(|h| h.subphase()))
                {
                    Some(AcquaintanceSubphase::Forward) => format!(
                        "{base} · {}",
                        i18n.get_keys().acquaintance().dir_forward().inner()
                    ),
                    Some(AcquaintanceSubphase::Reverse) => format!(
                        "{base} · {}",
                        i18n.get_keys().acquaintance().dir_reverse().inner()
                    ),
                    None => base,
                }
            },
            AcquaintanceStage::Summary | AcquaintanceStage::Inactive => String::new(),
        }
    });

    let total = Signal::derive(move || {
        let ctx = ctx_stored.get_value();
        ctx.state
            .with(|state| state.hand.as_ref().map(|h| h.len()).unwrap_or(0))
    });
    // Прогресс полосы: видимый счётчик каждой карты в текущей подфазе
    // (в показе все нули, в тренировке — из доменной машины).
    let progress = Signal::derive(move || {
        let ctx = ctx_stored.get_value();
        ctx.state.with(|state| {
            let Some(hand) = state.hand.as_ref() else {
                return Vec::new();
            };
            let subphase = hand.subphase();
            hand.presentation_order()
                .iter()
                .map(|id| match hand.entry(*id) {
                    // Выведенная («Уже знаю») карта схлопывает ячейку.
                    Some(e) if e.is_retired() => u8::MAX,
                    Some(e) => e.progress_in(subphase),
                    None => 0,
                })
                .collect()
        })
    });

    view! {
        <div data-testid="acquaintance-view" class="relative px-0.5 sm:px-1 py-1 sm:py-2">
            <div class="flex items-center justify-between gap-3 mb-2">
                <Tag test_id=Signal::derive(|| "acquaintance-phase-tag".to_string())>
                    {move || phase_label.get()}
                </Tag>
                <HandProgressStrip total progress />
            </div>

            <Show when=move || {
                let ctx = ctx_stored.get_value();
                ctx.state.get().stage == AcquaintanceStage::Presentation
            }>
                <PresentationBody ctx=ctx_stored.get_value() />
            </Show>
            <Show when=move || {
                let ctx = ctx_stored.get_value();
                ctx.state.get().stage == AcquaintanceStage::Training
            }>
                <TrainingBody ctx=ctx_stored.get_value() />
            </Show>
            <Show when=move || {
                let ctx = ctx_stored.get_value();
                ctx.state.get().stage == AcquaintanceStage::Summary
            }>
                <div data-testid="acquaintance-summary" class="text-center py-10 space-y-6">
                    <div class="stamp inline-block">
                        {t!(i18n, acquaintance.summary_stamp)}
                    </div>
                    <div>
                        <Button
                            variant=Signal::derive(|| ButtonVariant::Filled)
                            on_click=Callback::new(move |_| {
                                ctx_stored.get_value().state.update(|state| {
                                    state.stage = AcquaintanceStage::Inactive;
                                });
                            })
                            test_id=Signal::derive(|| "acquaintance-to-reviews-btn".to_string())
                        >
                            {t!(i18n, acquaintance.to_reviews)}
                        </Button>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn PresentationBody(ctx: AcquaintanceContext) -> impl IntoView {
    let ctx_stored = StoredValue::new(ctx);

    // Динамическая сборка слайда: пересоздаётся на каждом изменении индекса
    // (паттерн lesson_card_renderer).
    let render_slide = move || {
        let ctx = ctx_stored.get_value();
        let state = ctx.state.get();
        let slides = ctx.slides.get();
        let Some(slide) = slides.get(state.slide_index) else {
            return ().into_any();
        };
        match slide {
            AcquaintanceSlideData::Vocabulary {
                word,
                pos_label,
                translations,
                ..
            } => view! {
                <WordSlide
                    word=word.clone()
                    pos_label=pos_label.clone()
                    translations=translations.clone()
                    known_kanji=ctx.known_kanji
                />
            }
            .into_any(),
            AcquaintanceSlideData::Kanji {
                kanji,
                name,
                radicals,
                example_words,
                on_readings,
                kun_readings,
                ..
            } => view! {
                <KanjiSlide
                    kanji=kanji.clone()
                    name=name.clone()
                    radicals=radicals.clone()
                    example_words=example_words.clone()
                    on_readings=on_readings.clone()
                    kun_readings=kun_readings.clone()
                    known_kanji=ctx.known_kanji
                    native_language=ctx.native_language.get_untracked()
                />
            }
            .into_any(),
            AcquaintanceSlideData::Grammar {
                title,
                short_description,
                how_to_form,
                examples,
                explanation,
                nuances,
                ..
            } => view! {
                <GrammarSlide
                    title=title.clone()
                    short_description=short_description.clone()
                    how_to_form=how_to_form.clone()
                    examples=examples.clone()
                    explanation=explanation.clone()
                    nuances=nuances.clone()
                    known_kanji=ctx.known_kanji
                />
            }
            .into_any(),
        }
    };

    view! {
        <div class="min-h-[60vh] flex flex-col" data-testid="acquaintance-slide">
            <div class="flex-1 py-6">{render_slide}</div>
            <ActionBar ctx=ctx_stored.get_value() />
        </div>
    }
}

#[component]
fn WordSlide(
    word: String,
    pos_label: Option<String>,
    translations: Vec<String>,
    known_kanji: RwSignal<HashSet<char>>,
) -> impl IntoView {
    let word_stored = StoredValue::new(word.clone());
    let pos_stored = StoredValue::new(pos_label.clone());
    let translations_stored = StoredValue::new(translations.clone());
    view! {
        <div class="text-center space-y-4" data-testid="acquaintance-word-slide">
            <p class="font-serif text-5xl leading-tight text-[var(--fg-black)] break-words">
                <FuriganaText text=word_stored.get_value() known_kanji=known_kanji.get_untracked() />
            </p>
            <Show when=move || pos_label.is_some()>
                <Tag test_id=Signal::derive(|| "acquaintance-word-pos".to_string())>
                    {move || pos_stored.get_value().clone().unwrap_or_default()}
                </Tag>
            </Show>
            <ul class="space-y-1 pt-2">
                {move || {
                    translations_stored
                        .get_value()
                        .into_iter()
                        .map(|translation| {
                            view! {
                                <li class="font-mono text-sm text-[var(--fg-black)]">
                                    {translation}
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ul>
        </div>
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
fn KanjiSlide(
    kanji: String,
    name: String,
    radicals: Option<Vec<RadicalDisplay>>,
    example_words: Option<Vec<(String, String)>>,
    on_readings: Option<Vec<ReadingItem>>,
    kun_readings: Option<Vec<ReadingItem>>,
    known_kanji: RwSignal<HashSet<char>>,
    native_language: NativeLanguage,
) -> impl IntoView {
    view! {
        <div class="text-center" data-testid="acquaintance-kanji-slide">
            <KanjiCardDetails
                kanji
                name
                radicals
                example_words
                on_readings
                kun_readings
                known_kanji=known_kanji.get_untracked()
                native_language=native_language
            />
        </div>
    }
}

#[component]
#[allow(clippy::too_many_arguments)]
fn GrammarSlide(
    title: String,
    short_description: String,
    how_to_form: String,
    examples: String,
    explanation: String,
    nuances: String,
    known_kanji: RwSignal<HashSet<char>>,
) -> impl IntoView {
    let stored_title = StoredValue::new(title);
    let stored_short = StoredValue::new(short_description);
    let stored_how_to = StoredValue::new(how_to_form);
    let stored_examples = StoredValue::new(examples);
    let stored_explanation = StoredValue::new(explanation);
    let stored_nuances = StoredValue::new(nuances);
    let kk = known_kanji.get_untracked();
    let kk_for_how_to = kk.clone();
    let kk_for_examples = kk.clone();
    view! {
        <div class="space-y-4" data-testid="acquaintance-grammar-slide">
            <h2 class="font-serif text-3xl text-[var(--fg-black)]">
                {stored_title.get_value()}
            </h2>
            <p class="font-mono text-sm">{stored_short.get_value()}</p>
            <Show when=move || !stored_how_to.get_value().is_empty()>
                <div class="border border-[var(--border-light)] bg-[var(--bg-warm)] p-4">
                    <MarkdownText
                        content=Signal::derive(move || stored_how_to.get_value())
                        known_kanji=kk_for_how_to.clone()
                        variant=Signal::derive(|| MarkdownVariant::Compact)
                    />
                </div>
            </Show>
            <Show when=move || !stored_examples.get_value().is_empty()>
                <MarkdownText
                    content=Signal::derive(move || stored_examples.get_value())
                    known_kanji=kk_for_examples.clone()
                    variant=Signal::derive(|| MarkdownVariant::Default)
                />
            </Show>
            <Show when=move || !stored_explanation.get_value().is_empty()>
                <p class="font-mono text-sm text-[var(--fg-muted)] whitespace-pre-line">
                    {stored_explanation.get_value()}
                </p>
            </Show>
            <Show when=move || !stored_nuances.get_value().is_empty()>
                <p class="font-mono text-sm text-[var(--fg-muted)] whitespace-pre-line">
                    {stored_nuances.get_value()}
                </p>
            </Show>
        </div>
    }
}

/// Action bar фазы показа: слева «Уже знаю» (Ghost → inline-confirm),
/// справа «Дальше». Подтверждение живёт только на текущем слайде.
#[component]
fn ActionBar(ctx: AcquaintanceContext) -> impl IntoView {
    let i18n = use_i18n();

    let advance = Callback::new(move |_: ()| {
        ctx.state.update(|state| {
            state.confirm_known = false;
            if state.advance_presentation() {
                state.stage = AcquaintanceStage::Training;
            }
        });
    });

    let mark_known_and_advance = {
        let ctx = ctx.clone();
        Callback::new(move |card_id: Ulid| {
            let mut complete_now = false;
            ctx.state.update(|state| {
                state.confirm_known = false;
                state.skipped_ids.insert(card_id);
                // Карта выбывает из руки: критерий считается выполненным,
                // тренировка и подфазы её больше не ждут (спека §Тренировка).
                if let Some(hand) = state.hand.as_mut() {
                    hand.retire_card(card_id);
                }
                if state.advance_presentation() {
                    // Активных карт не осталось — рука закрывается без тренировки:
                    // сценарий спеки «все карты отмечены известными».
                    complete_now = !state.hand.as_ref().is_some_and(|hand| {
                        hand.presentation_order()
                            .iter()
                            .any(|id| hand.entry(*id).is_some_and(|e| !e.is_retired()))
                    });
                    if !complete_now {
                        state.stage = AcquaintanceStage::Training;
                    }
                }
            });
            if complete_now {
                ctx.complete_hand_and_show_summary();
            }
        })
    };

    let repo_stored = StoredValue::new(ctx.repository.clone());
    // Защита от двойного клика «Да, знаю» на время асинхронной записи.
    let known_in_flight = RwSignal::new(false);
    let on_yes_know = {
        Callback::new(move |_: ()| {
            if known_in_flight.get_untracked() {
                return;
            }
            let index = ctx.state.get_untracked().slide_index;
            let Some(card_id) = ctx
                .slides
                .get_untracked()
                .get(index)
                .map(|slide| slide.card_id())
            else {
                return;
            };
            known_in_flight.set(true);
            let repo = repo_stored.get_value();
            spawn_local(async move {
                // «Уже знаю» идёт существующим механизмом mark-as-known и не
                // тратит дневной лимит (docs/acquaintance-mode.md §4).
                if let Err(e) = MarkCardAsKnownUseCase::new(&repo).execute(card_id).await {
                    tracing::error!("Mark-as-known failed for {card_id}: {e}");
                }
                mark_known_and_advance.run(card_id);
                known_in_flight.set(false);
            });
        })
    };

    let on_confirm_open = {
        let ctx = ctx.clone();
        Callback::new(move |_: ()| {
            ctx.state.update(|state| state.confirm_known = true);
        })
    };

    let on_cancel = {
        let ctx = ctx.clone();
        Callback::new(move |_: ()| {
            ctx.state.update(|state| state.confirm_known = false);
        })
    };

    view! {
        <div
            class="mt-4 flex items-center justify-between gap-3"
            data-testid="acquaintance-action-bar"
        >
            <Show
                when=move || ctx.state.get().confirm_known
                fallback=move || {
                    view! {
                        <Button
                            variant=Signal::derive(|| ButtonVariant::Ghost)
                            on_click=Callback::new(move |_| on_confirm_open.run(()))
                            test_id=Signal::derive(|| "acquaintance-know-btn".to_string())
                        >
                            {t!(i18n, acquaintance.already_know)}
                        </Button>
                    }
                }
            >
                <div
                    class="flex items-center gap-2 border border-[var(--border-dark)] px-3 py-2"
                    data-testid="acquaintance-know-confirm-panel"
                >
                    <span class="font-mono text-xs uppercase tracking-widest">
                        {t!(i18n, acquaintance.confirm_known)}
                    </span>
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Olive)
                        on_click=Callback::new(move |_| on_yes_know.run(()))
                        test_id=Signal::derive(|| "acquaintance-know-confirm".to_string())
                    >
                        {t!(i18n, acquaintance.yes_i_know)}
                    </Button>
                    <Button
                        variant=Signal::derive(|| ButtonVariant::Default)
                        on_click=Callback::new(move |_| on_cancel.run(()))
                        test_id=Signal::derive(|| "acquaintance-know-cancel".to_string())
                    >
                        {t!(i18n, acquaintance.cancel)}
                    </Button>
                </div>
            </Show>

            <Show when=move || !ctx.state.get().confirm_known fallback=move || ()>
                <Button
                    variant=Signal::derive(|| ButtonVariant::Filled)
                    on_click=Callback::new(move |_| advance.run(()))
                    test_id=Signal::derive(|| "acquaintance-next-btn".to_string())
                >
                    <span>{t!(i18n, lesson.next)}</span>
                </Button>
            </Show>
        </div>
    }
}
