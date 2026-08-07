use std::collections::HashSet;

use crate::i18n::*;
use crate::pages::lesson::card_type::CardType;
use crate::ui_components::{
    AudioButtons, Button, ButtonVariant, Card, FuriganaText, MarkdownText, Tag, Text, TextSize,
    TypographyVariant, is_speech_supported, speak_word, stop_current_audio,
};
use leptos::prelude::*;

use super::scoring_helpers::ScoringCard;

/// Single onboarding-scoring card. Split out of `ScoringStep` so the parent
/// stays under the 200-line file cap and so the question-area branch (kanji
/// versus the rest) is testable in isolation.
///
/// Kanji cards intentionally render without [`FuriganaText`] or
/// [`AudioButtons`]: the reading is what the user is being asked "do you know
/// this kanji?" about, so revealing it next to the character would defeat the
/// assessment. Readings + meaning land in the answer area instead.
#[component]
pub fn ScoringCardView(
    card: Signal<Option<ScoringCard>>,
    is_rating: Signal<bool>,
    on_know: Callback<()>,
    on_dont_know: Callback<()>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    // Kanji cards are not auto-spoken: hearing the reading would leak the
    // answer the assessment is asking the user to recall. Stop any
    // currently-playing audio BEFORE starting the new one so rapid card
    // transitions don't accumulate a queue of overlapping utterances
    // (especially on device-ai where synthesize is blocking and not
    // interruptible by stop_speech alone).
    Effect::new(move |_| {
        if let Some(c) = card.get() {
            if c.card_type != CardType::Kanji && is_speech_supported() {
                stop_current_audio();
                speak_word(&c.question, 1.0);
            }
        }
    });

    view! {
        <div data-testid=test_id_val class="flex flex-col">
            {move || {
                card.get().map(|c| {
                    let is_kanji = matches!(c.card_type, CardType::Kanji);
                    let card_type = c.card_type;
                    let question = c.question.clone();
                    let question_for_audio = c.question.clone();
                    let answer = c.answer.clone();
                    let readings = c.readings.clone();
                    let i18n_for_card = i18n;
                    let dont_know_label = t!(i18n_for_card, onboarding.scoring.dont_know).into_any();
                    let know_label = t!(i18n_for_card, onboarding.scoring.know).into_any();

                    let question_signal = Signal::derive(move || question.clone());
                    let answer_signal = Signal::derive(move || answer.clone());

                    view! {
                        <Card class=Signal::derive(|| "p-6 flex-1 flex flex-col".to_string())>
                            // Header row: Tag (left) + AudioButtons (right).
                            // Previously AudioButtons was absolutely positioned
                            // over the question text, causing overlap on long
                            // grammar titles.
                            <div class="flex items-center justify-between mb-4">
                                <Tag variant=Signal::derive(move || card_type.tag_variant())>
                                    {card_type.label(&i18n_for_card)}
                                </Tag>
                                {move || {
                                    if is_kanji {
                                        ().into_any()
                                    } else {
                                        view! {
                                            <AudioButtons
                                                text=question_for_audio.clone()
                                                audio_path=None
                                                class=Signal::derive(|| "".to_string())
                                                test_id=Signal::derive(|| "scoring-step-audio".to_string())
                                            />
                                        }.into_any()
                                    }
                                }}
                            </div>

                            // Question area: min-height stabilises the layout
                            // so the buttons below don't jump when card content
                            // (readings, answer length) varies between cards.
                            <div class="flex items-center justify-center scoring-card-question-area">
                                {move || {
                                    let q = question_signal.get();
                                    if is_kanji {
                                        view! {
                                            <div class="text-2xl" data-testid="scoring-step-question-kanji">
                                                {q}
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="text-2xl">
                                                <FuriganaText
                                                    text=q
                                                    known_kanji=HashSet::new()
                                                    test_id=Signal::derive(|| "scoring-step-question".to_string())
                                                />
                                            </div>
                                        }.into_any()
                                    }
                                }}
                            </div>

                            <div class="mt-4 text-center">
                                {move || {
                                    if let Some(value) = readings.clone() {
                                        view! {
                                            <div class="mb-2 scoring-step-readings" data-testid="scoring-step-readings">
                                                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                                    {value}
                                                </Text>
                                            </div>
                                        }.into_any()
                                    } else {
                                        ().into_any()
                                    }
                                }}
                                <MarkdownText
                                    content=answer_signal
                                    known_kanji=HashSet::new()
                                    test_id=Signal::derive(|| "scoring-step-answer".to_string())
                                />
                            </div>
                        </Card>

                        // Buttons are rendered OUTSIDE the Card so their
                        // position is stable relative to the card bottom,
                        // not affected by the card's variable content height.
                        <div class="grid grid-cols-2 gap-3 mt-4">
                            <Button
                                variant=ButtonVariant::Default
                                disabled=Signal::derive(move || is_rating.get())
                                on_click=Callback::new(move |_: leptos::ev::MouseEvent| on_dont_know.run(()))
                                test_id=Signal::derive(|| "scoring-step-dont-know".to_string())
                            >
                                {dont_know_label}
                                <span class="kbd-hint text-xs ml-1">"[1]"</span>
                            </Button>

                            <Button
                                variant=ButtonVariant::Olive
                                disabled=Signal::derive(move || is_rating.get())
                                on_click=Callback::new(move |_: leptos::ev::MouseEvent| on_know.run(()))
                                test_id=Signal::derive(|| "scoring-step-know".to_string())
                            >
                                {know_label}
                                <span class="kbd-hint text-xs ml-1">"[2]"</span>
                            </Button>
                        </div>
                    }
                })
            }}
        </div>
    }
}
