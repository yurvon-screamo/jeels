use super::acquaintance_keyboard::{
    AcquaintanceKeyboardActions, create_acquaintance_keyboard_handler,
};
use super::acquaintance_state::{
    AcquaintanceContext, AcquaintanceSlideData, should_autoplay_word_audio,
};
use super::grammar_example::{first_example_markdown, grammar_example_front};
use super::keyboard_handler::is_typing_target;
use crate::i18n::*;
use crate::ui_components::{
    Button, ButtonVariant, FuriganaText, MarkdownText, MarkdownVariant, ReadingGroup,
    is_speech_supported, speak_word,
};
use leptos::prelude::*;
use leptos_use::use_event_listener;
use origa::domain::{AcquaintanceSubphase, AnswerOutcome, NativeLanguage};
use ulid::Ulid;

/// Раскрытие ответа: и кнопка, и Space ведут себя одинаково.
/// Окно финиша руки (hand_finishing) игнорируется: карта уже отвечена,
/// повторное раскрытие невозможно.
fn do_reveal(
    ctx_stored: &StoredValue<AcquaintanceContext>,
    current_id: &Memo<Ulid>,
    showing_answer: &RwSignal<bool>,
) {
    let ctx = ctx_stored.get_value();
    if ctx.state.with_untracked(|state| state.hand_finishing) {
        return;
    }
    showing_answer.set(true);
    let card_id = current_id.get_untracked();
    if !card_id.is_nil() {
        speak_reverse_answer(&ctx, card_id);
    }
}

/// Запись ответа: и кнопки [1]/[2], и клавиши 1/2 ведут себя одинаково.
/// Окно финиша руки (hand_finishing) игнорируется — защита от двойного
/// клика/клавиши до монтирования экрана завершения.
fn do_rate(
    ctx_stored: &StoredValue<AcquaintanceContext>,
    current_id: &Memo<Ulid>,
    showing_answer: &RwSignal<bool>,
    rotation_index: &RwSignal<usize>,
    training_order: &RwSignal<Vec<Ulid>>,
    remembered: bool,
) {
    let ctx = ctx_stored.get_value();
    if ctx.state.with_untracked(|state| state.hand_finishing) {
        return;
    }
    let outcome = record_on_hand(&ctx, current_id.get_untracked(), remembered);
    finish_answer(
        &ctx,
        showing_answer,
        rotation_index,
        training_order,
        outcome,
    );
    // Шапке нужен тип новой текущей карты (тег типа).
    ctx.current_card.set(current_id.get_untracked().non_nil());
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
    // with_untracked: чтение при создании компонента вне реактивного
    // контекста не должно подписываться (и не плодит console-warning).
    ctx.state.with_untracked(|state| {
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
/// закрывшие критерий продолжают отвечаться: успехи заморожены, «Не помню»
/// переоткрывает карту (шкала подфазы — в ноль).
///
/// Единственный источник истины для перерисовки — `AnswerOutcome`
/// из доменной машины; UI не дублирует подсчёт успехов.
#[component]
pub fn TrainingBody(ctx: AcquaintanceContext) -> impl IntoView {
    let i18n = use_i18n();
    let ctx_stored = StoredValue::new(ctx);
    // Раскрытие ответа живёт в контексте: шапке он нужен для видимости
    // кнопки озвучки (JP-сторона скрыта на Reverse-фронте).
    let showing_answer = ctx_stored.get_value().showing_answer;
    showing_answer.set(false);
    let rotation_index = RwSignal::new(0usize);
    let training_order = RwSignal::new(build_rotation_order(&ctx_stored.get_value()));

    let current_id = Memo::new(move |_| {
        let order = training_order.get();
        if order.is_empty() {
            return Ulid::nil();
        }
        order[rotation_index.get() % order.len()]
    });

    // Автозвук Forward-фронта — один на СМЕНУ карты: Memo по card_id
    // молчит при перезапусках рендер-замыкания с той же картой (запись
    // ответа, пересборка slides), звучит только новой карте. Reverse
    // озвучивается при раскрытии ответа (speak_reverse_answer).
    // Тот же канал, что и автозвук обычного урока (speak_word): сначала
    // CDN pitch-аудио файла, TTS — только fallback (юзер-репорт: в
    // знакомстве звучал TTS вместо аудиофайла).
    let is_muted =
        use_context::<super::lesson_state::LessonContext>().map(|lesson_ctx| lesson_ctx.is_muted);
    let autoplay_ctx = ctx_stored;
    Effect::new(move |_| {
        let card_id = current_id.get();
        if card_id.is_nil() {
            return;
        }
        let ctx = autoplay_ctx.get_value();
        let forward = ctx
            .state
            .with_untracked(|state| state.hand.as_ref().and_then(|h| h.subphase()))
            == Some(AcquaintanceSubphase::Forward);
        if !forward {
            return;
        }
        let muted = is_muted
            .as_ref()
            .map(|signal| signal.get_untracked())
            .unwrap_or(false);
        if !should_autoplay_word_audio(muted, is_speech_supported()) {
            return;
        }
        let word = ctx
            .slides
            .get_untracked()
            .iter()
            .find(|slide| slide.card_id() == card_id)
            .and_then(|slide| slide.word().map(str::to_string));
        if let Some(word) = word {
            speak_word(&word, 1.0);
        }
    });

    // Текущая карта тренировки для тега типа в шапке: отдельный сигнал,
    // запись при монтаже не перезапускает родительские Show.
    ctx_stored
        .get_value()
        .current_card
        .set(current_id.get_untracked().non_nil());

    // Клавиатура: те же хендлы, что у кнопок (спека §8.3). Гейты окна
    // финиша — внутри do_reveal/do_rate.
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

    // Окно финиша: кнопки ухода со слайда скрыты, пока рука закрывается.
    let hand_finishing = Signal::derive(move || {
        let ctx = ctx_stored.get_value();
        ctx.state.with(|state| state.hand_finishing)
    });

    view! {
        <div
            class="flex flex-col grow"
            data-testid="acquaintance-training"
            data-card-id=move || {
                let id = current_id.get();
                if id.is_nil() {
                    String::new()
                } else {
                    id.to_string()
                }
            }
        >
            <div class="flex-1 py-2 sm:py-3">
                {move || {
                    let Some(card_id) = current_id.get().non_nil() else {
                        return ().into_any();
                    };
                    let reverse = ctx_stored.get_value().state.with(|state| {
                        state.hand.as_ref().and_then(|h| h.subphase())
                    }) == Some(AcquaintanceSubphase::Reverse);
                    // Вопрос остаётся на стороне ответа — уменьшенным и
                    // приглушённым сверху, под ним divider и ответ
                    // (паттерн обычного урока, lesson_card_answer).
                    // Приглушение — семантический класс front-dimmed, не
                    // tailwind opacity-60: голая opacity каскадировала в
                    // тултип кандзи и делала его полупрозрачным.
                    view! {
                        <div
                            class=move || if showing_answer.get() {
                                "pt-1 pb-2 scale-90 origin-top front-dimmed"
                            } else {
                                ""
                            }
                        >
                            <TrainingFrontSlide
                                ctx=ctx_stored.get_value()
                                card_id=card_id
                                reverse=reverse
                            />
                        </div>
                        <Show when=move || showing_answer.get() fallback=move || ()>
                            <div
                                class="border-t border-[var(--border-light)] my-2"
                                data-testid="acquaintance-answer-divider"
                            ></div>
                            <TrainingAnswerSlide
                                ctx=ctx_stored.get_value()
                                card_id=card_id
                                reverse=reverse
                            />
                        </Show>
                    }
                    .into_any()
                }}
            </div>
            <Show when=move || !showing_answer.get() && !hand_finishing.get() fallback=move || ()>
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
            <Show when=move || showing_answer.get() && !hand_finishing.get() fallback=move || ()>
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
/// Действие после ответа тренировки (docs/acquaintance-mode.md,
/// правило «Тренировка», H-итерация UX):
/// - успех, закрывший forward-критерий последнему активному слову, меняет
///   сторону НЕМЕДЛЕННО: полоса заполнилась — юзер видит переход и
///   сброшенные шкалы, а не «полную полосу без перехода»;
/// - иначе — следующая карта круга; на границе круга порядок
///   перемешивается. Ошибка юзера порядок не трогает.
pub enum AfterAnswerAction {
    /// Все активные слова закрыли подфазу: новый круг с нуля (Reverse),
    /// порядок перемешивается.
    SwitchedSubphase,
    /// Следующая карта текущего круга; `reshuffled` — круг закончился и
    /// порядок нового круга перемешен.
    NextCard { reshuffled: bool },
}

pub fn after_answer(
    success: bool,
    rotation_index: usize,
    active_len: usize,
    hand: &mut origa::domain::AcquaintanceHand,
) -> AfterAnswerAction {
    if success && hand.advance_subphase_if_words_done() {
        return AfterAnswerAction::SwitchedSubphase;
    }
    if active_len == 0 || (rotation_index + 1) % active_len != 0 {
        return AfterAnswerAction::NextCard { reshuffled: false };
    }
    AfterAnswerAction::NextCard { reshuffled: true }
}

/// Перемешивание круга с защитой стыка: первая карта нового круга не
/// повторяет последнюю карту предыдущего — иначе одна карта идёт дважды
/// подряд и кажется, что круг «застрял».
fn reshuffle_avoiding_repeat(prev_last: Option<Ulid>, mut cards: Vec<Ulid>) -> Vec<Ulid> {
    if cards.len() > 1 {
        cards = shuffled_order(cards);
        if prev_last.is_some_and(|last| cards[0] == last) {
            cards.swap(0, 1);
        }
    }
    cards
}

fn finish_answer(
    ctx: &AcquaintanceContext,
    showing_answer: &RwSignal<bool>,
    rotation_index: &RwSignal<usize>,
    training_order: &RwSignal<Vec<Ulid>>,
    outcome: AnswerOutcome,
) {
    // Финальный ответ: рука закрывается. Отвеченная карта ОСТАЁТСЯ на
    // экране (showing_answer не сбрасывается) — без этого фронт того же
    // слова пере-показывался «лишним вопросом», пока идёт запись и до
    // монтирования экрана завершения (юзер-репорт о резком переходе).
    // Кнопки и клавиатура гасятся флагом hand_finishing.
    if matches!(outcome, AnswerOutcome::HandCompleted) {
        ctx.complete_hand();
        return;
    }

    showing_answer.set(false);

    let prev_last = training_order.get_untracked().last().copied();
    let mut action = AfterAnswerAction::NextCard { reshuffled: false };
    ctx.state.update(|state| {
        if let Some(hand) = state.hand.as_mut() {
            action = after_answer(
                matches!(outcome, AnswerOutcome::Counted { .. }),
                rotation_index.get_untracked(),
                training_order.get_untracked().len(),
                hand,
            );
        }
    });
    match action {
        AfterAnswerAction::SwitchedSubphase => {
            // Сторона сменилась в момент заполнения полосы: новый круг
            // с нуля в перемешанном порядке.
            rotation_index.set(0);
            training_order.set(reshuffle_avoiding_repeat(
                prev_last,
                training_order.get_untracked(),
            ));
        },
        AfterAnswerAction::NextCard { reshuffled } => {
            rotation_index.set(rotation_index.get_untracked() + 1);
            if reshuffled {
                training_order.set(reshuffle_avoiding_repeat(
                    prev_last,
                    training_order.get_untracked(),
                ));
            }
        },
    }
}

/// Повтор аудио слова в ответе Reverse-подфазы (спека §8.2); guard
/// is_speech_supported гасит среды без TTS.
fn speak_if_supported(word: &str) {
    if is_speech_supported() {
        speak_word(word, 1.0);
    }
}

/// Forward-фронт слова в тренировке: чистый рендер — автозвук живёт в
/// `TrainingBody` одним Effect'ом по Memo текущей карты (дедуп: один
/// звук на смену карты, а не на каждое перемонтирование фронта).
#[component]
fn WordTrainingFront(
    word: String,
    known_kanji: std::collections::HashSet<char>,
    native_language: NativeLanguage,
) -> impl IntoView {
    view! {
        <p class="font-serif text-5xl text-[var(--fg-black)] break-words">
            <FuriganaText
                text=word
                known_kanji
                native_language=native_language
                with_kanji_tooltip=true
            />
        </p>
    }
}

/// Фронт тренировки: японская сторона (Forward) или перевод (Reverse,
/// только слова). Кандзи показывают только знак — значение является
/// ответом; грамматика — японскую строку примера без перевода (смысл
/// тоже ответ, спека §Тренировка).
#[component]
fn TrainingFrontSlide(ctx: AcquaintanceContext, card_id: Ulid, reverse: bool) -> impl IntoView {
    let known_kanji = ctx.known_kanji;
    // Отступы фронта зависят от фазы: пока юзер думает — воздух вокруг
    // вопроса; после раскрытия ответа вопрос сжимается в шапку ответа
    // (баг-репорт: огромные отступы съедали место на стороне ответа).
    let front_class = move || {
        if ctx.showing_answer.get() {
            "text-center py-1"
        } else {
            "text-center pt-8 pb-12 sm:pt-10 sm:pb-16"
        }
    };
    view! {
        <div class=front_class data-testid="acquaintance-training-front">
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
                                <WordTrainingFront
                                    word=word
                                    known_kanji=known_kanji.get_untracked()
                                    native_language=ctx.native_language.get_untracked()
                                />
                            }
                                .into_any()
                        }
                    },
                    // Только знак: значение и чтения — ответ.
                    AcquaintanceSlideData::Kanji { kanji, .. } => view! {
                        <p class="font-serif text-6xl text-[var(--fg-black)]">{kanji}</p>
                    }
                        .into_any(),
                    AcquaintanceSlideData::Grammar { title, examples, .. } => {
                        // Пустые examples — фронт вырождается в заголовок
                        // конструкции («знак» правила, не смысл).
                        let front =
                            grammar_example_front(&examples).unwrap_or_else(|| title.clone());
                        view! {
                            <p class="font-serif text-3xl text-[var(--fg-black)] leading-relaxed">
                                {front}
                            </p>
                        }
                            .into_any()
                    },
                }
            }}
        </div>
    }
}

/// Ответ тренировки: противоположная фронту сторона. Для слов Reverse —
/// слово с фуриганой и повтор аудио (спека §8.2); кандзи раскрывают
/// значения и частотные чтения; грамматика — смысл с полным примером.
#[component]
fn TrainingAnswerSlide(ctx: AcquaintanceContext, card_id: Ulid, reverse: bool) -> impl IntoView {
    let known_kanji = ctx.known_kanji;
    let i18n = use_i18n();
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
                                    <FuriganaText
                                        text=word
                                        known_kanji=known_kanji.get_untracked()
                                        native_language=ctx.native_language.get_untracked()
                                        with_kanji_tooltip=true
                                    />
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
                    AcquaintanceSlideData::Kanji {
                        name,
                        on_readings,
                        kun_readings,
                        ..
                    } => {
                        let has_on = on_readings.is_some();
                        let has_kun = kun_readings.is_some();
                        let on = StoredValue::new(on_readings);
                        let kun = StoredValue::new(kun_readings);
                        // Знак уже смотрит на юзера сжатым вопросом над
                        // divider — ответ его не повторяет (баг-репорт о
                        // дубле): главным текстом идёт значение, под ним
                        // частотные чтения по центральной оси карточки.
                        view! {
                            <p class="font-serif text-3xl text-[var(--fg-black)]">{name}</p>
                            <div class="answer-readings pt-2 space-y-2">
                                {has_on.then(|| {
                                    view! {
                                        <ReadingGroup
                                            label=Signal::derive(move || {
                                                i18n.get_keys().lesson().on_yomi().inner().to_string()
                                            })
                                            readings=on
                                        />
                                    }
                                })}
                                {has_kun.then(|| {
                                    view! {
                                        <ReadingGroup
                                            label=Signal::derive(move || {
                                                i18n.get_keys().lesson().kun_yomi().inner().to_string()
                                            })
                                            readings=kun
                                        />
                                    }
                                })}
                            </div>
                        }
                            .into_any()
                    },
                    AcquaintanceSlideData::Grammar {
                        title,
                        short_description,
                        examples,
                        ..
                    } => {
                        // Фронт — JP-пример; при пустых examples фронт был
                        // заголовком конструкции, и ответ не дублирует его.
                        let example = first_example_markdown(&examples);
                        let front_was_title = example.is_none();
                        let examples_stored = StoredValue::new(example.unwrap_or_default());
                        let title_stored = StoredValue::new(title);
                        view! {
                            <Show when=move || !front_was_title>
                                <h2 class="font-serif text-2xl text-[var(--fg-black)]">
                                    {title_stored.get_value()}
                                </h2>
                            </Show>
                            <p class="font-mono text-sm">{short_description}</p>
                            <Show when=move || !front_was_title>
                                <MarkdownText
                                    content=Signal::derive(move || examples_stored.get_value())
                                    known_kanji=known_kanji.get_untracked()
                                    variant=Signal::derive(|| MarkdownVariant::Compact)
                                />
                            </Show>
                        }
                            .into_any()
                    },
                }
            }}
        </div>
    }
}

#[cfg(test)]
mod rotation_tests {
    use super::*;
    use origa::domain::CardType;

    fn two_word_hand() -> (origa::domain::AcquaintanceHand, Ulid, Ulid) {
        let a = Ulid::new();
        let b = Ulid::new();
        let hand = origa::domain::AcquaintanceHand::new(vec![
            (a, CardType::Vocabulary),
            (b, CardType::Vocabulary),
        ])
        .unwrap();
        (hand, a, b)
    }

    /// Полный цикл двух слов: пока не все закрыли forward — следующая
    /// карта круга; закрывающий успех меняет сторону немедленно, и шкалы
    /// слов сбрасываются (полоса больше не «висит заполненной»).
    #[test]
    fn closing_success_switches_subphase_immediately() {
        // Arrange: [a, b], все ответы «помню»
        let (mut hand, a, b) = two_word_hand();

        // Круги 1-2 (ответы с индексами 0..3): сторона та же.
        for answer_index in 0..4 {
            let action = after_answer(true, answer_index, 2, &mut hand);
            let at_boundary = (answer_index + 1) % 2 == 0;
            assert!(matches!(
                action,
                AfterAnswerAction::NextCard { reshuffled } if reshuffled == at_boundary
            ));
            hand.record_answer(if answer_index % 2 == 0 { a } else { b }, true)
                .unwrap();
        }

        // 5-й ответ (индекс 4): a закрыл forward, но b ещё нет — смены нет
        // (и это ещё не граница круга).
        hand.record_answer(a, true).unwrap();
        assert!(matches!(
            after_answer(true, 4, 2, &mut hand),
            AfterAnswerAction::NextCard { reshuffled: false }
        ));

        // 6-й ответ (индекс 5): b закрывает forward — смена НЕМЕДЛЕННО.
        hand.record_answer(b, true).unwrap();
        assert!(matches!(
            after_answer(true, 5, 2, &mut hand),
            AfterAnswerAction::SwitchedSubphase
        ));
        assert_eq!(
            hand.subphase(),
            Some(origa::domain::AcquaintanceSubphase::Reverse)
        );
    }

    /// Закрытие mid-круга (ошибка отодвинула слово) тоже меняет сторону
    /// сразу — юзер не доигрывает круг с «полной полосой».
    #[test]
    fn mid_circle_closing_answer_switches_without_waiting_boundary() {
        // Arrange: [a, b]; b ошибся на первом круге
        let (mut hand, a, b) = two_word_hand();
        hand.record_answer(a, true).unwrap();
        hand.record_answer(b, false).unwrap();
        hand.record_answer(a, true).unwrap();
        hand.record_answer(b, true).unwrap();
        hand.record_answer(a, true).unwrap(); // a закрыл (5-й, граница была на 4-м и 6-м)
        assert!(matches!(
            after_answer(true, 5, 2, &mut hand),
            AfterAnswerAction::NextCard { reshuffled: true }
        ));
        hand.record_answer(b, true).unwrap(); // b: 2/3 (7-й ответ, индекс 5)
        assert!(matches!(
            after_answer(true, 5, 2, &mut hand),
            AfterAnswerAction::NextCard { reshuffled: true }
        ));
        // 8-й ответ (индекс 6, середина круга): b закрывает forward —
        // смена сразу, не ждём границы.
        hand.record_answer(b, true).unwrap(); // b: 3/3
        assert!(matches!(
            after_answer(true, 6, 2, &mut hand),
            AfterAnswerAction::SwitchedSubphase
        ));
    }

    /// Ошибка не двигает смену: subphase остаётся, только карта круга.
    #[test]
    fn failed_answer_keeps_subphase_and_order() {
        let (mut hand, _a, _b) = two_word_hand();
        let outcome = hand.record_answer(_a, false).unwrap();
        assert!(matches!(outcome, AnswerOutcome::Failed));
        assert!(matches!(
            after_answer(false, 0, 2, &mut hand),
            AfterAnswerAction::NextCard { reshuffled: false }
        ));
        assert_eq!(
            hand.subphase(),
            Some(origa::domain::AcquaintanceSubphase::Forward)
        );
    }

    /// Стык кругов: первая карта нового круга не повторяет последнюю
    /// карту предыдущего (рука > 1 карты); одиночная карта неизбежно
    /// повторяется.
    #[test]
    fn reshuffle_avoiding_repeat_keeps_seam_distinct() {
        let x = Ulid::new();
        let y = Ulid::new();
        let z = Ulid::new();
        let cards = vec![y, z, x];
        let reshuffled = reshuffle_avoiding_repeat(Some(x), cards);
        assert_ne!(
            reshuffled[0], x,
            "первая карта нового круга не повторяет последнюю прошлого"
        );
        assert_eq!(reshuffled.len(), 3);

        // Одиночная карта: дубль неизбежен, порядок сохранён.
        let single = reshuffle_avoiding_repeat(Some(x), vec![x]);
        assert_eq!(single, vec![x]);
    }
}
