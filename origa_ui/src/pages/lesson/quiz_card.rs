use crate::i18n::*;
use crate::ui_components::{
    Card, DisplayText, FuriganaText, Heading, HeadingLevel, KanjiViewMode, KanjiWritingSection,
    Text, TextSize, TypographyVariant, is_speech_supported, speak_word, stop_current_audio,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, MultiQuizResult, NativeLanguage, QuizCard, QuizMode};
use std::collections::HashSet;
use tracing::warn;

use super::card_type::CardType;
use super::lesson_card_header::{LessonCardAudio, LessonCardTagsQuiz};
use super::next_card_button::NextCardButton;
use super::quiz_options::QuizOptions;
use super::quiz_options_multi::QuizOptionsMulti;
use super::quiz_result::QuizResult;
use super::quiz_result_display::QuizResultDisplay;

pub(crate) fn should_show_answer_display(result: QuizResult, card_type: CardType) -> bool {
    if card_type == CardType::Phrase {
        return true;
    }
    !matches!(result, QuizResult::Correct | QuizResult::MultiCorrect)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hides_answer_display_on_correct_for_non_phrase_cards() {
        assert!(!should_show_answer_display(
            QuizResult::Correct,
            CardType::Vocabulary
        ));
        assert!(!should_show_answer_display(
            QuizResult::Correct,
            CardType::Grammar
        ));
        assert!(!should_show_answer_display(
            QuizResult::Correct,
            CardType::Kanji
        ));
    }

    #[test]
    fn hides_answer_display_on_multi_correct_for_non_phrase_cards() {
        assert!(!should_show_answer_display(
            QuizResult::MultiCorrect,
            CardType::Vocabulary
        ));
    }

    #[test]
    fn shows_answer_display_on_incorrect_for_non_phrase_cards() {
        assert!(should_show_answer_display(
            QuizResult::Incorrect,
            CardType::Vocabulary
        ));
        assert!(should_show_answer_display(
            QuizResult::MultiPartial,
            CardType::Grammar
        ));
        assert!(should_show_answer_display(
            QuizResult::DontKnow,
            CardType::Kanji
        ));
    }

    #[test]
    fn always_shows_answer_display_for_phrase_cards_regardless_of_result() {
        assert!(should_show_answer_display(
            QuizResult::Correct,
            CardType::Phrase
        ));
        assert!(should_show_answer_display(
            QuizResult::MultiCorrect,
            CardType::Phrase
        ));
        assert!(should_show_answer_display(
            QuizResult::Incorrect,
            CardType::Phrase
        ));
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum QuizVariant {
    #[default]
    Meaning,
    Reading,
    Grammar,
}

#[component]
pub fn QuizCardView(
    quiz_card: QuizCard,
    show_result: Signal<bool>,
    selected_option: Option<usize>,
    on_select_option: Callback<usize>,
    on_dont_know: Callback<()>,
    dont_know_selected: Signal<bool>,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
    #[prop(optional)] quiz_variant: QuizVariant,
    #[prop(into, default = Signal::derive(|| HashSet::new()))] selected_options: Signal<
        HashSet<usize>,
    >,
    #[prop(default = Signal::derive(|| false))] multi_submitted: Signal<bool>,
    #[prop(default = None)] multi_result: Option<MultiQuizResult>,
    #[prop(default = Callback::new(|_: usize| {}))] on_toggle: Callback<usize>,
    #[prop(default = Callback::new(|_: ()| {}))] on_submit: Callback<()>,
    #[prop(default = Signal::derive(|| false))] waiting_for_next: Signal<bool>,
    #[prop(default = Callback::new(|_: ()| {}))] on_next_card: Callback<()>,
    #[prop(default = false)] lenient_grading: bool,
) -> impl IntoView {
    let i18n = use_i18n();
    let card = quiz_card.card().clone();
    let card_type = CardType::from(&card);
    let lang = native_language;
    let is_na_adj = super::na_adjective_helper::is_na_adjective_card(&card);
    let part_of_speech = card.vocabulary_part_of_speech();

    let question_text = match card.question(&lang) {
        Ok(q) => q.text().to_string(),
        Err(e) => {
            warn!(
                card_type = ?card_type,
                content_key = %card.content_key(),
                error = %e,
                "Failed to get card question"
            );
            String::new()
        },
    };
    let question = StoredValue::new(question_text.clone());
    let display_question = StoredValue::new(if is_na_adj {
        super::na_adjective_helper::append_na_suffix(&question_text)
    } else {
        question_text.clone()
    });
    let options: StoredValue<Vec<origa::domain::QuizOption>> =
        StoredValue::new(quiz_card.options().to_vec());
    let multi_result_stored = StoredValue::new(multi_result);

    let quiz_result = move || {
        if dont_know_selected.get() && show_result.get() {
            return QuizResult::DontKnow;
        }
        if let Some(selected) = selected_option {
            let opts = options.get_value();
            if let Some(opt) = opts.get(selected) {
                return if opt.is_correct() {
                    QuizResult::Correct
                } else {
                    QuizResult::Incorrect
                };
            }
        }
        QuizResult::None
    };

    let kanji_for_animation: StoredValue<Option<String>> = StoredValue::new(match &card {
        DomainCard::Kanji(_) => {
            let text = match card.question(&lang) {
                Ok(q) => q.text().to_string(),
                Err(e) => {
                    warn!(
                        card_type = ?card_type,
                        content_key = %card.content_key(),
                        error = %e,
                        "Failed to get card question"
                    );
                    String::new()
                },
            };
            Some(text)
        },
        _ => None,
    });

    let lesson_ctx = use_context::<super::lesson_state::LessonContext>();
    let question_text = question.get_value();

    Effect::new(move |_| {
        let is_muted = lesson_ctx
            .as_ref()
            .map(|ctx| ctx.is_muted.get_untracked())
            .unwrap_or(false);
        if !show_result.get() && card_type != CardType::Kanji && is_speech_supported() && !is_muted
        {
            speak_word(&question_text, 1.0);
        }
    });

    on_cleanup(move || {
        stop_current_audio();
    });

    let is_multi = quiz_card.mode() == QuizMode::Multi;

    view! {
        <div class="flex flex-col">
            <LessonCardTagsQuiz
                card_type=card_type
                quiz_variant=quiz_variant
                part_of_speech=part_of_speech
            />
            <Card class=Signal::derive(|| super::LESSON_CARD_CLASS.to_string()) shadow=true test_id="lesson-card-root">
                <LessonCardAudio
                    card_type=card_type
                    question_text=display_question.get_value()
                    is_reversed=false
                    audio_path=None
                />

            <div class="flex-1 flex flex-col justify-center">
                <div class="text-center mb-0.5 sm:mb-1">
                    <Show when=move || kanji_for_animation.get_value().is_none()>
                        <div class="mb-4">
                            <Heading level=HeadingLevel::H2>
                                <FuriganaText text=display_question.get_value() known_kanji=known_kanji.get() native_language=native_language with_kanji_tooltip=true/>
                            </Heading>
                        </div>
                    </Show>

                    <Show when=move || kanji_for_animation.get_value().is_some()>
                        {move || {
                            kanji_for_animation.get_value().map(|kanji: String| {
                                let kanji_clone = kanji.clone();
                                view! {
                                    <div class="mb-3 sm:mb-6">
                                        <DisplayText>
                                            {kanji}
                                        </DisplayText>
                                    </div>
                                    <KanjiWritingSection kanji=kanji_clone mode=KanjiViewMode::Animation />
                                }
                            })
                        }}
                    </Show>

                    <Text size=TextSize::Default variant=TypographyVariant::Muted class="mt-4">
                        {if is_multi && quiz_variant == QuizVariant::Reading {
                            t!(i18n, lesson.choose_all_readings).into_any()
                        } else {
                            match quiz_variant {
                                QuizVariant::Meaning => t!(i18n, lesson.choose_answer).into_any(),
                                QuizVariant::Reading => t!(i18n, lesson.choose_reading).into_any(),
                                QuizVariant::Grammar => t!(i18n, lesson.choose_grammar).into_any(),
                            }
                        }}
                    </Text>
                </div>

                <Show when=move || is_multi>
                    <QuizOptionsMulti
                        options=options.get_value()
                        selected_options=selected_options.get()
                        show_result=show_result
                        multi_submitted=multi_submitted
                        multi_result=multi_result_stored.get_value()
                        on_toggle=on_toggle
                        on_submit=on_submit
                        on_dont_know=on_dont_know
                        dont_know_selected=dont_know_selected
                        known_kanji=known_kanji
                        waiting_for_next=waiting_for_next
                        on_next_card=on_next_card
                    />
                    <Show when=move || multi_submitted.get()>
                        {move || {
                            multi_result_stored
                                .get_value()
                                .map(|r| {
                                    let result = if lenient_grading {
                                        QuizResult::from_multi_result_lenient(&r)
                                    } else {
                                        QuizResult::from_multi_result(&r)
                                    };
                                    view! {
                                        <QuizResultDisplay
                                            quiz_result=result
                                            multi_result=multi_result_stored.get_value()
                                            options=options.get_value()
                                        />
                                    }
                                })
                        }}
                    </Show>
                </Show>

                <Show when=move || !is_multi>
                    <QuizOptions
                        options=options.get_value()
                        selected_option=selected_option
                        show_result=show_result
                        quiz_result=quiz_result()
                        on_select_option=on_select_option
                        on_dont_know=on_dont_know
                        dont_know_selected=dont_know_selected
                        known_kanji=known_kanji
                    />

                    <Show when=move || show_result.get() && quiz_result() != QuizResult::DontKnow>
                        <QuizResultDisplay quiz_result=quiz_result() />
                    </Show>

                    <Show when=move || waiting_for_next.get() && show_result.get()>
                        <NextCardButton on_next_card=on_next_card />
                    </Show>
                </Show>
            </div>
        </Card>
        </div>
    }
}
