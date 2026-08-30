use super::acquaintance_keyboard::{AcquaintanceKeyAction, resolve_key_action};
use super::acquaintance_state::{
    AcquaintanceContext, AcquaintanceSlideData, AcquaintanceStage, audio_button_visible,
    should_autoplay_word_audio,
};
use super::card_type::CardType as UiCardType;
use super::hand_progress_strip::{HandProgressStrip, presentation_fill};
use super::kanji_card_details::{KanjiCardDetails, RadicalDisplay};
use super::keyboard_handler::is_typing_target;
use super::training_view::TrainingBody;
use crate::i18n::*;
use crate::ui_components::{
    Button, ButtonVariant, ConfirmModal, FuriganaText, KanjiAnimation, MarkdownText,
    MarkdownVariant, ReadingItem, Tag, is_speech_supported, speak_word,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_icons::Icon;
use leptos_use::use_event_listener;
use origa::domain::{AcquaintanceSubphase, NativeLanguage};
use origa::use_cases::MarkCardAsKnownUseCase;
use std::collections::HashSet;
use ulid::Ulid;

/// Маппинг доменного типа карты на UI-тип с лейблом и цветом тега
/// (как в онбординге — CardType::label + tag_variant).
fn ui_card_type(card_type: origa::domain::CardType) -> UiCardType {
    match card_type {
        origa::domain::CardType::Vocabulary => UiCardType::Vocabulary,
        origa::domain::CardType::Kanji => UiCardType::Kanji,
        origa::domain::CardType::Grammar => UiCardType::Grammar,
        origa::domain::CardType::Phrase => UiCardType::Phrase,
    }
}

/// Префаза руки знакомства на странице урока: показ, тренировка.
/// Итогового экрана нет — закрытая рука сразу открывает ревью (F-итерация).
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
            AcquaintanceStage::Training => i18n
                .get_keys()
                .acquaintance()
                .phase_training()
                .inner()
                .to_string(),
            AcquaintanceStage::Inactive => String::new(),
        }
    });

    // Направление тренировки слов — отдельным компактным тегом: единый
    // длинный тег «фаза · направление» не влезал на мобильных и растягивал
    // строку по вертикали.
    let direction_label = Signal::derive(move || {
        let ctx = ctx_stored.get_value();
        if ctx.state.with(|state| state.stage) != AcquaintanceStage::Training {
            return None;
        }
        match ctx
            .state
            .with(|state| state.hand.as_ref().and_then(|h| h.subphase()))
        {
            Some(AcquaintanceSubphase::Forward) => Some(
                i18n.get_keys()
                    .acquaintance()
                    .dir_forward()
                    .inner()
                    .to_string(),
            ),
            Some(AcquaintanceSubphase::Reverse) => Some(
                i18n.get_keys()
                    .acquaintance()
                    .dir_reverse()
                    .inner()
                    .to_string(),
            ),
            None => None,
        }
    });

    let total = Signal::derive(move || {
        let ctx = ctx_stored.get_value();
        ctx.state
            .with(|state| state.hand.as_ref().map(|h| h.len()).unwrap_or(0))
    });

    // Тип текущей карты: в показе — слайд по индексу, в тренировке —
    // карта из отдельного сигнала контекста. Шапка показывает его вторым
    // тегом.
    let current_card_type = Signal::derive(move || {
        let ctx = ctx_stored.get_value();
        let state = ctx.state.get();
        let card_id = match state.stage {
            AcquaintanceStage::Presentation => ctx
                .slides
                .get()
                .get(state.slide_index)
                .map(|slide| slide.card_id()),
            _ => ctx.current_card.get(),
        }?;
        let hand = state.hand?;
        hand.entry(card_id).map(|entry| entry.card_type())
    });

    let card_type_tag = Signal::derive(move || current_card_type.get().map(ui_card_type));

    // Японское слово текущей карты и его часть речи — для кнопки озвучки
    // и POS-тега в шапке.
    let current_slide = Signal::derive(move || {
        let ctx = ctx_stored.get_value();
        let state = ctx.state.get();
        match state.stage {
            AcquaintanceStage::Presentation => ctx.slides.get().get(state.slide_index).cloned(),
            _ => ctx.current_card.get().and_then(|card_id| {
                ctx.slides
                    .get()
                    .iter()
                    .find(|s| s.card_id() == card_id)
                    .cloned()
            }),
        }
    });
    let current_word = Signal::derive(move || {
        current_slide
            .get()
            .and_then(|slide| slide.word().map(str::to_string))
    });
    let pos_tag_label = Signal::derive(move || {
        current_slide
            .get()
            .and_then(|slide| slide.pos_label().map(str::to_string))
    });

    // Кнопка озвучки: видна, когда японская сторона слова на экране
    // (Reverse-фронт прячет её — кнопка не подсказывает ответ).
    let audio_visible = Signal::derive(move || {
        let ctx = ctx_stored.get_value();
        let state = ctx.state.get();
        audio_button_visible(
            state.stage,
            state.hand.as_ref().and_then(|hand| hand.subphase()),
            ctx.showing_answer.get(),
            current_word.get().is_some(),
        )
    });
    let speak_current_word = Callback::new(move |_: ()| {
        if let Some(word) = current_word.get() {
            speak_word(&word, 1.0);
        }
    });

    // Прогресс полосы — единственный индикатор во время знакомства:
    // в показе ячейки заполняются по пройденным слайдам, в тренировке —
    // успешными ответами каждой карты в текущей подфазе.
    let progress = Signal::derive(move || {
        let ctx = ctx_stored.get_value();
        let state = ctx.state.get();
        let Some(hand) = state.hand.as_ref() else {
            return Vec::new();
        };
        let subphase = hand.subphase();
        if state.stage == AcquaintanceStage::Presentation {
            return presentation_fill(hand.len(), state.slide_index)
                .into_iter()
                .zip(hand.presentation_order().iter())
                .map(|(fill, card_id)| {
                    // Выведенная («Уже знаю») карта схлопывает ячейку.
                    if hand.entry(*card_id).is_some_and(|entry| entry.is_retired()) {
                        u8::MAX
                    } else {
                        fill
                    }
                })
                .collect();
        }
        hand.presentation_order()
            .iter()
            .map(|card_id| match hand.entry(*card_id) {
                Some(entry) if entry.is_retired() => u8::MAX,
                Some(entry) => entry.progress_in(subphase),
                None => 0,
            })
            .collect()
    });

    // Текстовое дублирование полосы (спека §8.4): позиция показа или число
    // карт, закрывших критерий.
    let strip_label = Signal::derive(move || {
        let ctx = ctx_stored.get_value();
        let state = ctx.state.get();
        let Some(hand) = state.hand.as_ref() else {
            return String::new();
        };
        let total = hand.len();
        let keys = i18n.get_keys().acquaintance();
        match state.stage {
            AcquaintanceStage::Presentation => format!(
                "{} {}/{}",
                keys.strip_presentation().inner(),
                (state.slide_index + 1).min(total.max(1)),
                total
            ),
            AcquaintanceStage::Training => {
                let subphase = hand.subphase();
                let closed = hand
                    .presentation_order()
                    .iter()
                    .filter(|card_id| {
                        hand.entry(**card_id)
                            .is_some_and(|entry| entry.criterion_met(subphase))
                    })
                    .count();
                format!("{}: {}/{}", keys.strip_training().inner(), closed, total)
            },
            AcquaintanceStage::Inactive => String::new(),
        }
    });

    view! {
        <div data-testid="acquaintance-view" class="relative px-0.5 sm:px-1 py-1 sm:py-2">
            <div class="flex flex-col gap-1 mb-2 sm:flex-row sm:items-center sm:justify-between">
                // Полоса — первой строкой (мобильные теги переносятся
                // ниже, а не растягивают строку по вертикали), рядом —
                // кнопка озвучки текущего слова.
                <div class="flex items-center gap-2">
                    <HandProgressStrip total progress label=strip_label />
                    <Show when=move || audio_visible.get()>
                        <button
                            data-testid="acquaintance-audio-btn"
                            class="text-[var(--fg-muted)] hover:text-[var(--fg-black)] transition-colors cursor-pointer shrink-0"
                            on:click=move |_| speak_current_word.run(())
                        >
                            <Icon icon=icondata::LuVolume2 width="16" height="16" />
                        </button>
                    </Show>
                </div>
                <div class="flex items-center gap-2 flex-wrap min-w-0">
                    <Tag test_id=Signal::derive(|| "acquaintance-phase-tag".to_string())>
                        {move || phase_label.get()}
                    </Tag>
                    <Show when=move || direction_label.get().is_some()>
                        <Tag test_id=Signal::derive(|| "acquaintance-direction-tag".to_string())>
                            {move || direction_label.get().unwrap_or_default()}
                        </Tag>
                    </Show>
                    <Show when=move || card_type_tag.get().is_some()>
                        <Tag
                            variant=Signal::derive(move || {
                                card_type_tag.get().map(|t| t.tag_variant()).unwrap_or_default()
                            })
                            test_id=Signal::derive(|| "acquaintance-card-type-tag".to_string())
                        >
                            {move || {
                                card_type_tag
                                    .get()
                                    .map(|t| t.label(&i18n))
                                    .unwrap_or_default()
                            }}
                        </Tag>
                    </Show>
                    <Show when=move || pos_tag_label.get().is_some()>
                        <Tag test_id=Signal::derive(|| "acquaintance-pos-tag".to_string())>
                            {move || pos_tag_label.get().unwrap_or_default()}
                        </Tag>
                    </Show>
                </div>
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
                    translations=translations.clone()
                    known_kanji=ctx.known_kanji
                    native_language=ctx.native_language.get_untracked()
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
    translations: Vec<String>,
    known_kanji: RwSignal<HashSet<char>>,
    native_language: NativeLanguage,
) -> impl IntoView {
    let word_stored = StoredValue::new(word.clone());
    let translations_stored = StoredValue::new(translations.clone());

    // Автозвук слова при показе слайда — тот же механизм, что у карточек
    // слов обычного урока (lesson_card.rs): TTS доступен и не выключен.
    let is_muted =
        use_context::<super::lesson_state::LessonContext>().map(|lesson_ctx| lesson_ctx.is_muted);
    Effect::new(move |_| {
        let muted = is_muted
            .as_ref()
            .map(|signal| signal.get_untracked())
            .unwrap_or(false);
        if should_autoplay_word_audio(muted, is_speech_supported()) {
            speak_word(&word_stored.get_value(), 1.0);
        }
    });

    view! {
        <div class="text-center space-y-4" data-testid="acquaintance-word-slide">
            <p class="font-serif text-5xl leading-tight text-[var(--fg-black)] break-words">
                <FuriganaText
                    text=word_stored.get_value()
                    known_kanji=known_kanji.get_untracked()
                    native_language=native_language
                    with_kanji_tooltip=true
                />
            </p>
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
            // Сам знак крупно с анимацией черт (спека §8.2) — чтения и
            // значения ниже через существующий компонент деталей.
            <div class="flex justify-center">
                <KanjiAnimation
                    kanji=kanji.clone()
                    fallback=None
                    test_id=Signal::derive(|| "acquaintance-kanji-animation".to_string())
                />
            </div>
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
    let kk_for_explanation = kk.clone();
    let kk_for_nuances = kk.clone();
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
                <div data-testid="acquaintance-grammar-explanation">
                    <MarkdownText
                        content=Signal::derive(move || stored_explanation.get_value())
                        known_kanji=kk_for_explanation.clone()
                        variant=Signal::derive(|| MarkdownVariant::Default)
                    />
                </div>
            </Show>
            <Show when=move || !stored_nuances.get_value().is_empty()>
                <div data-testid="acquaintance-grammar-nuances">
                    <MarkdownText
                        content=Signal::derive(move || stored_nuances.get_value())
                        known_kanji=kk_for_nuances.clone()
                        variant=Signal::derive(|| MarkdownVariant::Default)
                    />
                </div>
            </Show>
        </div>
    }
}

/// Action bar фазы показа: слева «Уже знаю» (Ghost → общий ConfirmModal),
/// справа «Дальше».
#[component]
fn ActionBar(ctx: AcquaintanceContext) -> impl IntoView {
    let i18n = use_i18n();
    let confirm_open = RwSignal::new(false);

    let advance = Callback::new(move |_: ()| {
        ctx.state.update(|state| {
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
                ctx.complete_hand();
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
            confirm_open.set(false);
        })
    };

    // Space = «Дальше» в показе (спека §8.3).
    let kb_ctx = StoredValue::new(ctx.clone());
    let _ = use_event_listener(document(), leptos::ev::keydown, move |ev| {
        if is_typing_target(ev.target().as_ref()) {
            return;
        }
        let stage = kb_ctx.get_value().state.get().stage;
        if resolve_key_action(stage, false, &ev.key()) == Some(AcquaintanceKeyAction::Advance) {
            ev.prevent_default();
            advance.run(());
        }
    });

    view! {
        <div
            class="mt-4 flex items-center justify-between gap-3"
            data-testid="acquaintance-action-bar"
        >
            <Button
                variant=Signal::derive(|| ButtonVariant::Ghost)
                on_click=Callback::new(move |_| confirm_open.set(true))
                test_id=Signal::derive(|| "acquaintance-know-btn".to_string())
            >
                {t!(i18n, acquaintance.already_know)}
            </Button>

            <Button
                variant=Signal::derive(|| ButtonVariant::Filled)
                on_click=Callback::new(move |_| advance.run(()))
                test_id=Signal::derive(|| "acquaintance-next-btn".to_string())
            >
                <span>{t!(i18n, lesson.next)}</span>
                <span class="kbd-hint text-[var(--fg-light)]">
                    {t!(i18n, lesson.space_key)}
                </span>
            </Button>
        </div>

        <ConfirmModal
            test_id=Signal::derive(|| "acquaintance-know-confirm".to_string())
            is_open=confirm_open
            is_busy=known_in_flight.into()
            title=Signal::derive(move || {
                i18n
                    .get_keys()
                    .acquaintance()
                    .confirm_known()
                    .inner()
                    .to_string()
            })
            message=Signal::derive(move || {
                i18n
                    .get_keys()
                    .acquaintance()
                    .confirm_known_message()
                    .inner()
                    .to_string()
            })
            confirm_label=Signal::derive(move || {
                i18n.get_keys().acquaintance().yes_i_know().inner().to_string()
            })
            on_confirm=on_yes_know
            on_close=Callback::new(move |_| confirm_open.set(false))
        />
    }
}
